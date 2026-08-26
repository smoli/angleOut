//! The level settings panel: everything about a level that is not its grid.
//!
//! A [`LevelDefinition`] is a token grid plus ten other fields, and the grid is
//! only half a level - a background, how many balls are in play at once, what
//! counts as winning and which pickups the level hands out are all authored
//! here rather than in the map.
//!
//! Every setting is a *stepper*: a label, the value as it stands, and a `<` and
//! a `>` that walk it. There is no text entry in this game to type a number
//! into, and a stepper needs none - it also means a setting has no invalid state
//! to be in, because the step itself is what keeps the value in range. What a
//! level file already holds can still be anything at all, so every step snaps
//! its value back onto the ladder rather than trusting it (see [`stepped`]).
//!
//! The panel hit-tests itself against the rectangles it lays out, rather than
//! going through `Interaction`: it is the same trade the editor already makes
//! for the grid - [`world_to_cell`](crate::level::layout::world_to_cell) rather
//! than mesh picking - and it keeps what is on screen and what is clickable the
//! same rectangle, since [`settings_rows`] is what draws *and* what reads a
//! click.

use std::time::Duration;

use bevy::asset::AssetServer;
use bevy::color::palettes::css::{GOLD, SILVER, WHITE};
use bevy::prelude::{default, Bundle, Color, Commands, Component, Node, PositionType, Rect, Res, Text, TextColor, TextFont, UiRect, Val, Vec2};
use bevy::text::{FontSize, Justify, TextLayout};
use bevy::ui::{BackgroundColor, GlobalZIndex};

use crate::level::{LevelDefinition, WinCriteria};
use crate::pickups::PickupType;

use super::{EditorEntity, EDITOR_FONT};

/// The backgrounds a level can be given.
///
/// A named list rather than every scene in `ship3_003.glb`: the same file holds
/// the ball, the paddle and the pickups, and none of those is a backdrop. The
/// alternative is free text, and there is nowhere in the game to type it.
pub const BACKGROUNDS: &[&str] = &[
    "ship3_003.glb#Scene10",
    "ship3_003.glb#Scene11",
    "ship3_003.glb#Scene12",
    "ship3_003.glb#Scene13",
];

/// How fast the backdrop scrolls, in steps and at the outside.
///
/// Never backwards: `arena_update_background` only wraps a segment that has run
/// off the near end, so a negative velocity sends the backdrop away and never
/// brings it back.
const SCROLL_STEP: f32 = 5.0;
const SCROLL_MAX: f32 = 100.0;

/// At least one ball, or the level can never be launched -
/// `MatchEvent::BallSpawn` only ever adds one while `balls_in_play` is below
/// this. Eight is as many as anything downstream is built for: a force field
/// panel shows [`FORCE_FIELD_HIT_SLOTS`](crate::materials::force_field::FORCE_FIELD_HIT_SLOTS)
/// impacts at once.
const MIN_SIMULTANEOUS_BALLS: i32 = 1;
const MAX_SIMULTANEOUS_BALLS: i32 = 8;

/// The win criterion is a *fraction* of the blocks hit rather than lost, so the
/// whole range is meaningful: 100% is "lose none of them", 0% is "clear the
/// level, however badly".
const WIN_STEP: f32 = 0.05;

/// The time limit walks in half minutes up to ten. Below the first rung is
/// `None`, which is what every level ships with.
const TIME_STEP: u64 = 30;
const TIME_MAX: u64 = 600;

/// As many pickups as a level can hand out. A level has a few dozen blocks and
/// one pickup lands per block, so twenty is already more than generous.
const MAX_PICKUPS: usize = 20;

/// One thing about the level that the panel can change.
///
/// Everything in [`LevelDefinition`] except the `targets` the grid editor owns
/// and the free-floating `obstacles`, which are out of scope for this epic.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Setting {
    Background,
    ScrollVelocity,
    SimultaneousBalls,
    WinPercentage,
    TimeLimit,
    ExtraBalls,
    Grabbers,
    WallLeft,
    WallRight,
}

/// The settings, in the order they are on screen.
pub const SETTINGS: [Setting; 9] = [
    Setting::Background,
    Setting::ScrollVelocity,
    Setting::SimultaneousBalls,
    Setting::WinPercentage,
    Setting::TimeLimit,
    Setting::ExtraBalls,
    Setting::Grabbers,
    Setting::WallLeft,
    Setting::WallRight,
];

impl Setting {
    /// What the setting is called on screen.
    pub fn label(&self) -> &'static str {
        match self {
            Setting::Background => "Background",
            Setting::ScrollVelocity => "Scroll speed",
            Setting::SimultaneousBalls => "Balls at once",
            Setting::WinPercentage => "Win at",
            Setting::TimeLimit => "Time limit",
            Setting::ExtraBalls => "Extra balls",
            Setting::Grabbers => "Grabbers",
            Setting::WallLeft => "Left wall",
            Setting::WallRight => "Right wall",
        }
    }

    /// The value as it stands, as the panel writes it.
    ///
    /// Whatever the level holds, including something no step would ever have
    /// produced - a level file is hand-editable, and a panel that cannot show
    /// what is in one is no use for fixing it.
    pub fn value(&self, level: &LevelDefinition) -> String {
        match self {
            Setting::Background => background_name(&level.background_asset).to_string(),
            Setting::ScrollVelocity => number(level.background_scroll_velocity),
            Setting::SimultaneousBalls => level.simultaneous_balls.to_string(),

            Setting::WinPercentage => match level.win_criteria {
                WinCriteria::BlockHitPercentage(pct) => percentage(pct),
            },

            Setting::TimeLimit => match level.time_limit {
                None => "off".to_string(),
                Some(limit) => clock(limit),
            },

            Setting::ExtraBalls => pickup_count(level, PickupKind::MoreBalls).to_string(),
            Setting::Grabbers => pickup_count(level, PickupKind::Grabber).to_string(),
            Setting::WallLeft => on_off(level.default_wall_l),
            Setting::WallRight => on_off(level.default_wall_r),
        }
    }

    /// Steps the setting `by` rungs - `-1` for the `<` button, `1` for the `>`
    /// - and says whether that changed the level at all.
    ///
    /// The answer is what keeps change detection honest, as it is for painting a
    /// cell: a click at the end of a setting's range edits nothing, and a level
    /// that did not change must not look to the rest of the editor as though it
    /// did.
    pub fn step(&self, level: &mut LevelDefinition, by: i32) -> bool {
        match self {
            Setting::Background => {
                let background = next_background(&level.background_asset, by);

                replace(&mut level.background_asset, background)
            }

            Setting::ScrollVelocity => {
                let velocity =
                    stepped(level.background_scroll_velocity, by, SCROLL_STEP, 0.0, SCROLL_MAX);

                replace(&mut level.background_scroll_velocity, velocity)
            }

            Setting::SimultaneousBalls => {
                let balls = level
                    .simultaneous_balls
                    .saturating_add(by)
                    .clamp(MIN_SIMULTANEOUS_BALLS, MAX_SIMULTANEOUS_BALLS);

                replace(&mut level.simultaneous_balls, balls)
            }

            Setting::WinPercentage => {
                let WinCriteria::BlockHitPercentage(pct) = level.win_criteria;
                let criteria = WinCriteria::BlockHitPercentage(stepped(pct, by, WIN_STEP, 0.0, 1.0));

                replace(&mut level.win_criteria, criteria)
            }

            Setting::TimeLimit => {
                let limit = next_time_limit(level.time_limit, by);

                replace(&mut level.time_limit, limit)
            }

            Setting::ExtraBalls => step_pickups(level, PickupKind::MoreBalls, by),
            Setting::Grabbers => step_pickups(level, PickupKind::Grabber, by),

            // A switch is set rather than walked: `>` turns the wall on and `<`
            // turns it off, so a second click on the same button is a click that
            // changes nothing rather than one that undoes the first.
            Setting::WallLeft => replace(&mut level.default_wall_l, by > 0),
            Setting::WallRight => replace(&mut level.default_wall_r, by > 0),
        }
    }
}

/// Writes `value` into `slot`, and says whether that was a change.
fn replace<T: PartialEq>(slot: &mut T, value: T) -> bool {
    if *slot == value {
        return false;
    }

    *slot = value;
    true
}

/// The next value along a ladder of `step`s, `by` rungs from where `current` is.
///
/// `current` is snapped onto the ladder before it is stepped, so a hand-written
/// 17 goes to 20 rather than to 22, and anything that is not a number at all -
/// a `NaN` in a level file - starts from the bottom of the range instead of
/// staying `NaN` forever.
fn stepped(current: f32, by: i32, step: f32, min: f32, max: f32) -> f32 {
    let current = if current.is_finite() { current.clamp(min, max) } else { min };
    let rung = (current / step).round() + by as f32;

    (rung * step).clamp(min, max)
}

/// The background `by` entries along from the one the level is on.
///
/// A level whose background is not one of [`BACKGROUNDS`] - hand-written, or
/// from a glTF this build no longer has - joins the list at the end it was
/// stepped from, rather than jumping into the middle of it.
fn next_background(current: &str, by: i32) -> String {
    let count = BACKGROUNDS.len() as i32;

    let next = match BACKGROUNDS.iter().position(|background| *background == current) {
        Some(at) => (at as i32 + by).rem_euclid(count),
        None if by >= 0 => 0,
        None => count - 1,
    };

    BACKGROUNDS[next as usize].to_string()
}

/// The time limit `by` half-minutes along from the one the level is on, where
/// the rung below the first is "no limit at all".
fn next_time_limit(current: Option<Duration>, by: i32) -> Option<Duration> {
    let rung = match current {
        Some(limit) => (limit.as_secs() as f64 / TIME_STEP as f64).round() as i64 + by as i64,
        None if by > 0 => 1,
        None => return None,
    };

    (rung > 0).then(|| Duration::from_secs((rung as u64 * TIME_STEP).min(TIME_MAX)))
}

/// The two kinds of global pickup, as the panel counts them.
///
/// `Grabber` is here for the reason the card gives: nothing in the game
/// constructs one today, so the panel is where a level first gets to hand one
/// out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PickupKind {
    MoreBalls,
    Grabber,
}

impl PickupKind {
    fn matches(&self, pickup: &PickupType) -> bool {
        matches!(
            (self, pickup),
            (PickupKind::MoreBalls, PickupType::MoreBalls(_)) | (PickupKind::Grabber, PickupType::Grabber(_))
        )
    }

    /// One pickup of this kind, as the panel hands them out.
    ///
    /// One of the thing: the amount a pickup carries is not read anywhere in the
    /// game today, and the panel counts pickups rather than amounts - so a level
    /// that already holds a `MoreBalls(3)` keeps it, and only ever gains and
    /// loses whole pickups either side of it.
    fn one(&self) -> PickupType {
        match self {
            PickupKind::MoreBalls => PickupType::MoreBalls(1),
            PickupKind::Grabber => PickupType::Grabber(1),
        }
    }
}

fn pickup_count(level: &LevelDefinition, kind: PickupKind) -> usize {
    level.global_pickups.iter().filter(|pickup| kind.matches(pickup)).count()
}

/// Adds or takes away pickups of one kind until the level holds `by` more of
/// them than it did. Says whether it had to.
fn step_pickups(level: &mut LevelDefinition, kind: PickupKind, by: i32) -> bool {
    let have = pickup_count(level, kind) as i32;
    let want = (have + by).clamp(0, MAX_PICKUPS as i32) as usize;

    if want == have as usize {
        return false;
    }

    while pickup_count(level, kind) > want {
        let last = level
            .global_pickups
            .iter()
            .rposition(|pickup| kind.matches(pickup))
            .expect("there are more of them in there than are wanted");

        level.global_pickups.remove(last);
    }

    while pickup_count(level, kind) < want {
        level.global_pickups.push(kind.one());
    }

    true
}

/// The readable half of a background asset path: the scene, without the glTF it
/// lives in.
fn background_name(asset: &str) -> &str {
    asset.rsplit('#').next().unwrap_or(asset)
}

/// A number as an author reads it, rather than as Rust prints it: `20`, not
/// `20.0`, and one decimal for anything a level file put between the rungs.
fn number(value: f32) -> String {
    if !value.is_finite() {
        return value.to_string();
    }

    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    }
}

fn percentage(fraction: f32) -> String {
    format!("{}%", number(fraction * 100.0))
}

fn clock(limit: Duration) -> String {
    let seconds = limit.as_secs();

    format!("{}:{:02}", seconds / 60, seconds % 60)
}

fn on_off(on: bool) -> String {
    if on { "on" } else { "off" }.to_string()
}


// --- layout ---------------------------------------------------------------

/// Where the panel sits, in logical window pixels - the same space
/// `Window::cursor_position` reads in, which is what lets a click be tested
/// against it directly.
const PANEL_ORIGIN: Vec2 = Vec2::new(16.0, 16.0);
const PANEL_PADDING: f32 = 8.0;

const TITLE_HEIGHT: f32 = 26.0;
const ROW_HEIGHT: f32 = 26.0;

/// How much of a row's height a button leaves alone, so two rows of buttons do
/// not run into one another.
const ROW_INSET: f32 = 2.0;

const COLUMN_GAP: f32 = 4.0;
const LABEL_WIDTH: f32 = 150.0;
const VALUE_WIDTH: f32 = 110.0;
const BUTTON_WIDTH: f32 = 26.0;

const ROW_WIDTH: f32 = LABEL_WIDTH + VALUE_WIDTH + 2.0 * BUTTON_WIDTH + 3.0 * COLUMN_GAP;

const TITLE_FONT_SIZE: f32 = 18.0;
const ROW_FONT_SIZE: f32 = 15.0;

const PANEL_BACKGROUND: Color = Color::srgba(0.04, 0.04, 0.07, 0.85);
const BUTTON_BACKGROUND: Color = Color::srgba(0.18, 0.18, 0.24, 0.95);

/// Marks everything the panel draws, so a changed level can take the whole of it
/// down and put it up again saying the new thing.
#[derive(Component)]
pub struct SettingsPanel;

/// A value on screen, tagged with the setting it is showing.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingValue(pub Setting);

/// A button on screen: the setting it steps and which way.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingButton {
    pub setting: Setting,
    pub by: i32,
}

/// One setting's row, as rectangles on the window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SettingRow {
    pub setting: Setting,
    pub label: Rect,
    pub down: Rect,
    pub value: Rect,
    pub up: Rect,
}

/// The whole panel's footprint.
///
/// What the editor keeps its hands off: a click in here is aimed at a setting,
/// and the cell that happens to be underneath is not what the author meant.
pub fn panel_rect() -> Rect {
    let height = PANEL_PADDING + TITLE_HEIGHT + SETTINGS.len() as f32 * ROW_HEIGHT + PANEL_PADDING;

    Rect::new(
        PANEL_ORIGIN.x,
        PANEL_ORIGIN.y,
        PANEL_ORIGIN.x + ROW_WIDTH + 2.0 * PANEL_PADDING,
        PANEL_ORIGIN.y + height,
    )
}

fn title_rect() -> Rect {
    let left = PANEL_ORIGIN.x + PANEL_PADDING;
    let top = PANEL_ORIGIN.y + PANEL_PADDING;

    Rect::new(left, top, left + ROW_WIDTH, top + TITLE_HEIGHT)
}

/// The rows of the panel, laid out once and used twice: this is what the panel
/// draws, and what a click is read against.
pub fn settings_rows() -> Vec<SettingRow> {
    SETTINGS
        .iter()
        .enumerate()
        .map(|(index, setting)| settings_row(*setting, index))
        .collect()
}

fn settings_row(setting: Setting, index: usize) -> SettingRow {
    let left = PANEL_ORIGIN.x + PANEL_PADDING;
    let top = PANEL_ORIGIN.y + PANEL_PADDING + TITLE_HEIGHT + index as f32 * ROW_HEIGHT;
    let bottom = top + ROW_HEIGHT;

    // The columns, left to right: what it is called, the button that steps it
    // down, what it says, the button that steps it up.
    let label = Rect::new(left, top, left + LABEL_WIDTH, bottom);
    let down = column(label.max.x + COLUMN_GAP, top, BUTTON_WIDTH);
    let value = Rect::new(down.max.x + COLUMN_GAP, top, down.max.x + COLUMN_GAP + VALUE_WIDTH, bottom);
    let up = column(value.max.x + COLUMN_GAP, top, BUTTON_WIDTH);

    SettingRow { setting, label, down, value, up }
}

/// A button-sized column of a row, kept clear of the rows above and below it.
fn column(left: f32, top: f32, width: f32) -> Rect {
    Rect::new(left, top + ROW_INSET, left + width, top + ROW_HEIGHT - ROW_INSET)
}

/// The setting a click at `pixel` is aimed at, and how far it steps it.
///
/// `None` for a click anywhere else, including inside the panel but not on a
/// button - the panel swallows those, it does not act on them.
pub fn setting_at(pixel: Vec2) -> Option<(Setting, i32)> {
    settings_rows().into_iter().find_map(|row| {
        if row.down.contains(pixel) {
            Some((row.setting, -1))
        } else if row.up.contains(pixel) {
            Some((row.setting, 1))
        } else {
            None
        }
    })
}


// --- drawing --------------------------------------------------------------

/// Puts the panel on screen, saying what the level says right now.
///
/// The whole panel is rebuilt rather than the one value that moved, as the block
/// grid is: ten nodes is nothing, and this only runs on a frame the level
/// actually changed.
pub fn spawn_settings_panel(
    level: &LevelDefinition,
    asset_server: &Res<AssetServer>,
    commands: &mut Commands,
) {
    commands.spawn((
        panel_node(panel_rect()),
        BackgroundColor(PANEL_BACKGROUND),
        GlobalZIndex(PANEL_Z),
        SettingsPanel,
        EditorEntity,
    ));

    commands.spawn((
        panel_text(title_rect(), "Level settings", TITLE_FONT_SIZE, GOLD.into(), Justify::Left, asset_server),
        SettingsPanel,
        EditorEntity,
    ));

    for row in settings_rows() {
        commands.spawn((
            panel_text(row.label, row.setting.label(), ROW_FONT_SIZE, SILVER.into(), Justify::Left, asset_server),
            SettingsPanel,
            EditorEntity,
        ));

        commands.spawn((
            panel_text(row.value, &row.setting.value(level), ROW_FONT_SIZE, WHITE.into(), Justify::Center, asset_server),
            SettingValue(row.setting),
            SettingsPanel,
            EditorEntity,
        ));

        for (rect, glyph, by) in [(row.down, "<", -1), (row.up, ">", 1)] {
            commands.spawn((
                panel_text(rect, glyph, ROW_FONT_SIZE, GOLD.into(), Justify::Center, asset_server),
                BackgroundColor(BUTTON_BACKGROUND),
                SettingButton { setting: row.setting, by },
                SettingsPanel,
                EditorEntity,
            ));
        }
    }
}

/// Above the panel's own background, and above the rest of the editor's UI.
const PANEL_Z: i32 = 10;
const PANEL_CONTENT_Z: i32 = 11;

/// A rectangle of the panel, where [`settings_rows`] put it.
fn panel_node(rect: Rect) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(rect.min.x),
        top: Val::Px(rect.min.y),
        width: Val::Px(rect.width()),
        height: Val::Px(rect.height()),
        ..default()
    }
}

/// A rectangle of the panel with something written in it, centred in its own
/// height so a row reads as a row.
fn panel_text(
    rect: Rect,
    text: &str,
    size: f32,
    colour: Color,
    justify: Justify,
    asset_server: &Res<AssetServer>,
) -> impl Bundle {
    // The glyphs are laid out from the top of the node, so the padding is what
    // puts them in the middle of it. A line box is about 1.2 times the font
    // size.
    let padding = ((rect.height() - size * 1.2) / 2.0).max(0.0);

    (
        Node { padding: UiRect::top(Val::Px(padding)), ..panel_node(rect) },
        Text::new(text),
        TextFont {
            font: asset_server.load(EDITOR_FONT).into(),
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(colour),
        TextLayout::justify(justify),
        GlobalZIndex(PANEL_CONTENT_Z),
    )
}


#[cfg(test)]
mod tests {
    use super::*;

    use bevy::prelude::default;

    use crate::level::campaign::{level_to_ron, parse_level};

    /// The setting stepped `times` times in the `by` direction, from `level`.
    fn step(level: &mut LevelDefinition, setting: Setting, by: i32, times: usize) {
        for _ in 0..times {
            setting.step(level, by);
        }
    }

    /// One click of a setting, from the level every level starts as.
    fn stepped_once(setting: Setting, by: i32) -> LevelDefinition {
        let mut level = LevelDefinition::default();
        setting.step(&mut level, by);

        level
    }

    /// The fields of [`LevelDefinition`] that differ between two levels, by
    /// name - which is how "a setting changes its own field and nothing else"
    /// is asked.
    fn differences(a: &LevelDefinition, b: &LevelDefinition) -> Vec<&'static str> {
        let mut differ = vec![];

        for (name, same) in [
            ("background_asset", a.background_asset == b.background_asset),
            ("background_scroll_velocity", a.background_scroll_velocity == b.background_scroll_velocity),
            ("simultaneous_balls", a.simultaneous_balls == b.simultaneous_balls),
            ("win_criteria", a.win_criteria == b.win_criteria),
            ("targets", a.targets == b.targets),
            ("time_limit", a.time_limit == b.time_limit),
            ("global_pickups", a.global_pickups == b.global_pickups),
            ("obstacles", a.obstacles == b.obstacles),
            ("default_wall_l", a.default_wall_l == b.default_wall_l),
            ("default_wall_r", a.default_wall_r == b.default_wall_r),
        ] {
            if !same {
                differ.push(name);
            }
        }

        differ
    }

    /// The field each setting is about. Anything else it moved is a bug the
    /// author would find much later, in a level that quietly changed under them.
    fn field_of(setting: Setting) -> &'static str {
        match setting {
            Setting::Background => "background_asset",
            Setting::ScrollVelocity => "background_scroll_velocity",
            Setting::SimultaneousBalls => "simultaneous_balls",
            Setting::WinPercentage => "win_criteria",
            Setting::TimeLimit => "time_limit",
            Setting::ExtraBalls | Setting::Grabbers => "global_pickups",
            Setting::WallLeft => "default_wall_l",
            Setting::WallRight => "default_wall_r",
        }
    }

    /// Every setting is a way of changing exactly one thing about the level -
    /// and, in particular, never the grid, which is the other half of the
    /// editor.
    #[test]
    fn a_setting_changes_its_own_field_and_nothing_else() {
        for setting in SETTINGS {
            // Whichever way it moves from the default - `default_wall_l` is
            // already on, so only `<` changes it.
            for by in [-1, 1] {
                let level = stepped_once(setting, by);
                let changed = differences(&LevelDefinition::default(), &level);

                assert!(
                    changed.is_empty() || changed == vec![field_of(setting)],
                    "{setting:?} stepped by {by} changed {changed:?}"
                );
            }
        }
    }

    /// Every setting has to be reachable from the default level, or the panel
    /// has a row that does nothing.
    #[test]
    fn every_setting_can_be_changed_from_where_a_new_level_starts() {
        for setting in SETTINGS {
            assert!(
                [-1, 1].iter().any(|by| setting.step(&mut LevelDefinition::default(), *by)),
                "{setting:?} cannot be moved off its default at all"
            );
        }
    }

    /// The stepper is what keeps a value in range, so walking it into the wall
    /// has to stop at the wall rather than run past it.
    #[test]
    fn stepping_a_setting_stops_at_the_ends_of_its_range() {
        let mut level = LevelDefinition::default();

        for setting in SETTINGS {
            step(&mut level, setting, 1, 200);
        }

        assert_eq!(level.background_scroll_velocity, SCROLL_MAX);
        assert_eq!(level.simultaneous_balls, MAX_SIMULTANEOUS_BALLS);
        assert_eq!(level.win_criteria, WinCriteria::BlockHitPercentage(1.0));
        assert_eq!(level.time_limit, Some(Duration::from_secs(TIME_MAX)));
        assert_eq!(level.global_pickups.len(), 2 * MAX_PICKUPS);
        assert!(level.default_wall_l && level.default_wall_r);

        for setting in SETTINGS {
            step(&mut level, setting, -1, 200);
        }

        assert_eq!(level.background_scroll_velocity, 0.0);
        assert_eq!(level.simultaneous_balls, MIN_SIMULTANEOUS_BALLS);
        assert_eq!(level.win_criteria, WinCriteria::BlockHitPercentage(0.0));
        assert_eq!(level.time_limit, None, "below the shortest limit is no limit at all");
        assert_eq!(level.global_pickups, vec![]);
        assert!(!level.default_wall_l && !level.default_wall_r);
    }

    /// A click that cannot move a setting any further is a click that edits
    /// nothing - which is what the editor's change detection hangs off.
    #[test]
    fn a_step_that_changes_nothing_says_so() {
        let mut level = LevelDefinition::default();

        for setting in SETTINGS {
            // The background is the one setting with no bottom: it is a ring of
            // the backdrops the game has, and stepping it always lands on
            // another one.
            if setting == Setting::Background {
                continue;
            }

            step(&mut level, setting, -1, 200);

            assert!(
                !setting.step(&mut level, -1),
                "{setting:?} is at the bottom of its range and still reports a change"
            );
        }
    }

    #[test]
    fn the_background_walks_the_ones_the_game_has_and_wraps_round() {
        let mut level = LevelDefinition::default();
        assert_eq!(level.background_asset, BACKGROUNDS[0], "the default is the first of them");

        for background in BACKGROUNDS.iter().skip(1) {
            Setting::Background.step(&mut level, 1);
            assert_eq!(&level.background_asset, background);
        }

        Setting::Background.step(&mut level, 1);
        assert_eq!(level.background_asset, BACKGROUNDS[0], "past the last one is the first one");

        Setting::Background.step(&mut level, -1);
        assert_eq!(level.background_asset, *BACKGROUNDS.last().unwrap());
    }

    /// A level file can name a background the panel has never heard of. Showing
    /// it is the point - and stepping it joins the list at the end it was
    /// stepped from rather than jumping into the middle.
    #[test]
    fn a_background_the_panel_does_not_know_is_shown_and_can_be_stepped_away_from() {
        let unknown = LevelDefinition {
            background_asset: "somewhere_else.glb#Backdrop".to_string(),
            ..default()
        };

        assert_eq!(Setting::Background.value(&unknown), "Backdrop");

        let mut level = unknown.clone();
        Setting::Background.step(&mut level, 1);
        assert_eq!(level.background_asset, BACKGROUNDS[0]);

        let mut level = unknown;
        Setting::Background.step(&mut level, -1);
        assert_eq!(level.background_asset, *BACKGROUNDS.last().unwrap());
    }

    /// The card's "invalid or out-of-range input is rejected without crashing":
    /// a level file is hand-editable, so the panel has to show whatever is in
    /// one and put it back in range on the first click rather than carry it
    /// along.
    #[test]
    fn values_no_step_would_ever_have_made_are_shown_and_stepped_back_into_range() {
        let mut level = LevelDefinition {
            background_scroll_velocity: f32::NAN,
            simultaneous_balls: -5,
            win_criteria: WinCriteria::BlockHitPercentage(5.0),
            time_limit: Some(Duration::from_secs(7)),
            ..default()
        };

        // Nothing here panics, and the panel says what the file says.
        assert_eq!(Setting::SimultaneousBalls.value(&level), "-5");
        assert_eq!(Setting::WinPercentage.value(&level), "500%");
        assert_eq!(Setting::TimeLimit.value(&level), "0:07");

        for setting in SETTINGS {
            setting.step(&mut level, 1);
        }

        assert_eq!(level.background_scroll_velocity, SCROLL_STEP);
        assert_eq!(level.simultaneous_balls, MIN_SIMULTANEOUS_BALLS);
        assert_eq!(level.win_criteria, WinCriteria::BlockHitPercentage(1.0));
        assert_eq!(level.time_limit, Some(Duration::from_secs(TIME_STEP)));
    }

    /// A percentage above 100 cannot go any higher, and comes back down to a
    /// value that is on the ladder.
    #[test]
    fn an_out_of_range_percentage_comes_back_down_onto_the_ladder() {
        let mut level = LevelDefinition {
            win_criteria: WinCriteria::BlockHitPercentage(5.0),
            ..default()
        };

        Setting::WinPercentage.step(&mut level, -1);

        assert_eq!(level.win_criteria, WinCriteria::BlockHitPercentage(1.0 - WIN_STEP));
    }

    /// The card's reason for the pickup rows: nothing in the game constructs a
    /// `Grabber` today, and the panel is where a level first hands one out.
    #[test]
    fn the_panel_is_where_a_grabber_becomes_reachable() {
        let mut level = LevelDefinition::default();

        Setting::Grabbers.step(&mut level, 1);

        assert_eq!(level.global_pickups, vec![PickupType::Grabber(1)]);
        assert_eq!(Setting::Grabbers.value(&level), "1");
        assert_eq!(Setting::ExtraBalls.value(&level), "0", "the two rows count different things");
    }

    /// The rows count pickups rather than the amounts they carry, so a level
    /// that authored a `MoreBalls(3)` by hand keeps it, and only ever gains and
    /// loses whole pickups either side of it.
    #[test]
    fn stepping_the_pickup_rows_leaves_the_amounts_a_level_authored_alone() {
        let mut level = LevelDefinition {
            global_pickups: vec![PickupType::MoreBalls(3), PickupType::Grabber(2)],
            ..default()
        };

        assert_eq!(Setting::ExtraBalls.value(&level), "1");

        Setting::ExtraBalls.step(&mut level, 1);
        assert_eq!(
            level.global_pickups,
            vec![PickupType::MoreBalls(3), PickupType::Grabber(2), PickupType::MoreBalls(1)]
        );

        Setting::ExtraBalls.step(&mut level, -1);
        assert_eq!(
            level.global_pickups,
            vec![PickupType::MoreBalls(3), PickupType::Grabber(2)],
            "the one the panel added is the one it takes back"
        );

        Setting::Grabbers.step(&mut level, -1);
        assert_eq!(level.global_pickups, vec![PickupType::MoreBalls(3)]);
    }

    /// The walls are switches rather than steppers: `>` turns one on, `<` turns
    /// it off, and clicking the same button twice is not an undo.
    #[test]
    fn a_wall_is_switched_rather_than_walked() {
        let mut level = LevelDefinition::default();

        assert!(!Setting::WallLeft.step(&mut level, 1), "it is already on");
        assert!(Setting::WallLeft.step(&mut level, -1));
        assert!(!level.default_wall_l);
        assert!(!Setting::WallLeft.step(&mut level, -1), "off twice is still off");
        assert!(Setting::WallLeft.step(&mut level, 1));
        assert!(level.default_wall_l);
    }

    #[test]
    fn a_setting_says_what_it_holds_the_way_an_author_reads_it() {
        let level = LevelDefinition {
            background_asset: "ship3_003.glb#Scene12".to_string(),
            background_scroll_velocity: 20.0,
            simultaneous_balls: 3,
            win_criteria: WinCriteria::BlockHitPercentage(0.85),
            time_limit: Some(Duration::from_secs(90)),
            global_pickups: vec![PickupType::MoreBalls(1), PickupType::MoreBalls(1)],
            default_wall_l: true,
            default_wall_r: false,
            ..default()
        };

        assert_eq!(Setting::Background.value(&level), "Scene12");
        assert_eq!(Setting::ScrollVelocity.value(&level), "20");
        assert_eq!(Setting::SimultaneousBalls.value(&level), "3");
        assert_eq!(Setting::WinPercentage.value(&level), "85%");
        assert_eq!(Setting::TimeLimit.value(&level), "1:30");
        assert_eq!(Setting::ExtraBalls.value(&level), "2");
        assert_eq!(Setting::Grabbers.value(&level), "0");
        assert_eq!(Setting::WallLeft.value(&level), "on");
        assert_eq!(Setting::WallRight.value(&level), "off");
        assert_eq!(Setting::TimeLimit.value(&LevelDefinition::default()), "off");
    }

    /// The card's round trip: whatever the panel can make of a level, the level
    /// file can say - and says the same thing when it is read back.
    #[test]
    fn every_edited_value_round_trips_through_ron() {
        let mut level = LevelDefinition::default();

        // Somewhere in the middle of every range, rather than at an end: a value
        // that survives by being the default proves nothing.
        for (setting, times) in [
            (Setting::Background, 2),
            (Setting::ScrollVelocity, 4),
            (Setting::SimultaneousBalls, 3),
            (Setting::TimeLimit, 5),
            (Setting::ExtraBalls, 3),
            (Setting::Grabbers, 2),
        ] {
            step(&mut level, setting, 1, times);
        }

        step(&mut level, Setting::WinPercentage, -1, 3);
        Setting::WallLeft.step(&mut level, -1);
        Setting::WallRight.step(&mut level, -1);

        assert_eq!(
            differences(&LevelDefinition::default(), &level).len(),
            8,
            "every field the panel owns has to have moved, or the round trip is not being asked"
        );

        let written = level_to_ron(&level).expect("a level the panel made has to be writable");
        let read_back = parse_level(&written).unwrap_or_else(|e| panic!("{e}\n{written}"));

        assert_eq!(read_back, level, "written as:\n{written}");
    }

    /// Every value the ladders can produce, not just the one the test above
    /// happens to walk to: a float that does not survive the file is a level
    /// that quietly changes the next time it is opened.
    #[test]
    fn every_rung_of_every_ladder_round_trips_through_ron() {
        let mut level = LevelDefinition::default();

        for _ in 0..(SCROLL_MAX / SCROLL_STEP) as usize + 1 {
            Setting::ScrollVelocity.step(&mut level, 1);
            Setting::WinPercentage.step(&mut level, -1);
            Setting::TimeLimit.step(&mut level, 1);

            let written = level_to_ron(&level).unwrap();
            assert_eq!(parse_level(&written).unwrap(), level, "written as:\n{written}");
        }
    }


    // --- layout -----------------------------------------------------------

    fn centre(rect: Rect) -> Vec2 {
        rect.center()
    }

    /// What is drawn is what is clicked: the middle of every button the panel
    /// lays out has to read back as that button.
    #[test]
    fn every_button_the_panel_lays_out_hit_tests_back_to_itself() {
        for row in settings_rows() {
            assert_eq!(setting_at(centre(row.down)), Some((row.setting, -1)));
            assert_eq!(setting_at(centre(row.up)), Some((row.setting, 1)));
        }

        assert_eq!(settings_rows().len(), SETTINGS.len(), "every setting has a row");
    }

    /// A click on the panel that is not on a button is swallowed rather than
    /// acted on - and one outside the panel is not the panel's at all.
    #[test]
    fn a_click_that_is_not_on_a_button_steps_nothing() {
        for row in settings_rows() {
            assert_eq!(setting_at(centre(row.label)), None);
            assert_eq!(setting_at(centre(row.value)), None);
        }

        assert_eq!(setting_at(panel_rect().max + Vec2::splat(1.0)), None);
        assert_eq!(setting_at(panel_rect().min - Vec2::splat(1.0)), None);
    }

    /// The panel is what the editor keeps its hands off, so everything it draws
    /// has to be inside it.
    #[test]
    fn the_panel_covers_everything_it_lays_out() {
        let panel = panel_rect();

        for row in settings_rows() {
            for rect in [row.label, row.down, row.value, row.up] {
                assert!(panel.contains(rect.min) && panel.contains(rect.max), "{rect:?} sticks out of {panel:?}");
            }
        }

        assert!(panel.contains(title_rect().min) && panel.contains(title_rect().max));
    }

    /// Two rows that overlap are two settings one click would step.
    #[test]
    fn no_two_buttons_overlap() {
        let buttons: Vec<(Setting, i32, Rect)> = settings_rows()
            .into_iter()
            .flat_map(|row| [(row.setting, -1, row.down), (row.setting, 1, row.up)])
            .collect();

        for (a, (setting, by, rect)) in buttons.iter().enumerate() {
            for (other_setting, other_by, other) in buttons.iter().skip(a + 1) {
                let overlap = rect.min.x < other.max.x
                    && other.min.x < rect.max.x
                    && rect.min.y < other.max.y
                    && other.min.y < rect.max.y;

                assert!(
                    !overlap,
                    "{setting:?}/{by} at {rect:?} overlaps {other_setting:?}/{other_by} at {other:?}"
                );
            }
        }
    }
}
