//! Saving the level under edit, and saying what is wrong with it.
//!
//! Two halves that only look like one card. The first is the write: `std::fs`
//! rather than the asset server, which reads and never writes - plus the
//! consequence of writing into a directory `c0004` has a file watcher on, which
//! is that the editor's own save comes straight back at it as a hand edit. The
//! second is the *reading over the author's shoulder*: a level can be saved in
//! states the game can load and never play, and the editor says so.
//!
//! It says so and then saves anyway. There is no such thing as a level too
//! broken to write down - an author halfway through wiring a trigger up has a
//! level with a receiver and no trigger in it, and a save that refused would be
//! a save that punished them for stopping for lunch. Every complaint here is a
//! remark, never a veto.
//!
//! What is on screen is the third panel in the editor's left-hand column, laid
//! out and hit-tested against its own rectangles exactly as `c0010`'s settings
//! panel and `c0011`'s history bar are.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use bevy::asset::AssetServer;
use bevy::color::palettes::css::{DIM_GRAY, GOLD, ORANGE_RED, SILVER, WHITE};
use bevy::ecs::change_detection::DetectChangesMut;
use bevy::log::{info, warn};
use bevy::prelude::{Assets, ButtonInput, Color, Commands, Component, Entity, KeyCode, MouseButton, Query, Rect, Res, ResMut, Resource, Vec2, With};
use bevy::text::Justify;
use bevy::ui::{BackgroundColor, GlobalZIndex};
use bevy::window::{PrimaryWindow, Window};

use crate::block::trigger::{TriggerGroup, TriggerType};
use crate::block::{Block, BlockBehaviour, BlockType};
use crate::level::asset::LevelAsset;
use crate::level::campaign::{level_asset_path, load_campaign, save_campaign, save_level, LevelSaveError};
use crate::level::{LevelDefinition, TargetLayout};

use super::history::history_rect;
use super::settings::{panel_node, panel_text, BUTTON_BACKGROUND, COLUMN_GAP, PANEL_BACKGROUND, PANEL_ORIGIN, PANEL_PADDING, PANEL_Z, ROW_FONT_SIZE, ROW_HEIGHT, ROW_INSET, ROW_WIDTH, TITLE_FONT_SIZE, TITLE_HEIGHT};
use super::{blocks_of, commanding, EditorEntity, EditorLevel};


// --- where the editor writes ----------------------------------------------

/// The directory the editor saves into.
///
/// The same one [`Levels`](crate::level::Levels) was read from, but held as a
/// resource rather than looked up at the moment of writing: `levels_dir()` is
/// the running game's `assets/levels`, and a test that saves has to be able to
/// save somewhere that is not the repository's own.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct LevelsOnDisk(pub PathBuf);

/// The last thing the editor wrote to disk.
///
/// Kept so that [`editor_watch_the_file`](super::history::editor_watch_the_file)
/// can tell our own write apart from somebody else's. The file watcher cannot:
/// it reports a change to a file, not who made it.
#[derive(Resource, Debug, Default)]
pub struct LastSave(pub Option<LevelDefinition>);

impl LastSave {
    /// Whether the file the editor opened now holds exactly what the editor last
    /// wrote to it.
    ///
    /// Compared by value rather than by a flag set across the write, because the
    /// watcher answers on its own schedule - a debounce later, with however many
    /// further edits made in between - and a flag would have to guess how long to
    /// stay up. What we wrote does not go stale.
    pub fn is_what_the_file_now_says(
        &self,
        editor_level: &EditorLevel,
        level_assets: &Assets<LevelAsset>,
    ) -> bool {
        let Some(saved) = &self.0 else { return false; };
        let Some(handle) = &editor_level.source else { return false; };

        level_assets.get(handle).is_some_and(|asset| asset.0 == *saved)
    }
}


// --- what is wrong with the level -----------------------------------------

/// Something structurally wrong with a level, worth saying out loud and not
/// worth refusing a save over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Complaint {
    /// Receivers in a group that nothing ever triggers. They sit in whatever
    /// state they start in for the whole match.
    ReceiversWithNoTrigger { group: TriggerGroup, receivers: usize },

    /// Portals that are not trigger receivers.
    ///
    /// `block_update_portals` only moves a ball through a portal that carries a
    /// `BlockTriggerTarget`, which is what the `R` and `S` trigger types put on
    /// one - so a portal that is not a receiver is scenery. That is a wider net
    /// than "a portal with no trigger": a portal marked `A` is a trigger rather
    /// than a receiver, and is every bit as shut.
    PortalsThatNothingOpens { portals: usize },

    /// No blocks a ball can destroy, so the win condition can never be reached.
    NothingToBreak,
}

impl Complaint {
    /// What the author is told, in the one place it is worded.
    pub fn message(&self) -> String {
        match self {
            Complaint::ReceiversWithNoTrigger { group, receivers } => format!(
                "Trigger group {group}: {receivers} receiver{} and no trigger to start {}",
                plural(*receivers),
                if *receivers == 1 { "it" } else { "them" },
            ),

            Complaint::PortalsThatNothingOpens { portals } => format!(
                "{portals} portal{} not a trigger receiver - {} never open",
                if *portals == 1 { " is" } else { "s are" },
                if *portals == 1 { "it can" } else { "they can" },
            ),

            Complaint::NothingToBreak => {
                "Nothing here can be broken - this level can never be won".to_string()
            }
        }
    }
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

/// Everything the editor has to say about a level, in a fixed order so that two
/// saves of the same level read the same way.
pub fn complaints(level: &LevelDefinition) -> Vec<Complaint> {
    // A `Custom` layout's blocks are spawned in code rather than authored, so
    // there is nothing here for the editor to have read and nothing for it to
    // have an opinion about.
    if matches!(level.targets, TargetLayout::Custom(_)) {
        return vec![];
    }

    let blocks = blocks_of(level);
    let mut complaints = vec![];

    let triggered: BTreeSet<TriggerGroup> = blocks
        .iter()
        .filter(|block| block.trigger_type.as_ref().is_some_and(|t| !is_receiver(t)))
        .map(group_of)
        .collect();

    let mut orphaned: BTreeMap<TriggerGroup, usize> = BTreeMap::new();

    for block in blocks.iter().filter(|block| block.trigger_type.as_ref().is_some_and(is_receiver)) {
        let group = group_of(block);

        if !triggered.contains(&group) {
            *orphaned.entry(group).or_default() += 1;
        }
    }

    complaints.extend(
        orphaned
            .into_iter()
            .map(|(group, receivers)| Complaint::ReceiversWithNoTrigger { group, receivers }),
    );

    let shut = blocks
        .iter()
        .filter(|block| block.behaviour == BlockBehaviour::Portal)
        .filter(|block| !block.trigger_type.as_ref().is_some_and(is_receiver))
        .count();

    if shut > 0 {
        complaints.push(Complaint::PortalsThatNothingOpens { portals: shut });
    }

    // An obstacle is a grid cell that happens to be unbreakable, and
    // `make_grid_from_string_layout` leaves it out of the count a match plays
    // against - so a wall of them is a level with nothing in it.
    if !blocks.iter().any(|block| block.block_type != BlockType::Obstacle) {
        complaints.push(Complaint::NothingToBreak);
    }

    complaints
}

fn is_receiver(trigger: &TriggerType) -> bool {
    matches!(
        trigger,
        TriggerType::ReceiverStartingInactive | TriggerType::ReceiverStartingActive
    )
}

/// The group a block's trigger belongs to, as `block_spawn` reads it: a token
/// carrying a trigger character with no group digit behind it is group 0.
fn group_of(block: &Block) -> TriggerGroup {
    block.trigger_group.unwrap_or(0)
}


// --- what the last save had to say ----------------------------------------

/// The outcome of the last thing the author asked of a file, and everything the
/// editor has to say about the level it holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// What happened, worded as the panel says it.
    pub outcome: String,

    /// Whether that outcome was a failure - the one line the panel writes in red
    /// on its own account rather than the level's.
    pub failed: bool,

    /// What is wrong with the level that was written anyway. Recomputed for
    /// every report, so what is on screen is about the level as it stands and
    /// not as it stood at some earlier save.
    pub complaints: Vec<Complaint>,
}

/// The report on screen, or `None` before the author has asked for anything.
#[derive(Resource, Debug, Default, PartialEq, Eq)]
pub struct SaveReport(pub Option<Report>);

impl SaveReport {
    fn say(&mut self, outcome: impl Into<String>, failed: bool, level: &LevelDefinition) {
        self.0 = Some(Report {
            outcome: outcome.into(),
            failed,
            complaints: complaints(level),
        });
    }

    /// The lines the panel writes under its buttons, each in the colour it is
    /// written in.
    pub fn lines(&self) -> Vec<(String, Color)> {
        let Some(report) = &self.0 else { return vec![]; };

        let outcome = match report.failed {
            true => (report.outcome.clone(), ORANGE_RED.into()),
            false => (report.outcome.clone(), WHITE.into()),
        };

        std::iter::once(outcome)
            .chain(report.complaints.iter().map(|c| (c.message(), ORANGE_RED.into())))
            .collect()
    }
}


// --- doing it -------------------------------------------------------------

/// One of the two things the panel can be asked for.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveAction {
    Save,
    Campaign,
}

/// The actions, in the order they are on screen.
pub const SAVE_ACTIONS: [SaveAction; 2] = [SaveAction::Save, SaveAction::Campaign];

impl SaveAction {
    /// What the button is called on screen.
    pub fn label(&self) -> &'static str {
        match self {
            SaveAction::Save => "Save",
            SaveAction::Campaign => "Campaign",
        }
    }

    /// What it would do, said against the level as it stands - and whether it is
    /// something it can do at all.
    ///
    /// A row that would do nothing is drawn dimmed rather than left off, as the
    /// history bar draws a step with nothing to take: a button that comes and
    /// goes is one an author has to look for.
    fn what(&self, editor_level: &EditorLevel) -> (String, bool) {
        match (self, editor_level.source_path()) {
            (SaveAction::Save, Some(path)) => (path, true),
            (SaveAction::Save, None) => ("a new file in levels/".to_string(), true),
            (SaveAction::Campaign, Some(_)) => ("add to the campaign".to_string(), true),
            (SaveAction::Campaign, None) => ("save the level first".to_string(), false),
        }
    }
}

/// `Ctrl+S`, as every other program on the machine spells it.
pub fn editor_save_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    dir: Res<LevelsOnDisk>,
    asset_server: Res<AssetServer>,
    mut last_save: ResMut<LastSave>,
    mut report: ResMut<SaveReport>,
    mut editor_level: ResMut<EditorLevel>,
) {
    if !commanding(&keys) || !keys.just_pressed(KeyCode::KeyS) {
        return;
    }

    save(&mut editor_level, &dir.0, &asset_server, &mut last_save, &mut report);
}

/// The same, plus enrolling a level in the campaign, from the panel.
///
/// The press rather than the hold, as the settings panel and the history bar
/// read their buttons.
pub fn editor_save_click(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    dir: Res<LevelsOnDisk>,
    asset_server: Res<AssetServer>,
    mut last_save: ResMut<LastSave>,
    mut report: ResMut<SaveReport>,
    mut editor_level: ResMut<EditorLevel>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(cursor) = windows.iter().next().and_then(|window| window.cursor_position()) else { return; };
    let Some(action) = action_at(cursor) else { return; };

    match action {
        SaveAction::Save => save(&mut editor_level, &dir.0, &asset_server, &mut last_save, &mut report),
        SaveAction::Campaign => enrol(&editor_level, &dir.0, &mut report),
    }
}

/// Writes the level under edit out, and reads it over for the author on the way.
///
/// Nothing about a complaint stops the write - the file is on disk before the
/// level is read at all.
fn save(
    editor_level: &mut ResMut<EditorLevel>,
    dir: &Path,
    asset_server: &AssetServer,
    last_save: &mut ResMut<LastSave>,
    report: &mut ResMut<SaveReport>,
) {
    let name = match editor_level.source_path() {
        Some(path) => file_name(&path).to_string(),
        None => next_free_name(dir),
    };

    let path = dir.join(&name);
    let level = editor_level.level.clone();

    if let Err(e) = save_level(&path, &level) {
        warn!("could not save the level: {e}");
        report.say(could_not(&name, &e), true, &level);

        return;
    }

    // What the watcher will hand back to us in a moment, so that our own write
    // is not read as somebody else's hand edit.
    last_save.0 = Some(level.clone());

    // A level that had never been on disk has a file now, and the next save
    // belongs in that file rather than in another new one. Written around change
    // detection because the level itself has not moved - only where it lives -
    // and the panels that hang off a changed level have nothing new to say.
    if editor_level.source.is_none() {
        editor_level.bypass_change_detection().source = Some(asset_server.load(level_asset_path(&name)));
    }

    let complaints = complaints(&level);
    info!("saved {}", path.display());

    for complaint in &complaints {
        warn!("{}", complaint.message());
    }

    report.say(format!("Saved {}", level_asset_path(&name)), false, &level);
}

/// Adds the level's file to the campaign, so the game plays it.
///
/// The index names files, not levels under edit, so a level that has never been
/// saved has nothing to enrol - and one that is already in the campaign is left
/// exactly where it is rather than played twice.
fn enrol(editor_level: &EditorLevel, dir: &Path, report: &mut ResMut<SaveReport>) {
    let level = &editor_level.level;

    let Some(path) = editor_level.source_path() else {
        report.say("Save the level first - the campaign names files", true, level);
        return;
    };

    let name = file_name(&path).to_string();

    let mut campaign = match load_campaign(dir) {
        Ok(campaign) => campaign,
        Err(e) => {
            warn!("could not read the campaign: {e}");
            report.say(format!("Could not read the campaign: {e}"), true, level);

            return;
        }
    };

    if campaign.levels.contains(&name) {
        report.say(format!("{path} is already in the campaign"), false, level);
        return;
    }

    campaign.levels.push(name);

    if let Err(e) = save_campaign(dir, &campaign) {
        warn!("could not write the campaign: {e}");
        report.say(format!("Could not write the campaign: {e}"), true, level);

        return;
    }

    info!("{path} is now the last level of the campaign");
    report.say(format!("{path} is now the last level of the campaign"), false, level);
}

fn could_not(name: &str, error: &LevelSaveError) -> String {
    format!("Could not save {}: {}", level_asset_path(name), because(error))
}

/// The error's own words, without the path it says as well - the panel has one
/// line for this and has already named the file.
fn because(error: &LevelSaveError) -> String {
    match error {
        LevelSaveError::Io(_, e) => e.to_string(),
        LevelSaveError::Ron(e) => e.to_string(),
    }
}

/// The file name of an asset path - what `levels/level4.ron` is called in the
/// directory the editor writes to.
fn file_name(asset_path: &str) -> &str {
    asset_path.rsplit('/').next().unwrap_or(asset_path)
}

/// The name a level that has never been on disk is given.
///
/// There is nowhere in this game to type a file name, so the editor picks one:
/// `levelN.ron`, one past the highest number the directory already holds. It is
/// the name the shipped levels already use, and the number means nothing beyond
/// "not taken" - play order is `campaign.ron`'s to say, not the file name's.
pub fn next_free_name(dir: &Path) -> String {
    let highest = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| numbered_level(&entry.file_name().to_string_lossy()))
        .max();

    let mut next = highest.map_or(0, |number| number + 1);

    // A gap in the numbering is nobody's business; a name that is already taken
    // is - and a directory we could not read at all leaves us guessing.
    while dir.join(level_file_name(next)).exists() {
        next += 1;
    }

    level_file_name(next)
}

fn level_file_name(number: u32) -> String {
    format!("level{number}.ron")
}

fn numbered_level(name: &str) -> Option<u32> {
    name.strip_prefix("level")?.strip_suffix(".ron")?.parse().ok()
}


// --- the panel ------------------------------------------------------------

/// How far under the history bar the panel sits.
const PANEL_GAP: f32 = 8.0;

/// How wide the button that does the thing is - as little as "Campaign" needs,
/// so the rest of the row is free to say what it would do.
const ACTION_WIDTH: f32 = 84.0;

/// How many rows of the panel a line of the report is given.
///
/// Two, because a report line is a sentence where every other row of every other
/// panel is a label: "Trigger group 3: 1 receiver and no trigger to start it"
/// does not fit the column at [`ROW_FONT_SIZE`] and wraps onto a second line.
/// The text node itself stays one row tall - what is reserved here is the room
/// under it for the wrap to land in.
const REPORT_ROWS: usize = 2;

/// Marks everything the panel draws, so a new report can take the whole of it
/// down and put it up again saying the new thing.
#[derive(Component)]
pub struct SavePanel;

/// A line of the report on screen, tagged with where in the report it is - a
/// query hands entities back in no order a test can rely on.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportLine(pub usize);

/// One action's row, as rectangles on the window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SaveRow {
    pub action: SaveAction,
    pub button: Rect,
    pub what: Rect,
}

/// The panel's footprint: under the history bar, in the same column and the same
/// width, and as tall as the report it is carrying.
///
/// What the editor keeps its hands off, as it does the two panels above it - a
/// click in here is aimed at a file, and the cell that happens to be underneath
/// is not what the author meant.
pub fn save_rect(report: &SaveReport) -> Rect {
    let top = panel_top();
    let rows = SAVE_ACTIONS.len() + REPORT_ROWS * report.lines().len();
    let height = PANEL_PADDING + TITLE_HEIGHT + rows as f32 * ROW_HEIGHT + PANEL_PADDING;

    Rect::new(
        PANEL_ORIGIN.x,
        top,
        PANEL_ORIGIN.x + ROW_WIDTH + 2.0 * PANEL_PADDING,
        top + height,
    )
}

/// Where the panel starts. Fixed, where its bottom is not: the report grows
/// downwards, into empty screen, rather than pushing the column around.
fn panel_top() -> f32 {
    history_rect().max.y + PANEL_GAP
}

fn panel_title_rect() -> Rect {
    let left = PANEL_ORIGIN.x + PANEL_PADDING;
    let top = panel_top() + PANEL_PADDING;

    Rect::new(left, top, left + ROW_WIDTH, top + TITLE_HEIGHT)
}

/// The rows of the panel, laid out once and used twice: this is what the panel
/// draws, and what a click is read against.
pub fn save_rows() -> Vec<SaveRow> {
    SAVE_ACTIONS
        .iter()
        .enumerate()
        .map(|(index, action)| save_row(*action, index))
        .collect()
}

fn save_row(action: SaveAction, index: usize) -> SaveRow {
    let (left, top, bottom) = row_bounds(index);

    // The button is kept clear of the rows above and below it, so two of them do
    // not run into one another.
    let button = Rect::new(left, top + ROW_INSET, left + ACTION_WIDTH, bottom - ROW_INSET);
    let what = Rect::new(button.max.x + COLUMN_GAP, top, left + ROW_WIDTH, bottom);

    SaveRow { action, button, what }
}

/// A report line's rectangle: the whole width of the panel, under the buttons,
/// and one row tall with a row of room under it - see [`REPORT_ROWS`].
fn report_rect(index: usize) -> Rect {
    let (left, top, bottom) = row_bounds(SAVE_ACTIONS.len() + REPORT_ROWS * index);

    Rect::new(left, top, left + ROW_WIDTH, bottom)
}

fn row_bounds(index: usize) -> (f32, f32, f32) {
    let left = PANEL_ORIGIN.x + PANEL_PADDING;
    let top = panel_top() + PANEL_PADDING + TITLE_HEIGHT + index as f32 * ROW_HEIGHT;

    (left, top, top + ROW_HEIGHT)
}

/// The action a click at `pixel` is aimed at.
///
/// `None` for a click anywhere else, including inside the panel but not on a
/// button - the panel swallows those, it does not act on them.
pub fn action_at(pixel: Vec2) -> Option<SaveAction> {
    save_rows()
        .into_iter()
        .find(|row| row.button.contains(pixel))
        .map(|row| row.action)
}

/// Puts the panel on screen, saying what the last save had to say.
///
/// The whole panel is rebuilt rather than the one line that moved, as the
/// settings panel and the history bar are: a handful of nodes is nothing, and
/// this only runs on a frame something it is showing actually changed.
pub fn editor_show_save(
    report: Res<SaveReport>,
    editor_level: Res<EditorLevel>,
    shown: Query<Entity, With<SavePanel>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for entity in &shown {
        commands.entity(entity).despawn();
    }

    commands.spawn((
        panel_node(save_rect(&report)),
        BackgroundColor(PANEL_BACKGROUND),
        GlobalZIndex(PANEL_Z),
        SavePanel,
        EditorEntity,
    ));

    // The shortcut is said once, in the title, as the history bar says its two:
    // a button wide enough to hold its own shortcut leaves no room for what it
    // would do.
    commands.spawn((
        panel_text(panel_title_rect(), "File - Ctrl+S", TITLE_FONT_SIZE, GOLD.into(), Justify::Left, &asset_server),
        SavePanel,
        EditorEntity,
    ));

    for row in save_rows() {
        let (what, live) = row.action.what(&editor_level);

        let (button_colour, what_colour): (Color, Color) = match live {
            true => (GOLD.into(), SILVER.into()),
            false => (DIM_GRAY.into(), DIM_GRAY.into()),
        };

        commands.spawn((
            panel_text(row.button, row.action.label(), ROW_FONT_SIZE, button_colour, Justify::Center, &asset_server),
            BackgroundColor(BUTTON_BACKGROUND),
            row.action,
            SavePanel,
            EditorEntity,
        ));

        commands.spawn((
            panel_text(row.what, &what, ROW_FONT_SIZE, what_colour, Justify::Left, &asset_server),
            SavePanel,
            EditorEntity,
        ));
    }

    for (index, (line, colour)) in report.lines().into_iter().enumerate() {
        commands.spawn((
            panel_text(report_rect(index), &line, ROW_FONT_SIZE, colour, Justify::Left, &asset_server),
            ReportLine(index),
            SavePanel,
            EditorEntity,
        ));
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use bevy::prelude::default;

    use crate::config::BLOCK_GAP;
    use crate::level::campaign::{load_campaign, load_level, Campaign, CAMPAIGN_FILE};
    use super::super::settings::panel_rect;
    use crate::level::TargetLayout::SparseGrid;

    fn sparse(layout: &str) -> LevelDefinition {
        LevelDefinition {
            targets: SparseGrid(layout.to_string(), BLOCK_GAP),
            ..default()
        }
    }

    /// A directory of this test's own, so two tests writing levels do not write
    /// each other's.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("angleout_c0012_{name}"));

        fs::remove_dir_all(&dir).ok();
        fs::create_dir_all(&dir).expect("a directory to write levels into");

        dir
    }

    fn messages(level: &LevelDefinition) -> Vec<String> {
        complaints(level).iter().map(Complaint::message).collect()
    }


    // --- reading the level over the author's shoulder ----------------------

    #[test]
    fn a_level_of_plain_blocks_has_nothing_wrong_with_it() {
        assert_eq!(complaints(&sparse("AA AA AA\nAA AA AA")), vec![]);
    }

    /// The card's first rule. `R` and `S` are receivers; `A`, `B` and `C` are
    /// what starts and stops them. A receiver in a group with none of the latter
    /// sits in the state it started in for the whole match.
    #[test]
    fn a_receiver_whose_group_holds_no_trigger_is_called_out() {
        assert_eq!(
            complaints(&sparse("AA AAR3 AA")),
            vec![Complaint::ReceiversWithNoTrigger { group: 3, receivers: 1 }],
        );
    }

    #[test]
    fn a_receiver_whose_group_holds_a_trigger_is_left_alone() {
        assert_eq!(complaints(&sparse("AAA3 AAR3 AA")), vec![]);
        assert_eq!(complaints(&sparse("AAB3 AAS3 AA")), vec![]);
        assert_eq!(complaints(&sparse("AAC3 AAR3 AA")), vec![]);
    }

    /// The trigger has to be in the receiver's *own* group - that is the whole
    /// of what a group is for.
    #[test]
    fn a_trigger_in_another_group_is_no_help() {
        assert_eq!(
            complaints(&sparse("AAA1 AAR2 AA")),
            vec![Complaint::ReceiversWithNoTrigger { group: 2, receivers: 1 }],
        );
    }

    #[test]
    fn every_orphaned_group_is_named_once_with_its_receivers_counted() {
        assert_eq!(
            complaints(&sparse("AAR5 AAR5 AA\nAAR2 AA   AA")),
            vec![
                Complaint::ReceiversWithNoTrigger { group: 2, receivers: 1 },
                Complaint::ReceiversWithNoTrigger { group: 5, receivers: 2 },
            ],
            "in group order, so two saves of the same level read the same way",
        );
    }

    /// `make_block` reads a trigger character with no group digit behind it as
    /// no group at all, and `block_spawn` then plays it as group 0. The editor
    /// has to read it the same way or it would call out a pairing that works.
    #[test]
    fn a_trigger_with_no_group_digit_is_group_zero_here_too() {
        assert_eq!(complaints(&sparse("AAA AAR0 AA")), vec![]);
        assert_eq!(complaints(&sparse("AAA0 AAR AA")), vec![]);
    }

    /// The card's second rule. `block_update_portals` only ever moves a ball
    /// through a portal carrying a `BlockTriggerTarget`, which is what the `R`
    /// and `S` trigger types put on one.
    #[test]
    fn a_portal_that_is_not_a_receiver_is_called_out() {
        assert_eq!(
            complaints(&sparse("AI AA AA")),
            vec![Complaint::PortalsThatNothingOpens { portals: 1 }],
        );
    }

    #[test]
    fn a_portal_that_is_a_receiver_with_a_trigger_is_left_alone() {
        assert_eq!(complaints(&sparse("AAA4 AIR4 AA")), vec![]);
    }

    /// A portal marked as a *trigger* rather than a receiver is every bit as
    /// shut as one with no trigger at all - and its group is orphaned besides.
    #[test]
    fn a_portal_that_triggers_rather_than_receives_is_still_shut() {
        assert_eq!(
            complaints(&sparse("AIA4 AA AA")),
            vec![Complaint::PortalsThatNothingOpens { portals: 1 }],
        );
    }

    /// The card's third rule. An obstacle is a cell a ball bounces off and never
    /// breaks, and `make_grid_from_string_layout` leaves it out of the count the
    /// win criteria are measured against.
    #[test]
    fn a_level_of_nothing_but_obstacles_can_never_be_won() {
        assert_eq!(complaints(&sparse("ZA ZA ZA")), vec![Complaint::NothingToBreak]);
    }

    #[test]
    fn a_level_with_no_blocks_at_all_can_never_be_won_either() {
        assert_eq!(complaints(&sparse(".. .. ..")), vec![Complaint::NothingToBreak]);
    }

    #[test]
    fn one_breakable_block_among_the_obstacles_is_enough() {
        assert_eq!(complaints(&sparse("ZA AA ZA")), vec![]);
    }

    /// A `Custom` layout's blocks are spawned in code, so there is nothing in the
    /// level for the editor to have read - and "no breakable blocks" would be a
    /// complaint about a level it cannot see.
    #[test]
    fn a_level_built_in_code_is_not_judged() {
        let conveyor = LevelDefinition {
            targets: TargetLayout::Custom("Conveyor".to_string()),
            ..default()
        };

        assert_eq!(complaints(&conveyor), vec![]);
    }

    /// A `FilledGrid` is read through the token grid that says the same thing,
    /// as everything else in the editor reads one.
    #[test]
    fn a_filled_grid_is_read_like_the_grid_it_stands_for() {
        let obstacles = LevelDefinition {
            targets: TargetLayout::FilledGrid(3, 2, BlockType::Obstacle, BlockBehaviour::SittingDuck, BLOCK_GAP),
            ..default()
        };

        assert_eq!(complaints(&obstacles), vec![Complaint::NothingToBreak]);
    }

    #[test]
    fn every_complaint_says_what_is_wrong_in_words() {
        // `ZIR1` is the obstacle portal `level0.ron` opens on, minus its trigger:
        // a receiver, so the portal itself is not shut, but nothing to start it
        // - and nothing on the grid a ball could break either.
        assert_eq!(
            messages(&sparse("ZIR7 ZA ZA")),
            vec![
                "Trigger group 7: 1 receiver and no trigger to start it",
                "Nothing here can be broken - this level can never be won",
            ],
        );

        assert_eq!(
            messages(&sparse("AI AI AA")),
            vec!["2 portals are not a trigger receiver - they can never open"],
        );

        assert_eq!(
            messages(&sparse("AAR7 AAR7 AA")),
            vec!["Trigger group 7: 2 receivers and no trigger to start them"],
        );
    }


    // --- naming a level that has never been on disk ------------------------

    #[test]
    fn a_new_level_is_named_one_past_the_highest_the_directory_holds() {
        let dir = scratch("naming");

        for name in ["level0.ron", "level6.ron", "conveyor.ron", "campaign.ron"] {
            fs::write(dir.join(name), "").unwrap();
        }

        assert_eq!(next_free_name(&dir), "level7.ron");
    }

    #[test]
    fn the_first_level_of_an_empty_directory_is_level_zero() {
        assert_eq!(next_free_name(&scratch("naming_empty")), "level0.ron");
    }


    // --- the campaign ------------------------------------------------------

    /// Appending has to leave the levels already in the campaign where they are,
    /// in the order they were in - the index *is* the play order.
    #[test]
    fn a_level_added_to_the_campaign_goes_on_the_end() {
        let dir = scratch("campaign_append");
        let campaign = Campaign { levels: vec!["level0.ron".to_string(), "level1.ron".to_string()] };
        save_campaign(&dir, &campaign).unwrap();

        let mut appended = campaign.clone();
        appended.levels.push("level7.ron".to_string());
        save_campaign(&dir, &appended).unwrap();

        assert_eq!(load_campaign(&dir).unwrap(), appended);
        assert!(
            fs::read_to_string(dir.join(CAMPAIGN_FILE)).unwrap().contains("// The campaign, in play order."),
            "the header survives being appended to",
        );
    }

    // --- the panel ---------------------------------------------------------

    fn reporting(complaints: Vec<Complaint>) -> SaveReport {
        SaveReport(Some(Report { outcome: "Saved levels/level0.ron".to_string(), failed: false, complaints }))
    }

    /// Three panels in one column, laid out from the same numbers so they cannot
    /// drift into one another.
    #[test]
    fn the_panel_sits_under_the_two_above_it_in_the_same_column() {
        let quiet = SaveReport(None);

        assert!(panel_rect().max.y < history_rect().min.y, "the history bar is under the settings");
        assert!(history_rect().max.y < save_rect(&quiet).min.y, "and this is under the bar");

        assert_eq!(save_rect(&quiet).min.x, panel_rect().min.x);
        assert_eq!(save_rect(&quiet).max.x, panel_rect().max.x);
    }

    /// The report grows the panel downwards, into empty screen, rather than
    /// pushing the column above it around.
    #[test]
    fn a_panel_with_something_to_say_grows_downwards() {
        let quiet = SaveReport(None);
        let loud = reporting(vec![Complaint::NothingToBreak]);

        assert_eq!(save_rect(&loud).min.y, save_rect(&quiet).min.y);
        assert!(save_rect(&loud).max.y > save_rect(&quiet).max.y);
    }

    /// Everything the panel draws is inside the panel it draws it in - the rows
    /// are laid out from the top, and the height is worked out separately.
    #[test]
    fn every_row_of_the_panel_is_inside_it() {
        let report = reporting(vec![
            Complaint::ReceiversWithNoTrigger { group: 3, receivers: 2 },
            Complaint::NothingToBreak,
        ]);

        let rect = save_rect(&report);

        for row in save_rows() {
            assert!(rect.contains(row.button.min) && rect.contains(row.button.max), "{row:?}");
            assert!(rect.contains(row.what.min) && rect.contains(row.what.max), "{row:?}");
        }

        for index in 0..report.lines().len() {
            let line = report_rect(index);
            assert!(rect.contains(line.min) && rect.contains(line.max), "report line {index}");
        }
    }

    /// A click is read against the same rectangles the panel is drawn from.
    #[test]
    fn a_click_on_a_button_finds_the_action_it_names() {
        for row in save_rows() {
            assert_eq!(action_at(row.button.center()), Some(row.action));
        }

        assert_eq!(action_at(save_rows()[0].what.center()), None, "the panel swallows a click on the rest of a row");
        assert_eq!(action_at(Vec2::new(1200.0, 400.0)), None, "and has nothing to say about the play field");
    }


    /// The two halves of a round trip, without an editor in the way: what
    /// `save_level` writes is what `load_level` reads.
    #[test]
    fn a_level_written_here_reads_back_identical() {
        let dir = scratch("round_trip");
        let level = sparse("AA AAR3 ZA\n.. CA AI");

        save_level(&dir.join("level0.ron"), &level).unwrap();

        assert_eq!(load_level(&dir.join("level0.ron")).unwrap(), level);
    }
}
