//! Which level the editor is working on.
//!
//! Until this panel there was exactly one answer, decided before the author
//! arrived: [`editor_open`](super::editor_open) reads the level the campaign is
//! pointing at, once, and nothing afterwards can put a different one in front of
//! the editor. Editing `level4.ron` meant making the campaign play `level4.ron`
//! first, and starting a level from nothing was not possible at all.
//!
//! So: the files in the directory the editor already saves into, named one at a
//! time with a stepper either side, an `Open` that puts the named one in front
//! of the editor and a `New` that starts a blank grid.
//!
//! **Opening replaces what is being edited, and does not ask.** It is the same
//! bargain the rest of the editor strikes - `c0012` saves a level it has just
//! complained about, because an author is the one who knows whether they meant
//! it. What the panel owes them instead is to say plainly what it is they are
//! editing, which is what the line under the buttons is for: it names the file,
//! all the time, not only in the moment something arrives.
//!
//! **The history goes with the level.** Every entry in it is a pair of levels
//! that belonged to the file that has just been closed, and an undo that walked
//! back into one of them would put another level's blocks on this one's grid -
//! the same reasoning `c0011` drops the history on a hand edit for.
//!
//! What is on screen is the fifth and last panel in the editor's left-hand
//! column, laid out and hit-tested against its own rectangles exactly as
//! `c0010`'s settings panel, `c0011`'s history bar and `c0012`'s file panel are.

use std::fs;
use std::path::Path;

use bevy::asset::AssetServer;
use bevy::ecs::system::SystemParam;
use bevy::color::palettes::css::{DIM_GRAY, GOLD, ORANGE_RED, SILVER, WHITE};
use bevy::ecs::change_detection::DetectChangesMut;
use bevy::log::{info, warn};
use bevy::prelude::{ButtonInput, Color, Commands, Component, Entity, KeyCode, MouseButton, Query, Rect, Res, ResMut, Resource, Vec2, With};
use bevy::text::Justify;
use bevy::ui::{BackgroundColor, GlobalZIndex};
use bevy::window::{PrimaryWindow, Window};

use crate::level::asset::LevelAsset;
use crate::level::campaign::{level_asset_path, load_level, CAMPAIGN_FILE};

use super::history::EditHistory;
use super::playtest::playtest_rect;
use super::save::{LastSave, LevelsOnDisk, SaveReport};
use super::settings::{panel_node, panel_text, BUTTON_BACKGROUND, COLUMN_GAP, PANEL_BACKGROUND, PANEL_ORIGIN, PANEL_PADDING, PANEL_Z, ROW_FONT_SIZE, ROW_HEIGHT, ROW_INSET, ROW_WIDTH, TITLE_FONT_SIZE, TITLE_HEIGHT};
use super::{commanding, finish_stroke, EditorEntity, EditorLevel, PaintStroke, PendingRemoval};


// --- what there is to open ------------------------------------------------

/// The level files the editor can open, and which of them the panel is naming.
///
/// The list is read off the directory rather than out of the campaign: a level
/// that is not in `campaign.ron` is scratch, not hidden, and scratch is exactly
/// what a level halfway through being authored is.
#[derive(Resource, Debug, Default, PartialEq, Eq)]
pub struct LevelChoice {
    /// Every level file in the directory, in a stable order.
    pub names: Vec<String>,

    /// Which of [`LevelChoice::names`] the chooser row is naming. Meaningless
    /// when there are none, and kept inside the list otherwise.
    pub chosen: usize,

    /// The file the last `Open` could not read, if that is what it came to.
    ///
    /// Cleared by anything that succeeds, so the panel says what went wrong
    /// exactly as long as it is still the last thing that happened.
    pub failure: Option<String>,
}

impl LevelChoice {
    /// The file the panel is naming right now.
    pub fn chosen_name(&self) -> Option<&str> {
        self.names.get(self.chosen).map(String::as_str)
    }

    /// Reads the directory, and points the chooser at the level under edit.
    ///
    /// What entering the editor does: the panel opens naming what the author is
    /// already working on, so the first thing they read is where they are rather
    /// than whichever file happens to sort first.
    pub fn refresh(&mut self, dir: &Path, editing: Option<&str>) {
        self.names = level_names(dir);
        self.failure = None;

        let editing = editing.map(file_name);
        self.chosen = editing
            .and_then(|name| self.names.iter().position(|listed| listed == name))
            .unwrap_or(0);
    }

    /// Reads the directory again, keeping the author's choice where it can.
    ///
    /// What a save does to the panel: a level that had never been on disk has a
    /// file now, and a chooser that could not offer it until the editor was left
    /// and entered again would be one that lies about what is there.
    fn relist(&mut self, listed: Vec<String>) {
        let named = self.chosen_name().map(str::to_string);

        self.names = listed;
        self.chosen = named
            .and_then(|name| self.names.iter().position(|listed| *listed == name))
            .unwrap_or_else(|| self.chosen.min(self.names.len().saturating_sub(1)));
    }

    /// Walks the list, wrapping at both ends.
    ///
    /// Wrapping because the list is short and the alternative is a stepper that
    /// stops without saying why - and because "the one before the first" is a
    /// question with an obvious answer here, where the range of a setting's
    /// value is a question with none.
    fn step(&mut self, by: i32) {
        let count = self.names.len();
        if count == 0 {
            return;
        }

        let by = by.rem_euclid(count as i32) as usize;
        self.chosen = (self.chosen + by) % count;
    }

    /// Where in the list the chooser is, as the panel puts it: "4 of 11".
    fn position(&self) -> String {
        match self.names.len() {
            0 => "none".to_string(),
            count => format!("{} of {count}", self.chosen + 1),
        }
    }
}

/// Every level file in `dir`, sorted, with the campaign index left out - it
/// lives in the same directory and is not a level.
///
/// A directory that cannot be read is an empty list rather than an error: the
/// panel has a way of saying it has nothing to offer, and that is the same thing
/// from where the author is standing.
fn level_names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .filter(|name| name.ends_with(".ron") && name != CAMPAIGN_FILE)
        .collect();

    names.sort();
    names
}

/// The file name of an asset path - what `levels/level4.ron` is called in the
/// directory the editor reads and writes.
fn file_name(asset_path: &str) -> &str {
    asset_path.rsplit('/').next().unwrap_or(asset_path)
}

/// The name as the chooser row writes it, without the extension every one of
/// them shares - four characters that say nothing about which level it is, on
/// the row where room is tightest.
///
/// `demo_minimal_win_state_error`, the longest name in `assets/levels`, measures
/// 272 pixels of Orbitron at [`ROW_FONT_SIZE`] against the row's
/// [`ROW_WIDTH`] - 52 to spare.
fn as_shown(name: &str) -> &str {
    name.strip_suffix(".ron").unwrap_or(name)
}


// --- doing it -------------------------------------------------------------

/// One of the four things the panel can be asked for.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChooseAction {
    /// The previous file in the list, and the next one - which choose nothing,
    /// they only change what the panel is naming. Stepping is free: nothing
    /// happens to the level under edit until `Open`.
    Back,
    Forward,

    /// Put the named file's level in front of the editor.
    Open,

    /// Start a level that belongs to no file.
    New,
}

impl ChooseAction {
    /// What the button is called on screen.
    pub fn label(&self) -> &'static str {
        match self {
            ChooseAction::Back => "<",
            ChooseAction::Forward => ">",
            ChooseAction::Open => "Open",
            ChooseAction::New => "New",
        }
    }

    /// Whether it is something the panel can do at all, said against the list as
    /// it stands.
    ///
    /// A dead action is drawn dimmed rather than left off, as the file panel
    /// draws a save that would go nowhere: a button that comes and goes is one an
    /// author has to look for.
    fn live(&self, choice: &LevelChoice) -> bool {
        match self {
            ChooseAction::New => true,
            _ => !choice.names.is_empty(),
        }
    }
}

/// Everything an arriving level displaces, in one hand.
///
/// The panel's two ways in - a button and a shortcut - both put a level in front
/// of the editor, and putting one there touches seven resources: the level
/// itself and the six that say something about the one it replaces. Bundled so
/// that the two systems, and the two functions under them, name them once.
#[derive(SystemParam)]
pub struct Editing<'w> {
    stroke: ResMut<'w, PaintStroke>,
    history: ResMut<'w, EditHistory>,
    pending: ResMut<'w, PendingRemoval>,
    last_save: ResMut<'w, LastSave>,
    report: ResMut<'w, SaveReport>,
    choice: ResMut<'w, LevelChoice>,
    level: ResMut<'w, EditorLevel>,
}

/// `Ctrl+O` and `Ctrl+N`, as every other program on the machine spells them.
pub fn editor_choose_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    dir: Res<LevelsOnDisk>,
    asset_server: Res<AssetServer>,
    mut editing: Editing,
) {
    if !commanding(&keys) {
        return;
    }

    let action = if keys.just_pressed(KeyCode::KeyO) {
        ChooseAction::Open
    } else if keys.just_pressed(KeyCode::KeyN) {
        ChooseAction::New
    } else {
        return;
    };

    take(action, &dir.0, &asset_server, &mut editing);
}

/// The same, plus the two steppers, from the panel.
///
/// The press rather than the hold, as the settings panel and the history bar
/// read their buttons: a stepper walked once per frame would run the whole
/// directory in a fifth of a second.
pub fn editor_choose_click(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    dir: Res<LevelsOnDisk>,
    asset_server: Res<AssetServer>,
    mut editing: Editing,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(cursor) = windows.iter().next().and_then(|window| window.cursor_position()) else { return; };
    let Some(action) = action_at(cursor, &editing.report) else { return; };

    take(action, &dir.0, &asset_server, &mut editing);
}

/// One thing the panel was asked for, however it was asked.
fn take(action: ChooseAction, dir: &Path, asset_server: &AssetServer, editing: &mut Editing) {
    match action {
        ChooseAction::Back => step(&mut editing.choice, -1),
        ChooseAction::Forward => step(&mut editing.choice, 1),

        ChooseAction::New => {
            arrive(EditorLevel::blank(), editing);
            info!("a blank level - the next save gives it a file");
        }

        ChooseAction::Open => {
            let Some(name) = editing.choice.chosen_name().map(str::to_string) else { return; };
            let path = dir.join(&name);

            let level = match load_level(&path) {
                Ok(level) => level,

                // The level under edit is left exactly as it was: a file that
                // will not read is a reason to say so, not a reason to take an
                // author's work away from them.
                Err(e) => {
                    warn!("could not open {}: {e}", path.display());
                    editing.choice.failure = Some(level_asset_path(&name));

                    return;
                }
            };

            let source = Some(asset_server.load::<LevelAsset>(level_asset_path(&name)));
            arrive(EditorLevel { source, level }, editing);

            info!("editing {}", level_asset_path(&name));
        }
    }
}

/// Steps the chooser without touching anything else - what the two steppers do.
///
/// Written around change detection, as painting is: a stepper pressed against a
/// directory with one file in it must not look to the panel like a choice that
/// moved.
fn step(choice: &mut ResMut<LevelChoice>, by: i32) {
    let was = choice.chosen;
    choice.bypass_change_detection().step(by);

    if choice.chosen != was {
        choice.set_changed();
    }
}

/// Puts a level in front of the editor, and clears away everything that was true
/// of the one it replaces.
///
/// The history because every entry in it belongs to the level that has just
/// gone; the stroke because a button still held over the grid was painting that
/// level and not this one; the warning because it counted blocks that are no
/// longer there; what was last written to disk because that was another file;
/// and the report because "Saved levels/level0.ron" said above a level that is
/// no longer `level0.ron` is a sentence that has stopped being true.
fn arrive(arriving: EditorLevel, editing: &mut Editing) {
    // Takes whatever the mouse was in the middle of painting off the table. It
    // goes into the history first and the history is cleared a line later, which
    // is the point: what it must not do is stay in hand and be recorded, when
    // the button finally comes up, against a level it was never painted on.
    finish_stroke(&mut editing.stroke, &mut editing.history, &editing.level.level);

    editing.history.clear();
    editing.history.set_changed();

    editing.pending.set_if_neq(PendingRemoval(None));
    editing.last_save.0 = None;
    editing.report.set_if_neq(SaveReport(None));

    editing.choice.failure = None;
    *editing.level = arriving;
}

/// Reads the directory again after a save, which is when a file can appear in
/// it that was not there when the editor opened.
pub fn editor_relist_levels(dir: Res<LevelsOnDisk>, mut choice: ResMut<LevelChoice>) {
    let listed = level_names(&dir.0);

    // Read before it is written, so a directory that has not moved leaves the
    // panel alone rather than being redrawn after every save.
    if listed == choice.names {
        return;
    }

    choice.relist(listed);
}

/// Reads the directory on the way into the editor, pointing the chooser at
/// whatever is being edited.
pub fn editor_list_levels(
    dir: Res<LevelsOnDisk>,
    editor_level: Res<EditorLevel>,
    mut choice: ResMut<LevelChoice>,
) {
    choice.refresh(&dir.0, editor_level.source_path().as_deref());
}


// --- the panel ------------------------------------------------------------

/// How far under the playtest panel this one sits.
const PANEL_GAP: f32 = 8.0;

/// How wide a stepper is - the settings panel's own, so the two read as the
/// same control.
const STEP_WIDTH: f32 = 26.0;

/// How wide the two buttons that do something are - the file panel's and the
/// playtest panel's, so the column reads as a column.
const ACTION_WIDTH: f32 = 84.0;

/// How many rows of the panel are buttons.
const BUTTON_ROWS: usize = 2;

/// How many rows the line under them is given.
///
/// Two, because it is a sentence where the rows above it are labels: "could not
/// read levels/demo_minimal_win_state_error.ron" does not fit the column at
/// [`ROW_FONT_SIZE`] and wraps onto a second line. The text node itself stays one
/// row tall - what is reserved here is the room under it for the wrap to land
/// in.
const MESSAGE_ROWS: usize = 2;

/// Marks everything the panel draws, so a changed choice can take the whole of
/// it down and put it up again saying the new thing.
#[derive(Component)]
pub struct LevelPanel;

/// The line under the buttons, which is the one thing on this panel a test reads
/// by name rather than by the action it belongs to.
#[derive(Component)]
pub struct ChoiceMessage;

/// The row under the name: everything the panel can be asked for, and how far
/// through the directory the chooser has got.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActionRow {
    pub back: Rect,
    pub forward: Rect,
    pub open: Rect,
    pub new: Rect,
    pub note: Rect,
}

/// The panel's footprint: under the playtest panel, in the same column and the
/// same width.
///
/// Takes the report because the file panel two above it grows downwards with
/// what the last save had to say, and everything under it moves with that.
///
/// What the editor keeps its hands off, as it does the four panels above it - a
/// click in here is aimed at a file, and the cell that happens to be underneath
/// is not what the author meant.
pub fn choose_rect(report: &SaveReport) -> Rect {
    let top = playtest_rect(report).max.y + PANEL_GAP;
    let rows = BUTTON_ROWS + MESSAGE_ROWS;
    let height = PANEL_PADDING + TITLE_HEIGHT + rows as f32 * ROW_HEIGHT + PANEL_PADDING;

    Rect::new(
        PANEL_ORIGIN.x,
        top,
        PANEL_ORIGIN.x + ROW_WIDTH + 2.0 * PANEL_PADDING,
        top + height,
    )
}

fn panel_title_rect(report: &SaveReport) -> Rect {
    let left = choose_rect(report).min.x + PANEL_PADDING;
    let top = choose_rect(report).min.y + PANEL_PADDING;

    Rect::new(left, top, left + ROW_WIDTH, top + TITLE_HEIGHT)
}

/// The name the chooser is pointing at, across the whole width of the panel.
///
/// The whole width because a file name is the longest thing on the panel and the
/// one an author has to read: the steppers that walk it sit under it rather than
/// either side of it, where they would take 60 pixels off exactly the row that
/// has none to give - see [`as_shown`].
pub fn name_rect(report: &SaveReport) -> Rect {
    let (left, top, bottom) = row_bounds(report, 0);

    Rect::new(left, top, left + ROW_WIDTH, bottom)
}

/// The row under it: the two steppers, the two buttons, and what is left over
/// saying how far through the directory the chooser has got.
pub fn action_row(report: &SaveReport) -> ActionRow {
    let (left, top, bottom) = row_bounds(report, 1);

    // Every button is kept clear of the rows above and below it, so two of them
    // do not run into one another.
    let button = |left: f32, width: f32| Rect::new(left, top + ROW_INSET, left + width, bottom - ROW_INSET);

    let back = button(left, STEP_WIDTH);
    let forward = button(back.max.x + COLUMN_GAP, STEP_WIDTH);
    let open = button(forward.max.x + COLUMN_GAP, ACTION_WIDTH);
    let new = button(open.max.x + COLUMN_GAP, ACTION_WIDTH);
    let note = Rect::new(new.max.x + COLUMN_GAP, top, left + ROW_WIDTH, bottom);

    ActionRow { back, forward, open, new, note }
}

/// The line under the buttons: the whole width of the panel, with a row of room
/// under it - see [`MESSAGE_ROWS`].
fn message_rect(report: &SaveReport) -> Rect {
    let (left, top, bottom) = row_bounds(report, BUTTON_ROWS);

    Rect::new(left, top, left + ROW_WIDTH, bottom)
}

fn row_bounds(report: &SaveReport, index: usize) -> (f32, f32, f32) {
    let left = choose_rect(report).min.x + PANEL_PADDING;
    let top = choose_rect(report).min.y + PANEL_PADDING + TITLE_HEIGHT + index as f32 * ROW_HEIGHT;

    (left, top, top + ROW_HEIGHT)
}

/// Every button on the panel and where it is, laid out once and used three
/// times: this is what the panel draws, what a click is read against, and what
/// a test aims at.
pub fn buttons(report: &SaveReport) -> [(ChooseAction, Rect); 4] {
    let row = action_row(report);

    [
        (ChooseAction::Back, row.back),
        (ChooseAction::Forward, row.forward),
        (ChooseAction::Open, row.open),
        (ChooseAction::New, row.new),
    ]
}

/// Where one action's button is.
///
/// What the panel itself asks is [`buttons`], all four at once. This is for a
/// test aiming at one of them, which is not something anything on screen needs
/// to do.
#[cfg(test)]
pub fn button_of(action: ChooseAction, report: &SaveReport) -> Rect {
    buttons(report)
        .into_iter()
        .find(|(button, _)| *button == action)
        .map(|(_, rect)| rect)
        .expect("every action has a button")
}

/// The action a click at `pixel` is aimed at.
///
/// `None` for a click anywhere else, including inside the panel but not on a
/// button - the panel swallows those, it does not act on them.
pub fn action_at(pixel: Vec2, report: &SaveReport) -> Option<ChooseAction> {
    buttons(report)
        .into_iter()
        .find(|(_, rect)| rect.contains(pixel))
        .map(|(action, _)| action)
}

/// What the panel says under its buttons.
///
/// The file the editor is working on - which is the question this panel exists
/// to answer, and worth having on screen all the time rather than only in the
/// moment something arrives - or, when the last thing the panel was asked for
/// could not be done, what went wrong.
fn message(choice: &LevelChoice, editor_level: &EditorLevel) -> (String, Color) {
    if let Some(failed) = &choice.failure {
        return (format!("could not read {failed}"), ORANGE_RED.into());
    }

    match editor_level.source_path() {
        Some(path) => (format!("editing {path}"), WHITE.into()),
        None => ("editing a level with no file yet".to_string(), WHITE.into()),
    }
}

/// What the chooser row names, or what it says instead when the directory has
/// nothing in it to name.
fn chosen(choice: &LevelChoice) -> (&str, Color) {
    match choice.chosen_name() {
        Some(name) => (as_shown(name), WHITE.into()),
        None => ("no levels on disk", DIM_GRAY.into()),
    }
}

/// Puts the panel on screen.
///
/// The whole panel is rebuilt rather than the one line that moved, as the four
/// panels above it are: a handful of nodes is nothing, and this only runs on a
/// frame something it is showing actually changed.
pub fn editor_show_choice(
    choice: Res<LevelChoice>,
    report: Res<SaveReport>,
    editor_level: Res<EditorLevel>,
    shown: Query<Entity, With<LevelPanel>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for entity in &shown {
        commands.entity(entity).despawn();
    }

    commands.spawn((
        panel_node(choose_rect(&report)),
        BackgroundColor(PANEL_BACKGROUND),
        GlobalZIndex(PANEL_Z),
        LevelPanel,
        EditorEntity,
    ));

    // The shortcuts are said once, in the title, as the three panels above say
    // theirs: a button wide enough to hold its own shortcut leaves no room for
    // what it would do.
    commands.spawn((
        panel_text(panel_title_rect(&report), "Level - Ctrl+O / Ctrl+N", TITLE_FONT_SIZE, GOLD.into(), Justify::Left, &asset_server),
        LevelPanel,
        EditorEntity,
    ));

    let actions = action_row(&report);
    let (name, name_colour) = chosen(&choice);

    for (action, rect) in buttons(&report) {
        let colour: Color = match action.live(&choice) {
            true => GOLD.into(),
            false => DIM_GRAY.into(),
        };

        commands.spawn((
            panel_text(rect, action.label(), ROW_FONT_SIZE, colour, Justify::Center, &asset_server),
            BackgroundColor(BUTTON_BACKGROUND),
            action,
            LevelPanel,
            EditorEntity,
        ));
    }

    commands.spawn((
        panel_text(name_rect(&report), name, ROW_FONT_SIZE, name_colour, Justify::Center, &asset_server),
        LevelPanel,
        EditorEntity,
    ));

    commands.spawn((
        panel_text(actions.note, &choice.position(), ROW_FONT_SIZE, SILVER.into(), Justify::Center, &asset_server),
        LevelPanel,
        EditorEntity,
    ));

    let (message, colour) = message(&choice, &editor_level);

    commands.spawn((
        panel_text(message_rect(&report), &message, ROW_FONT_SIZE, colour, Justify::Left, &asset_server),
        ChoiceMessage,
        LevelPanel,
        EditorEntity,
    ));
}


#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use bevy::prelude::default;

    use crate::config::BLOCK_GAP;
    use crate::editor::save::{Complaint, Report};
    use crate::level::campaign::save_level;
    use crate::level::LevelDefinition;
    use crate::level::TargetLayout::SparseGrid;

    /// A directory of this test's own, so two tests listing levels do not list
    /// each other's.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("angleout_c0015_{name}"));

        fs::remove_dir_all(&dir).ok();
        fs::create_dir_all(&dir).expect("a directory to hold levels");

        dir
    }

    fn a_level(dir: &Path, name: &str) {
        save_level(&dir.join(name), &LevelDefinition {
            targets: SparseGrid("AA AA".to_string(), BLOCK_GAP),
            ..default()
        })
            .expect("a level to write");
    }

    /// A report with `lines` lines in it, which is what makes the file panel up
    /// the column grow and everything under it move.
    fn report_of(complaints: usize) -> SaveReport {
        SaveReport(Some(Report {
            outcome: "Saved levels/level0.ron".to_string(),
            failed: false,
            complaints: vec![Complaint::NothingToBreak; complaints],
        }))
    }

    fn listing(names: &[&str]) -> LevelChoice {
        LevelChoice {
            names: names.iter().map(|name| name.to_string()).collect(),
            ..default()
        }
    }


    // --- what there is to open ---------------------------------------------

    #[test]
    fn the_list_is_every_level_file_in_the_directory_but_the_campaign() {
        let dir = scratch("listing");

        a_level(&dir, "level1.ron");
        a_level(&dir, "alpha.ron");
        fs::write(dir.join(CAMPAIGN_FILE), "(levels: [])").unwrap();
        fs::write(dir.join("notes.txt"), "not a level").unwrap();

        assert_eq!(level_names(&dir), vec!["alpha.ron", "level1.ron"]);
    }

    /// The order has to be the same every time the panel is drawn, or a stepper
    /// walks somewhere different on every frame.
    #[test]
    fn the_list_is_in_the_same_order_every_time() {
        let dir = scratch("order");

        for name in ["c.ron", "a.ron", "b.ron"] {
            a_level(&dir, name);
        }

        assert_eq!(level_names(&dir), vec!["a.ron", "b.ron", "c.ron"]);
        assert_eq!(level_names(&dir), level_names(&dir));
    }

    /// A directory that is not there is a panel with nothing to offer, not a
    /// panic - the editor can be pointed anywhere.
    #[test]
    fn a_directory_that_cannot_be_read_offers_nothing() {
        assert_eq!(level_names(Path::new("/nowhere/at/all")), Vec::<String>::new());
    }

    #[test]
    fn the_steppers_walk_the_whole_list_and_wrap_at_both_ends() {
        let mut choice = listing(&["a.ron", "b.ron", "c.ron"]);

        assert_eq!(choice.chosen_name(), Some("a.ron"));

        choice.step(1);
        assert_eq!(choice.chosen_name(), Some("b.ron"));

        choice.step(1);
        assert_eq!(choice.chosen_name(), Some("c.ron"));

        choice.step(1);
        assert_eq!(choice.chosen_name(), Some("a.ron"), "the end wraps to the start");

        choice.step(-1);
        assert_eq!(choice.chosen_name(), Some("c.ron"), "and the start back to the end");
    }

    #[test]
    fn stepping_an_empty_list_names_nothing_and_does_not_panic() {
        let mut choice = listing(&[]);

        choice.step(-1);
        choice.step(1);

        assert_eq!(choice.chosen_name(), None);
        assert_eq!(choice.position(), "none");
    }

    /// The panel opens naming what the author is already working on, rather than
    /// whichever file happens to sort first.
    #[test]
    fn entering_points_the_chooser_at_the_level_under_edit() {
        let dir = scratch("points_at");

        for name in ["a.ron", "b.ron", "c.ron"] {
            a_level(&dir, name);
        }

        let mut choice = LevelChoice::default();
        choice.refresh(&dir, Some("levels/b.ron"));

        assert_eq!(choice.chosen_name(), Some("b.ron"));
        assert_eq!(choice.position(), "2 of 3");
    }

    /// A level that has never been on disk has no file to point at, and neither
    /// does one whose file has since been deleted.
    #[test]
    fn a_level_with_no_file_of_its_own_leaves_the_chooser_at_the_first() {
        let dir = scratch("no_file");
        a_level(&dir, "a.ron");

        let mut choice = LevelChoice::default();

        choice.refresh(&dir, None);
        assert_eq!(choice.chosen_name(), Some("a.ron"));

        choice.refresh(&dir, Some("levels/gone.ron"));
        assert_eq!(choice.chosen_name(), Some("a.ron"));
    }

    /// A save can put a file in the directory that was not there when the editor
    /// opened, and the chooser has to be able to offer it without the author
    /// leaving and coming back.
    #[test]
    fn a_file_that_appears_joins_the_list_where_the_choice_stays_put() {
        let dir = scratch("appears");

        a_level(&dir, "b.ron");
        let mut choice = LevelChoice::default();
        choice.refresh(&dir, Some("levels/b.ron"));

        a_level(&dir, "a.ron");
        choice.relist(level_names(&dir));

        assert_eq!(choice.names, vec!["a.ron", "b.ron"]);
        assert_eq!(choice.chosen_name(), Some("b.ron"), "the file that sorted in front must not steal the choice");
    }

    /// The chooser cannot be left pointing past the end of a list that shrank.
    #[test]
    fn a_file_that_goes_away_leaves_the_chooser_inside_the_list() {
        let dir = scratch("goes_away");

        for name in ["a.ron", "b.ron"] {
            a_level(&dir, name);
        }

        let mut choice = LevelChoice::default();
        choice.refresh(&dir, Some("levels/b.ron"));

        fs::remove_file(dir.join("b.ron")).unwrap();
        choice.relist(level_names(&dir));

        assert_eq!(choice.chosen_name(), Some("a.ron"));
    }

    #[test]
    fn the_chooser_writes_a_name_without_the_extension_they_all_share() {
        assert_eq!(as_shown("level4.ron"), "level4");
        assert_eq!(as_shown("level4"), "level4");
    }


    // --- what the panel says ------------------------------------------------

    /// The card's own question, answered on screen: the panel names the file
    /// being edited whenever there is nothing more pressing to say.
    #[test]
    fn the_panel_says_which_file_is_being_edited() {
        let editing = EditorLevel { source: None, level: LevelDefinition::default() };

        assert_eq!(
            message(&listing(&[]), &editing),
            ("editing a level with no file yet".to_string(), WHITE.into()),
        );
    }

    #[test]
    fn a_file_that_would_not_read_is_said_in_the_colour_of_a_complaint() {
        let choice = LevelChoice { failure: Some("levels/broken.ron".to_string()), ..default() };
        let editing = EditorLevel { source: None, level: LevelDefinition::default() };

        assert_eq!(
            message(&choice, &editing),
            ("could not read levels/broken.ron".to_string(), ORANGE_RED.into()),
        );
    }

    /// With nothing on disk there is nothing to open, and the panel says so
    /// rather than naming a file that is not there.
    #[test]
    fn an_empty_directory_is_said_rather_than_left_blank() {
        assert_eq!(chosen(&listing(&[])), ("no levels on disk", DIM_GRAY.into()));
        assert_eq!(chosen(&listing(&["a.ron"])), ("a", WHITE.into()));
    }

    /// Everything but `New` needs a file to work on, and a dead button is dimmed
    /// rather than taken away.
    #[test]
    fn nothing_but_new_is_live_with_an_empty_directory() {
        let empty = listing(&[]);

        assert!(ChooseAction::New.live(&empty));
        assert!(!ChooseAction::Open.live(&empty));
        assert!(!ChooseAction::Back.live(&empty));
        assert!(!ChooseAction::Forward.live(&empty));

        assert!(ChooseAction::Open.live(&listing(&["a.ron"])));
    }


    // --- where the panel is -------------------------------------------------

    /// The panel is the foot of the editor's column, and it has to stay under
    /// the panel above it however much the file panel has to say.
    #[test]
    fn the_panel_sits_under_the_playtest_panel_whatever_the_report_says() {
        for complaints in [0, 1, 5] {
            let report = report_of(complaints);

            assert!(
                choose_rect(&report).min.y >= playtest_rect(&report).max.y,
                "the panels overlap with {complaints} complaints on screen",
            );
        }
    }

    #[test]
    fn the_panel_moves_down_as_the_report_grows() {
        assert!(choose_rect(&report_of(3)).min.y > choose_rect(&report_of(0)).min.y);
    }

    /// Everything a click can be aimed at is inside what the editor keeps its
    /// hands off - a button hanging over the edge would be a button that paints
    /// the cell behind it.
    #[test]
    fn every_button_is_inside_the_panel() {
        let report = report_of(0);
        let panel = choose_rect(&report);
        let actions = action_row(&report);

        for rect in [name_rect(&report), actions.back, actions.forward, actions.open, actions.new, actions.note, message_rect(&report)] {
            assert!(panel.contains(rect.min) && panel.contains(rect.max), "{rect:?} is outside {panel:?}");
        }
    }

    /// Two buttons that overlap are two buttons an author cannot tell apart.
    #[test]
    fn no_two_buttons_share_a_pixel() {
        let report = report_of(0);

        let rects: Vec<Rect> = buttons(&report).into_iter().map(|(_, rect)| rect).collect();

        for (index, one) in rects.iter().enumerate() {
            for other in &rects[index + 1..] {
                assert!(one.intersect(*other).is_empty(), "{one:?} overlaps {other:?}");
            }
        }
    }

    #[test]
    fn a_click_finds_the_button_it_landed_on() {
        let report = report_of(0);

        for (action, rect) in buttons(&report) {
            assert_eq!(action_at(rect.center(), &report), Some(action));
            assert_eq!(button_of(action, &report), rect);
        }
    }

    /// The panel swallows a click that is on it but not on a button - it does
    /// not act on it, and neither does the grid underneath.
    #[test]
    fn a_click_on_the_panel_but_not_on_a_button_is_nothing() {
        let report = report_of(0);

        assert_eq!(action_at(message_rect(&report).center(), &report), None);
        assert_eq!(action_at(name_rect(&report).center(), &report), None);
        assert_eq!(action_at(Vec2::new(800.0, 400.0), &report), None, "and neither is a click out on the grid");
    }
}

