//! The palette: where the brush is chosen from.
//!
//! Four rows of choices, one per character of the token `c0007`'s [`Brush`]
//! writes - the block, how it behaves, the trigger it takes part in and the
//! group that trigger belongs to - so a level author picks what they are
//! painting off the screen instead of out of the format's documentation.
//!
//! Everything here is clickable *and* typeable. The letters are taken out of
//! tokens [`block_token`] wrote rather than restated in a table beside it, which
//! is what stops the palette from ever offering a letter the file would not be
//! read back with.
//!
//! # Why the letters need modifiers
//!
//! The format writes `A` for a Simple block, for a SittingDuck behaviour *and*
//! for a Start trigger; it tells them apart by where they sit in the token. A
//! keyboard has no positions to tell them apart by, so the axis becomes the
//! modifier and the letter stays the format's own: bare for the block, `Shift`
//! for the behaviour, `Alt` for the trigger, and the digit row for the group,
//! which collides with nothing either way. `Alt` rather than `Ctrl` because
//! [`commanding`] already spends `Ctrl`/`Cmd` on [`save`](super::save) and
//! [`history`](super::history), and `Ctrl+Z` must stay undo rather than becoming
//! the `Z` block.
//!
//! # Why the palette picks itself
//!
//! The same reason the settings panel does: a rectangle laid out once, drawn at
//! that rectangle and read a click against, so what is on screen and what is
//! clickable cannot come apart. It sits in a column of its own down the right
//! of the window, because the left one is already four panels deep.

use bevy::asset::AssetServer;
use bevy::color::palettes::css::{GOLD, SILVER};
use bevy::ecs::change_detection::{DetectChanges, DetectChangesMut};
use bevy::prelude::{BackgroundColor, ButtonInput, Color, Commands, Component, Entity, GlobalZIndex, Justify, KeyCode, MouseButton, Node, Query, Rect, Res, ResMut, Resource, UiRect, Val, Vec2, With};
use bevy::ui::BorderColor;
use bevy::window::{PrimaryWindow, Window};

use crate::block::trigger::{TriggerGroup, TriggerType};
use crate::block::{block_colours, BlockBehaviour, BlockType};
use crate::editor::settings::{panel_node, panel_rect, panel_text, COLUMN_GAP, PANEL_BACKGROUND, PANEL_ORIGIN, PANEL_PADDING, PANEL_Z, ROW_FONT_SIZE, ROW_HEIGHT, ROW_INSET, ROW_WIDTH, TITLE_FONT_SIZE, TITLE_HEIGHT};
use crate::level::layout::block_token;

use super::{commanding, Brush, EditorEntity};

/// The speed the format reads every evader back at.
///
/// `make_block` has no room for a speed, so an evader painted from here is the
/// evader a saved level reloads as - anything else would be a brush that paints
/// something the file cannot say.
const EVADER_SPEED: f32 = 50.0;

/// How tall a swatch is. Taller than a row, because a swatch is a colour and a
/// colour wants room to be one.
const SWATCH_HEIGHT: f32 = 36.0;

/// The width of the letter's own column in a text row, so nine labels line up
/// under each other in a font that is not monospaced.
const LETTER_WIDTH: f32 = 22.0;

/// How thick the outline around the chosen entry of a row is.
const CHOSEN_BORDER: f32 = 2.0;

/// The colour of what is not chosen. A swatch brings its own.
const ENTRY_BACKGROUND: Color = Color::srgba(0.18, 0.18, 0.24, 0.95);

/// One thing the palette can set: a character of the token, or the digit behind
/// it.
///
/// The `Option`s are the format's `.` - the block that is not there, which is
/// the erase brush, and the trigger that is not there, which is a block in no
/// group at all.
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteEntry {
    Block(Option<BlockType>),
    Behaviour(BlockBehaviour),
    Trigger(Option<TriggerType>),
    Group(TriggerGroup),
}

/// Which key names an entry, and what has to be held down with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shortcut {
    pub modifier: Modifier,
    pub key: KeyCode,
}

/// Which of the token's characters a press is aimed at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    /// The block - the first character, and the one changed most often, so it
    /// keeps the bare letter.
    None,
    /// The behaviour - the second character.
    Shift,
    /// The trigger - the third character.
    Alt,
}

impl Modifier {
    /// The keys any one of which counts as this modifier being held.
    pub fn keys(&self) -> &'static [KeyCode] {
        match self {
            Modifier::None => &[],
            Modifier::Shift => &[KeyCode::ShiftLeft, KeyCode::ShiftRight],
            Modifier::Alt => &[KeyCode::AltLeft, KeyCode::AltRight],
        }
    }

    /// How the shortcut is written on screen, in the heading of the row it
    /// belongs to.
    fn named(&self) -> &'static str {
        match self {
            Modifier::None => "",
            Modifier::Shift => "  (Shift)",
            Modifier::Alt => "  (Alt)",
        }
    }
}

impl PaletteEntry {
    /// The character the format writes this entry with, taken back out of a
    /// token [`block_token`] wrote - so the palette cannot drift from the file.
    pub fn letter(&self) -> char {
        match self {
            PaletteEntry::Block(None) | PaletteEntry::Trigger(None) => EMPTY_LETTER,

            PaletteEntry::Block(Some(block_type)) => {
                token_char(block_token(block_type, &BlockBehaviour::SittingDuck, None), 0)
            }

            PaletteEntry::Behaviour(behaviour) => {
                token_char(block_token(&BlockType::Simple, behaviour, None), 1)
            }

            PaletteEntry::Trigger(Some(trigger)) => token_char(
                block_token(&BlockType::Simple, &BlockBehaviour::SittingDuck, Some((trigger, 0))),
                2,
            ),

            PaletteEntry::Group(group) => {
                char::from_digit(*group as u32, 10).expect("the palette only offers 0..=9")
            }
        }
    }

    /// Which character of the token this entry is, which is the modifier it is
    /// reached under.
    pub fn modifier(&self) -> Modifier {
        match self {
            PaletteEntry::Block(_) | PaletteEntry::Group(_) => Modifier::None,
            PaletteEntry::Behaviour(_) => Modifier::Shift,
            PaletteEntry::Trigger(_) => Modifier::Alt,
        }
    }

    /// The press that chooses this entry.
    ///
    /// `None` for a letter no key is named after, which is the format growing a
    /// character this module has not been told about - see
    /// `every_entry_can_be_typed`.
    pub fn shortcut(&self) -> Option<Shortcut> {
        Some(Shortcut { modifier: self.modifier(), key: key_for(self.letter())? })
    }

    /// What the entry is called, for the rows that are words rather than
    /// colour. A swatch and a digit say what they are by being what they are.
    pub fn label(&self) -> &'static str {
        match self {
            PaletteEntry::Block(_) | PaletteEntry::Group(_) => "",

            PaletteEntry::Behaviour(behaviour) => match behaviour {
                BlockBehaviour::SittingDuck => "Sitting duck",
                BlockBehaviour::Spinner => "Spinner",
                BlockBehaviour::Vanisher => "Vanisher",
                BlockBehaviour::Repuslor => "Repulsor",
                BlockBehaviour::EvaderR(_) => "Evader, right",
                BlockBehaviour::EvaderL(_) => "Evader, left",
                BlockBehaviour::EvaderU(_) => "Evader, up",
                BlockBehaviour::EvaderD(_) => "Evader, down",
                BlockBehaviour::Portal => "Portal",
            },

            PaletteEntry::Trigger(None) => "No trigger",
            PaletteEntry::Trigger(Some(trigger)) => match trigger {
                TriggerType::Start => "Start",
                TriggerType::Stop => "Stop",
                TriggerType::StartStop => "Start and stop",
                TriggerType::ReceiverStartingInactive => "Receiver, starts off",
                TriggerType::ReceiverStartingActive => "Receiver, starts on",
            },
        }
    }

    /// Sets this entry's own part of the brush and leaves the rest alone -
    /// which is what makes the palette four independent rows rather than one
    /// list of whole brushes.
    pub fn apply(&self, brush: &mut Brush, group: &mut BrushGroup) {
        match self {
            // Choosing a block is choosing to paint one, so it is also how the
            // erase brush is put down again. The two are one row on screen for
            // exactly that reason: `.` is the format's own "no block here", and
            // erasing is painting it.
            PaletteEntry::Block(None) => brush.erase = true,
            PaletteEntry::Block(Some(block_type)) => {
                brush.erase = false;
                brush.block_type = block_type.clone();
            }

            PaletteEntry::Behaviour(behaviour) => brush.behaviour = behaviour.clone(),

            PaletteEntry::Trigger(None) => brush.trigger = None,
            PaletteEntry::Trigger(Some(trigger)) => {
                brush.trigger = Some((trigger.clone(), group.0))
            }

            PaletteEntry::Group(chosen) => {
                group.0 = *chosen;

                if let Some((trigger, _)) = &brush.trigger {
                    brush.trigger = Some((trigger.clone(), *chosen));
                }
            }
        }
    }

    /// Whether this is the entry the brush is set to - what the outline on
    /// screen is drawn around.
    ///
    /// Behaviours are compared by their letter rather than by themselves,
    /// because an evader carries a speed the format has no room for: a brush
    /// holding `EvaderR(30.0)` still paints `AE`, and a palette that refused to
    /// show `E` as chosen would be describing a block nobody can author.
    pub fn is_chosen(&self, brush: &Brush, group: &BrushGroup) -> bool {
        match self {
            PaletteEntry::Block(None) => brush.erase,
            PaletteEntry::Block(Some(block_type)) => !brush.erase && brush.block_type == *block_type,

            PaletteEntry::Behaviour(behaviour) => {
                behaviour_letter(behaviour) == behaviour_letter(&brush.behaviour)
            }

            PaletteEntry::Trigger(trigger) => {
                brush.trigger.as_ref().map(|(chosen, _)| chosen) == trigger.as_ref()
            }

            PaletteEntry::Group(chosen) => group.0 == *chosen,
        }
    }
}

/// The character the format has no character for: the empty half of a token.
const EMPTY_LETTER: char = '.';

fn behaviour_letter(behaviour: &BlockBehaviour) -> char {
    PaletteEntry::Behaviour(behaviour.clone()).letter()
}

/// The character at `position` of a token, or [`EMPTY_LETTER`] where the token
/// stops - a block in no trigger group is a two-character token whose third
/// character is the absence of one.
fn token_char(token: String, position: usize) -> char {
    token.chars().nth(position).unwrap_or(EMPTY_LETTER)
}

/// The key a format letter is typed on.
fn key_for(letter: char) -> Option<KeyCode> {
    Some(match letter {
        'A' => KeyCode::KeyA,
        'B' => KeyCode::KeyB,
        'C' => KeyCode::KeyC,
        'D' => KeyCode::KeyD,
        'E' => KeyCode::KeyE,
        'F' => KeyCode::KeyF,
        'G' => KeyCode::KeyG,
        'H' => KeyCode::KeyH,
        'I' => KeyCode::KeyI,
        'R' => KeyCode::KeyR,
        'S' => KeyCode::KeyS,
        'Z' => KeyCode::KeyZ,

        '0' => KeyCode::Digit0,
        '1' => KeyCode::Digit1,
        '2' => KeyCode::Digit2,
        '3' => KeyCode::Digit3,
        '4' => KeyCode::Digit4,
        '5' => KeyCode::Digit5,
        '6' => KeyCode::Digit6,
        '7' => KeyCode::Digit7,
        '8' => KeyCode::Digit8,
        '9' => KeyCode::Digit9,

        EMPTY_LETTER => KeyCode::Period,

        _ => return None,
    })
}

/// The trigger group a trigger type joins when one is chosen.
///
/// The brush holds its trigger as a type *and* a group or as neither (`c0007`),
/// which leaves nowhere to put a group picked before a type. Here is that
/// somewhere: the digit row is live whichever way round the author works, so
/// `3` and then `Start` paints the same token as `Start` and then `3`.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BrushGroup(pub TriggerGroup);

/// The window width the palette on screen was laid out for.
///
/// The palette is drawn into absolute pixels, so the rectangles it was drawn at
/// are the rectangles it *is* at until it is drawn again - however wide the
/// window has become in the meantime. Reading a click against the live width
/// instead would be exactly the drift this module says cannot happen: on the
/// frame a window is dragged narrower, the entries are still where they were put
/// and only this says where that was.
///
/// Written by [`editor_show_palette`], and by nothing else - it is a record of
/// what is on screen, not a setting.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct PaletteWidth(pub f32);

/// Marks everything the palette draws, so a moved brush can take the whole of
/// it down and put it up again with the outline somewhere else.
#[derive(Component)]
pub struct PalettePanel;

/// An entry on screen, tagged with what choosing it would set - as the settings
/// panel tags its buttons with the setting they step.
///
/// On the rectangle that is filled in and outlined, which is the one a test can
/// read the swatch's colour off. A `Node` carries a `BackgroundColor` and a
/// `BorderColor` whether or not anybody asked it to, so the letter written over
/// a swatch is otherwise indistinguishable from the swatch itself.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct PaletteChoice(pub PaletteEntry);


// --- layout ---------------------------------------------------------------

/// Every block type the format defines, in the order it writes them.
pub fn block_types() -> Vec<BlockType> {
    vec![
        BlockType::Simple,
        BlockType::Hardling,
        BlockType::Concrete,
        BlockType::SimpleTop,
        BlockType::Obstacle,
    ]
}

/// Every behaviour the format defines, `A` through `I`.
pub fn behaviours() -> Vec<BlockBehaviour> {
    vec![
        BlockBehaviour::SittingDuck,
        BlockBehaviour::Spinner,
        BlockBehaviour::Vanisher,
        BlockBehaviour::Repuslor,
        BlockBehaviour::EvaderR(EVADER_SPEED),
        BlockBehaviour::EvaderL(EVADER_SPEED),
        BlockBehaviour::EvaderU(EVADER_SPEED),
        BlockBehaviour::EvaderD(EVADER_SPEED),
        BlockBehaviour::Portal,
    ]
}

/// Every trigger type the format defines, with "none" in front of them.
pub fn trigger_types() -> Vec<Option<TriggerType>> {
    vec![
        None,
        Some(TriggerType::Start),
        Some(TriggerType::Stop),
        Some(TriggerType::StartStop),
        Some(TriggerType::ReceiverStartingInactive),
        Some(TriggerType::ReceiverStartingActive),
    ]
}

/// The groups the format has a digit for.
pub const TRIGGER_GROUPS: std::ops::RangeInclusive<TriggerGroup> = 0..=9;

/// What the palette draws, and where.
#[derive(Debug, Clone, PartialEq)]
pub struct PaletteItem {
    /// In logical window pixels, the space `Window::cursor_position` reads in.
    pub rect: Rect,
    pub kind: PaletteItemKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PaletteItemKind {
    /// The row at the top that says what the brush is set to.
    CurrentBrush,
    Heading(String),
    Entry(PaletteEntry),
}

/// Where the palette's column starts.
///
/// Down the right, where the settings panel, the history bar, the save panel and
/// the playtest panel are four deep down the left. The window's width is the one
/// thing on screen the editor does not otherwise need to know, and it is the
/// price of the palette not landing on top of them.
///
/// Never further left than the column down the left, whatever the window does.
/// A window too narrow for both columns has to give something up, and what it
/// gives up is the right-hand end of the palette running off the edge: every
/// entry still on screen is still exactly one entry, where a palette lying over
/// the settings panel would have clicks land on both at once. Nothing is lost
/// outright either way - an entry off the edge is still reachable by its
/// shortcut, which is what `Shift` and `Alt` are for.
fn palette_left(window_width: f32) -> f32 {
    let against_the_right_edge = window_width - PANEL_ORIGIN.x - ROW_WIDTH - 2.0 * PANEL_PADDING;

    against_the_right_edge.max(panel_rect().max.x + PANEL_GAP)
}

/// How much room the palette leaves between itself and the column down the
/// left, on a window narrow enough for the two to meet.
const PANEL_GAP: f32 = 8.0;

/// Everything the palette puts on screen, laid out top to bottom.
///
/// The one place the panel's shape is decided: the drawing and the clicking both
/// read it, so an entry cannot be somewhere other than where it looks.
pub fn palette_items(window_width: f32) -> Vec<PaletteItem> {
    let left = palette_left(window_width) + PANEL_PADDING;
    let mut items = vec![];
    let mut top = PANEL_ORIGIN.y + PANEL_PADDING;

    let full_row = |items: &mut Vec<PaletteItem>, top: &mut f32, height: f32, kind| {
        items.push(PaletteItem {
            rect: Rect::new(left, *top, left + ROW_WIDTH, *top + height),
            kind,
        });
        *top += height;
    };

    full_row(&mut items, &mut top, TITLE_HEIGHT, PaletteItemKind::CurrentBrush);

    // The blocks, as swatches across one row - with the erase brush riding along
    // at the end of it, because it is the same character of the same token.
    full_row(&mut items, &mut top, ROW_HEIGHT, heading("BLOCK", Modifier::None));

    let blocks: Vec<PaletteEntry> = block_types()
        .into_iter()
        .map(Some)
        .chain([None])
        .map(PaletteEntry::Block)
        .collect();

    for (index, entry) in blocks.iter().enumerate() {
        items.push(PaletteItem {
            rect: across(left, top, index, blocks.len(), SWATCH_HEIGHT, 0.0),
            kind: PaletteItemKind::Entry(entry.clone()),
        });
    }
    top += SWATCH_HEIGHT;

    full_row(&mut items, &mut top, ROW_HEIGHT, heading("BEHAVIOUR", Modifier::Shift));
    for behaviour in behaviours() {
        let entry = PaletteEntry::Behaviour(behaviour);
        full_row(&mut items, &mut top, ROW_HEIGHT, PaletteItemKind::Entry(entry));
    }

    full_row(&mut items, &mut top, ROW_HEIGHT, heading("TRIGGER", Modifier::Alt));
    for trigger in trigger_types() {
        let entry = PaletteEntry::Trigger(trigger);
        full_row(&mut items, &mut top, ROW_HEIGHT, PaletteItemKind::Entry(entry));
    }

    full_row(&mut items, &mut top, ROW_HEIGHT, heading("GROUP", Modifier::None));
    for group in TRIGGER_GROUPS {
        items.push(PaletteItem {
            rect: across(left, top, group as usize, TRIGGER_GROUPS.count(), ROW_HEIGHT, ROW_INSET),
            kind: PaletteItemKind::Entry(PaletteEntry::Group(group)),
        });
    }

    items
}

/// A heading that says which key its row answers to, so the modifier is on
/// screen rather than in the commit message.
fn heading(name: &str, modifier: Modifier) -> PaletteItemKind {
    PaletteItemKind::Heading(format!("{name}{}", modifier.named()))
}

/// The `index`th of `count` cells sharing one row's width, kept clear of the
/// rows above and below it by `inset`.
fn across(left: f32, top: f32, index: usize, count: usize, height: f32, inset: f32) -> Rect {
    let width = (ROW_WIDTH - (count - 1) as f32 * COLUMN_GAP) / count as f32;
    let cell_left = left + index as f32 * (width + COLUMN_GAP);

    Rect::new(cell_left, top + inset, cell_left + width, top + height - inset)
}

/// The whole panel's footprint - what the editor keeps its hands off.
///
/// Taken off what is actually in it rather than stated again, so a row added to
/// [`palette_items`] cannot end up hanging outside the panel it is drawn on or,
/// worse, outside the area the pointer treats as the palette.
pub fn palette_rect(window_width: f32) -> Rect {
    let items = palette_items(window_width);

    let mut bounds = items.first().expect("the palette is never empty").rect;
    for item in &items {
        bounds = bounds.union(item.rect);
    }

    Rect::new(
        bounds.min.x - PANEL_PADDING,
        bounds.min.y - PANEL_PADDING,
        bounds.max.x + PANEL_PADDING,
        bounds.max.y + PANEL_PADDING,
    )
}

/// The entry a click at `pixel` is aimed at.
///
/// `None` for a click anywhere else, including inside the panel but not on an
/// entry - the panel swallows those, as the settings panel does.
pub fn palette_entry_at(pixel: Vec2, window_width: f32) -> Option<PaletteEntry> {
    palette_items(window_width).into_iter().find_map(|item| match item.kind {
        PaletteItemKind::Entry(entry) if item.rect.contains(pixel) => Some(entry),
        _ => None,
    })
}


// --- the two ways in --------------------------------------------------------

/// Chooses the entry the pointer is on.
///
/// The press rather than the hold, as the settings panel reads its buttons: a
/// drag across the digit row is one choice, not ten.
pub fn editor_palette_click(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    drawn: Res<PaletteWidth>,
    mut brush: ResMut<Brush>,
    mut group: ResMut<BrushGroup>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(window) = windows.iter().next() else { return; };
    let Some(cursor) = window.cursor_position() else { return; };

    // The width the entries were drawn at, not the width the window is now: a
    // click lands on what the author can see.
    let Some(entry) = palette_entry_at(cursor, drawn.0) else { return; };

    choose(&entry, &mut brush, &mut group);
}

/// Chooses the entry a key names.
pub fn editor_palette_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    mut brush: ResMut<Brush>,
    mut group: ResMut<BrushGroup>,
) {
    let Some(entry) = entry_typed(&keys) else { return; };

    choose(&entry, &mut brush, &mut group);
}

/// The entry the keyboard is naming, if any.
///
/// The modifier is read first and the letter only inside the row it names, so
/// `Shift+A` is the SittingDuck behaviour and not also the Simple block: one
/// press belongs to one row.
///
/// A chord under [`commanding`] belongs to nobody here. Without that, `Ctrl+Z`
/// would undo an edit *and* pick up the `Z` block on the way past, and `Ctrl+S`
/// would save the level and change the brush while it did.
pub fn entry_typed(keys: &ButtonInput<KeyCode>) -> Option<PaletteEntry> {
    if commanding(keys) {
        return None;
    }

    let held = if keys.any_pressed(Modifier::Alt.keys().iter().copied()) {
        Modifier::Alt
    } else if keys.any_pressed(Modifier::Shift.keys().iter().copied()) {
        Modifier::Shift
    } else {
        Modifier::None
    };

    palette_entries().into_iter().find(|entry| {
        entry
            .shortcut()
            .is_some_and(|shortcut| shortcut.modifier == held && keys.just_pressed(shortcut.key))
    })
}

/// Every entry the palette offers, whatever the window is doing - the list the
/// keyboard reads, where the mouse reads the rectangles.
pub fn palette_entries() -> Vec<PaletteEntry> {
    let blocks = block_types().into_iter().map(Some).chain([None]).map(PaletteEntry::Block);
    let behaviours = behaviours().into_iter().map(PaletteEntry::Behaviour);
    let triggers = trigger_types().into_iter().map(PaletteEntry::Trigger);
    let groups = TRIGGER_GROUPS.map(PaletteEntry::Group);

    blocks.chain(behaviours).chain(triggers).chain(groups).collect()
}

/// Whether the palette on screen is still the palette this editor would draw.
///
/// Three reasons it might not be: the brush moved, the group the next trigger
/// would join moved, or the window changed width underneath it. The last is
/// asked as a *state* - the width on screen against the width of the window -
/// rather than by listening for a resize, because a state cannot be missed and
/// answers correctly however the window came to be that wide.
pub fn the_palette_is_out_of_date(
    brush: Res<Brush>,
    group: Res<BrushGroup>,
    drawn: Res<PaletteWidth>,
    windows: Query<&Window, With<PrimaryWindow>>,
) -> bool {
    brush.is_changed()
        || group.is_changed()
        || windows.iter().next().is_some_and(|window| window.width() != drawn.0)
}

/// One place where an entry becomes a brush, whether it was clicked or typed -
/// so the two ways in cannot come to mean different things.
///
/// Written around change detection, because the palette is redrawn whenever the
/// brush moves and choosing what is already chosen would redraw it once a frame
/// for as long as a key is held.
fn choose(entry: &PaletteEntry, brush: &mut ResMut<Brush>, group: &mut ResMut<BrushGroup>) {
    let (mut moved_brush, mut moved_group) = (brush.clone(), **group);

    entry.apply(&mut moved_brush, &mut moved_group);

    if moved_brush != **brush {
        **brush = moved_brush;
    }

    if moved_group != **group {
        **group = moved_group;
    }
}


// --- drawing --------------------------------------------------------------

/// Puts the palette on screen, and puts it there again whenever the brush moves.
///
/// The whole panel is rebuilt rather than the outline moved, as the settings
/// panel is: forty nodes is nothing, and this only runs on a frame the brush
/// actually changed.
pub fn editor_show_palette(
    brush: Res<Brush>,
    group: Res<BrushGroup>,
    windows: Query<&Window, With<PrimaryWindow>>,
    shown: Query<Entity, With<PalettePanel>>,
    asset_server: Res<AssetServer>,
    mut drawn: ResMut<PaletteWidth>,
    mut commands: Commands,
) {
    for entity in &shown {
        commands.entity(entity).despawn();
    }

    let Some(window) = windows.iter().next() else { return; };
    let width = window.width();

    // Said before anything is spawned, because it is what everything spawned
    // below will be read against.
    drawn.set_if_neq(PaletteWidth(width));

    commands.spawn((
        panel_node(palette_rect(width)),
        BackgroundColor(PANEL_BACKGROUND),
        GlobalZIndex(PANEL_Z),
        PalettePanel,
        EditorEntity,
    ));

    for item in palette_items(width) {
        match &item.kind {
            // What the brush is set to, spelled as the token it paints - which
            // is the same string the file will be holding.
            PaletteItemKind::CurrentBrush => {
                commands.spawn((
                    panel_text(
                        item.rect,
                        &format!("Brush   {}", brush.token()),
                        TITLE_FONT_SIZE,
                        GOLD.into(),
                        Justify::Left,
                        &asset_server,
                    ),
                    PalettePanel,
                    EditorEntity,
                ));
            }

            PaletteItemKind::Heading(heading) => {
                commands.spawn((
                    panel_text(
                        item.rect,
                        heading,
                        ROW_FONT_SIZE,
                        SILVER.into(),
                        Justify::Left,
                        &asset_server,
                    ),
                    PalettePanel,
                    EditorEntity,
                ));
            }

            PaletteItemKind::Entry(entry) => {
                spawn_entry(entry, item.rect, &brush, &group, &asset_server, &mut commands);
            }
        }
    }
}

/// One entry: a swatch in the block's own colour, a digit, or a labelled row -
/// outlined when it is what the brush is set to.
fn spawn_entry(
    entry: &PaletteEntry,
    rect: Rect,
    brush: &Brush,
    group: &BrushGroup,
    asset_server: &Res<AssetServer>,
    commands: &mut Commands,
) {
    let chosen = entry.is_chosen(brush, group);
    let ink: Color = if chosen { GOLD.into() } else { SILVER.into() };

    // A block is its colour, so the swatch is that colour with the format's
    // letter written on it rather than a word for it.
    let (background, split) = match entry {
        PaletteEntry::Block(Some(block_type)) => {
            let (color1, color2, split) = block_colours(block_type);
            (color1, split.then_some(color2))
        }

        _ => (ENTRY_BACKGROUND, None),
    };

    commands.spawn((
        Node {
            border: UiRect::all(Val::Px(CHOSEN_BORDER)),
            ..panel_node(rect)
        },
        BackgroundColor(background),
        BorderColor::all(if chosen { GOLD.into() } else { Color::NONE }),
        GlobalZIndex(PANEL_Z),
        PaletteChoice(entry.clone()),
        PalettePanel,
        EditorEntity,
    ));

    // `SimpleTop` is two colours, and a swatch that showed only one of them
    // would be a palette lying about what it paints. Spawned after the swatch
    // and at the same depth, so it lands on top of it.
    if let Some(color2) = split {
        let top_half = Rect::new(rect.min.x, rect.min.y, rect.max.x, rect.min.y + rect.height() / 2.0);

        commands.spawn((
            panel_node(top_half.inflate(-CHOSEN_BORDER)),
            BackgroundColor(color2),
            GlobalZIndex(PANEL_Z),
            PalettePanel,
            EditorEntity,
        ));
    }

    let letter = entry.letter().to_string();

    // A swatch and a digit are their own label, centred; a behaviour and a
    // trigger need words next to the letter, because neither changes a colour.
    let (letter_rect, justify, letter_ink) = match entry {
        PaletteEntry::Block(_) => (rect, Justify::Center, ink_on(background)),
        PaletteEntry::Group(_) => (rect, Justify::Center, ink),

        _ => (
            Rect::new(rect.min.x + COLUMN_GAP, rect.min.y, rect.min.x + LETTER_WIDTH, rect.max.y),
            Justify::Left,
            ink,
        ),
    };

    commands.spawn((
        panel_text(letter_rect, &letter, ROW_FONT_SIZE, letter_ink, justify, asset_server),
        PalettePanel,
        EditorEntity,
    ));

    if !entry.label().is_empty() {
        let label_rect = Rect::new(letter_rect.max.x + COLUMN_GAP, rect.min.y, rect.max.x, rect.max.y);

        commands.spawn((
            panel_text(label_rect, entry.label(), ROW_FONT_SIZE, ink, Justify::Left, asset_server),
            PalettePanel,
            EditorEntity,
        ));
    }
}

/// Black on a light swatch, white on a dark one - so the letter on the
/// `Concrete` swatch is as readable as the one on the `Obstacle` swatch.
fn ink_on(background: Color) -> Color {
    let srgba = background.to_srgba();
    let luminance = 0.2126 * srgba.red + 0.7152 * srgba.green + 0.0722 * srgba.blue;

    if luminance > 0.5 { Color::BLACK } else { Color::WHITE }
}


#[cfg(test)]
mod tests {
    use super::*;

    use crate::level::layout::EMPTY_SLOT;

    /// A window the size the game's own is, for the layout to be read against.
    const WINDOW: f32 = 1600.0;

    fn entries() -> Vec<PaletteEntry> {
        palette_entries()
    }

    /// The letters of every entry of one kind, in the order the palette offers
    /// them.
    fn letters(of: impl Fn(&PaletteEntry) -> bool) -> String {
        entries().iter().filter(|entry| of(entry)).map(PaletteEntry::letter).collect()
    }

    fn is_block(entry: &PaletteEntry) -> bool { matches!(entry, PaletteEntry::Block(_)) }
    fn is_behaviour(entry: &PaletteEntry) -> bool { matches!(entry, PaletteEntry::Behaviour(_)) }
    fn is_trigger(entry: &PaletteEntry) -> bool { matches!(entry, PaletteEntry::Trigger(_)) }
    fn is_group(entry: &PaletteEntry) -> bool { matches!(entry, PaletteEntry::Group(_)) }

    /// The brush the four entries make, from the brush the editor starts with.
    fn brush_of(chosen: &[PaletteEntry]) -> Brush {
        let mut brush = Brush::default();
        let mut group = BrushGroup::default();

        for entry in chosen {
            entry.apply(&mut brush, &mut group);
        }

        brush
    }


    // --- what is in it ----------------------------------------------------

    /// The card's first three criteria, asked of the format rather than of a
    /// list: every letter `make_block` reads is a letter the palette offers, in
    /// the order the format documents them.
    #[test]
    fn every_letter_the_format_defines_is_in_the_palette() {
        assert_eq!(letters(is_block), "ABCDZ.", "the five block types, and the erase brush");
        assert_eq!(letters(is_behaviour), "ABCDEFGHI", "the nine behaviours");
        assert_eq!(letters(is_trigger), ".ABCRS", "no trigger, the three kinds and the two receivers");
        assert_eq!(letters(is_group), "0123456789", "the ten groups the format has a digit for");
    }

    /// Not a letter twice inside a row either - a palette with two `A`
    /// behaviours would have one of them unreachable and neither obviously so.
    #[test]
    fn no_row_offers_the_same_letter_twice() {
        for (name, of) in [
            ("block", is_block as fn(&PaletteEntry) -> bool),
            ("behaviour", is_behaviour),
            ("trigger", is_trigger),
            ("group", is_group),
        ] {
            let mut seen: Vec<char> = letters(of).chars().collect();
            let before = seen.len();
            seen.sort();
            seen.dedup();

            assert_eq!(seen.len(), before, "the {name} row offers a letter twice");
        }
    }


    // --- the shortcuts ----------------------------------------------------

    /// The card's fifth criterion. A shortcut that came back `None` would be a
    /// click-only entry, which is the thing the card asks for the absence of.
    #[test]
    fn every_entry_can_be_typed() {
        for entry in entries() {
            assert!(entry.shortcut().is_some(), "{entry:?} has no key to be reached by");
        }
    }

    /// The answer to the collision the format's letters have: `A` names three
    /// entries, and the modifier is what tells them apart. Two entries under one
    /// press would mean one of them could never be chosen.
    #[test]
    fn no_two_entries_answer_to_the_same_press() {
        let mut presses: Vec<(Shortcut, PaletteEntry)> = vec![];

        for entry in entries() {
            let shortcut = entry.shortcut().expect("every entry can be typed");

            if let Some((_, other)) = presses.iter().find(|(seen, _)| *seen == shortcut) {
                panic!("{entry:?} and {other:?} both answer to {shortcut:?}");
            }

            presses.push((shortcut, entry));
        }
    }

    /// Which press belongs to which character of the token - the scheme the
    /// card's question settled on, written down where a change to it would fail.
    #[test]
    fn the_modifier_says_which_character_of_the_token_a_press_is_aimed_at() {
        assert_eq!(PaletteEntry::Block(Some(BlockType::Simple)).modifier(), Modifier::None);
        assert_eq!(PaletteEntry::Block(None).modifier(), Modifier::None);
        assert_eq!(PaletteEntry::Behaviour(BlockBehaviour::SittingDuck).modifier(), Modifier::Shift);
        assert_eq!(PaletteEntry::Trigger(Some(TriggerType::Start)).modifier(), Modifier::Alt);
        assert_eq!(PaletteEntry::Trigger(None).modifier(), Modifier::Alt);
        assert_eq!(PaletteEntry::Group(3).modifier(), Modifier::None);

        // And all three `A`s land on the same key, which is the point of the
        // modifier being the only thing that differs.
        for entry in [
            PaletteEntry::Block(Some(BlockType::Simple)),
            PaletteEntry::Behaviour(BlockBehaviour::SittingDuck),
            PaletteEntry::Trigger(Some(TriggerType::Start)),
        ] {
            assert_eq!(entry.shortcut().unwrap().key, KeyCode::KeyA, "{entry:?}");
        }
    }

    /// A key the keyboard reports as held has to be read by the row that owns
    /// it and by no other. Without the modifier being read first, `Shift+A`
    /// would set the behaviour *and* the block on its way past.
    #[test]
    fn a_press_belongs_to_exactly_one_row() {
        for entry in entries() {
            let shortcut = entry.shortcut().expect("every entry can be typed");

            let mut keys = ButtonInput::<KeyCode>::default();
            for modifier in shortcut.modifier.keys() {
                keys.press(*modifier);
            }
            keys.press(shortcut.key);

            assert_eq!(entry_typed(&keys), Some(entry.clone()), "{entry:?} is not what its own press chose");
        }
    }

    /// `Ctrl` is spent: `Ctrl+Z` is `c0011`'s undo and `Ctrl+S` is `c0012`'s
    /// save, and neither may also pick up the `Z` block or the `S` receiver on
    /// the way through.
    #[test]
    fn a_chord_the_editor_owns_is_not_also_a_palette_entry() {
        for commander in [KeyCode::ControlLeft, KeyCode::SuperLeft] {
            for (letter, held) in [
                (KeyCode::KeyZ, vec![]),
                (KeyCode::KeyS, vec![]),
                (KeyCode::KeyY, vec![]),
                (KeyCode::KeyZ, vec![KeyCode::ShiftLeft]),
            ] {
                let mut keys = ButtonInput::<KeyCode>::default();
                keys.press(commander);
                for modifier in &held {
                    keys.press(*modifier);
                }
                keys.press(letter);

                assert_eq!(
                    entry_typed(&keys),
                    None,
                    "{commander:?} + {held:?} + {letter:?} is the editor's chord, not the palette's"
                );
            }
        }
    }


    // --- what choosing does -----------------------------------------------

    /// The whole alphabet, through the palette: every token the format defines,
    /// spelled by choosing the entries whose letters make it up.
    ///
    /// This is the card's "clicking an entry sets the corresponding part of the
    /// brush" asked of all 2295 of them at once - and it is what would fail if
    /// an entry ever set a part of the brush that is not its own.
    #[test]
    fn choosing_the_entries_of_a_token_paints_that_token() {
        let mut checked = 0;

        for block_type in block_types() {
            for behaviour in behaviours() {
                for trigger in trigger_types() {
                    let groups: Vec<Option<TriggerGroup>> = match trigger {
                        None => vec![None],
                        Some(_) => TRIGGER_GROUPS.map(Some).collect(),
                    };

                    for group in groups {
                        let mut chosen = vec![
                            PaletteEntry::Block(Some(block_type.clone())),
                            PaletteEntry::Behaviour(behaviour.clone()),
                        ];

                        // The group first, so that the digit row is being asked
                        // to be live before a trigger type exists - which is
                        // the harder way round and the one `BrushGroup` is for.
                        if let Some(group) = group {
                            chosen.push(PaletteEntry::Group(group));
                        }
                        chosen.push(PaletteEntry::Trigger(trigger.clone()));

                        let expected = block_token(
                            &block_type,
                            &behaviour,
                            trigger.as_ref().zip(group),
                        );

                        assert_eq!(
                            brush_of(&chosen).token(),
                            expected,
                            "chose {chosen:?}"
                        );

                        checked += 1;
                    }
                }
            }
        }

        assert_eq!(checked, 5 * 9 * (1 + 5 * 10), "every token the format defines");
    }

    /// The digit row is live whichever way round the author works.
    #[test]
    fn a_group_chosen_before_a_trigger_is_the_group_the_trigger_joins() {
        let first = brush_of(&[
            PaletteEntry::Group(7),
            PaletteEntry::Trigger(Some(TriggerType::Stop)),
        ]);

        let second = brush_of(&[
            PaletteEntry::Trigger(Some(TriggerType::Stop)),
            PaletteEntry::Group(7),
        ]);

        assert_eq!(first.trigger, Some((TriggerType::Stop, 7)));
        assert_eq!(first, second, "the two orders have to mean the same brush");
    }

    /// Erasing is painting the format's own `.`, which is why it is the last
    /// swatch of the block row rather than a switch beside the palette.
    #[test]
    fn the_last_swatch_of_the_block_row_is_the_erase_brush() {
        let erasing = brush_of(&[PaletteEntry::Block(None)]);

        assert!(erasing.erase);
        assert_eq!(erasing.token(), EMPTY_SLOT);

        // And choosing a block is how it is put down again - there is no
        // separate way back.
        let painting = brush_of(&[
            PaletteEntry::Block(None),
            PaletteEntry::Block(Some(BlockType::Concrete)),
        ]);

        assert!(!painting.erase);
        assert_eq!(painting.token(), "CA");
    }

    /// A row sets its own character and no other. Chosen the hard way: start
    /// from a brush with something in every field, change one thing, and check
    /// the rest of the token did not move.
    #[test]
    fn an_entry_sets_only_its_own_part_of_the_brush() {
        let start = vec![
            PaletteEntry::Block(Some(BlockType::Hardling)),
            PaletteEntry::Behaviour(BlockBehaviour::Portal),
            PaletteEntry::Group(4),
            PaletteEntry::Trigger(Some(TriggerType::StartStop)),
        ];

        assert_eq!(brush_of(&start).token(), "BIC4");

        for (entry, expected) in [
            (PaletteEntry::Block(Some(BlockType::Obstacle)), "ZIC4"),
            (PaletteEntry::Behaviour(BlockBehaviour::Vanisher), "BCC4"),
            (PaletteEntry::Trigger(Some(TriggerType::Start)), "BIA4"),
            (PaletteEntry::Trigger(None), "BI"),
            (PaletteEntry::Group(9), "BIC9"),
        ] {
            let mut chosen = start.clone();
            chosen.push(entry.clone());

            assert_eq!(brush_of(&chosen).token(), expected, "after choosing {entry:?}");
        }
    }


    // --- what is shown as chosen ------------------------------------------

    /// The card's last criterion, asked of the outline: exactly one entry of
    /// each row is the one the brush is set to, for every brush the palette can
    /// make.
    #[test]
    fn exactly_one_entry_of_each_row_is_chosen() {
        for block in [Some(BlockType::SimpleTop), None] {
            for group in [0, 5] {
                for trigger in [None, Some(TriggerType::ReceiverStartingActive)] {
                    let mut brush = Brush::default();
                    let mut chosen_group = BrushGroup::default();

                    for entry in [
                        PaletteEntry::Block(block.clone()),
                        PaletteEntry::Behaviour(BlockBehaviour::EvaderD(EVADER_SPEED)),
                        PaletteEntry::Group(group),
                        PaletteEntry::Trigger(trigger.clone()),
                    ] {
                        entry.apply(&mut brush, &mut chosen_group);
                    }

                    for (name, of) in [
                        ("block", is_block as fn(&PaletteEntry) -> bool),
                        ("behaviour", is_behaviour),
                        ("trigger", is_trigger),
                        ("group", is_group),
                    ] {
                        let chosen: Vec<PaletteEntry> = entries()
                            .into_iter()
                            .filter(|entry| of(entry) && entry.is_chosen(&brush, &chosen_group))
                            .collect();

                        assert_eq!(
                            chosen.len(),
                            1,
                            "the {name} row of {} has {chosen:?} outlined",
                            brush.token()
                        );
                    }
                }
            }
        }
    }

    /// The format has no room for an evader's speed, so a level built in code
    /// can hand the editor one the palette cannot offer. Its row still has to
    /// say which letter is being painted, because that letter is what the file
    /// would get.
    #[test]
    fn an_evader_at_a_speed_the_format_cannot_hold_still_shows_its_own_row() {
        let brush = Brush {
            behaviour: BlockBehaviour::EvaderL(12.5),
            ..Brush::default()
        };

        let chosen: Vec<char> = entries()
            .into_iter()
            .filter(|entry| is_behaviour(entry) && entry.is_chosen(&brush, &BrushGroup::default()))
            .map(|entry| entry.letter())
            .collect();

        assert_eq!(chosen, ['F'], "{} is painted as an F", brush.token());
    }


    // --- where it sits ----------------------------------------------------

    /// The rectangle an entry is drawn at is the rectangle a click on it is
    /// read against - which is the whole reason the palette lays itself out.
    #[test]
    fn every_entry_is_found_where_it_is_drawn() {
        for item in palette_items(WINDOW) {
            let PaletteItemKind::Entry(entry) = item.kind else { continue; };

            assert_eq!(
                palette_entry_at(item.rect.center(), WINDOW),
                Some(entry.clone()),
                "{entry:?} is drawn at {:?} and not found there",
                item.rect
            );
        }
    }

    /// Every entry the keyboard can reach is also on screen to be clicked, and
    /// the other way round: two lists that disagreed would be a shortcut with no
    /// swatch, or a swatch with no shortcut.
    #[test]
    fn what_is_drawn_is_what_can_be_typed() {
        let drawn: Vec<PaletteEntry> = palette_items(WINDOW)
            .into_iter()
            .filter_map(|item| match item.kind {
                PaletteItemKind::Entry(entry) => Some(entry),
                _ => None,
            })
            .collect();

        assert_eq!(drawn, entries());
    }

    /// The panel is taken off what is in it, so nothing can hang outside the
    /// area the pointer treats as the palette.
    #[test]
    fn the_panel_covers_everything_it_lays_out() {
        let panel = palette_rect(WINDOW);

        for item in palette_items(WINDOW) {
            assert!(panel.contains(item.rect.min), "{:?} starts outside the panel", item.kind);
            assert!(panel.contains(item.rect.max), "{:?} ends outside the panel", item.kind);
        }

        assert!(
            palette_entry_at(panel.min - Vec2::splat(1.0), WINDOW).is_none(),
            "a pixel above and left of the panel is on nothing"
        );
        assert!(
            palette_entry_at(panel.max + Vec2::splat(1.0), WINDOW).is_none(),
            "and so is one below and right of it"
        );
    }

    /// Whatever the window does, the palette stays clear of the column down the
    /// left. A window with no room for both has to give something up, and what
    /// it gives up is the palette's right-hand end running off the edge - not
    /// two panels lying over each other, where a click would land on both.
    #[test]
    fn the_palette_never_climbs_onto_the_column_down_the_left() {
        let column = panel_rect().max.x;

        for width in [0.0, 100.0, 400.0, 700.0, 720.0, 800.0, 1600.0, 3000.0] {
            let palette = palette_rect(width);

            assert!(
                palette.min.x >= column,
                "on a {width}px window the palette starts at {} where the column down the left ends at {column}",
                palette.min.x
            );
        }
    }

    /// The palette is the one panel that has to know how wide the window is,
    /// because it is the one anchored to the far side of it.
    #[test]
    fn the_palette_hangs_off_the_right_edge_of_the_window() {
        for width in [800.0, 1600.0, 2560.0] {
            let panel = palette_rect(width);

            // Within a hundredth of a pixel rather than exactly: the panel's
            // width is taken off what is in it, and a row of six swatches
            // sharing 324 pixels does not divide evenly in binary.
            let margin = width - panel.max.x;
            assert!(
                (margin - PANEL_ORIGIN.x).abs() < 0.01,
                "the palette sits {margin}px from the right edge of a {width}px window"
            );

            assert_eq!(panel.min.y, PANEL_ORIGIN.y, "and level with the top of the other column");
            assert!(
                (panel.width() - palette_rect(WINDOW).width()).abs() < 0.01,
                "the same width whatever the window: {} against {}",
                panel.width(),
                palette_rect(WINDOW).width()
            );
        }
    }
}
