//! Undo and redo.
//!
//! Every edit the editor makes goes through here, and every edit is *the two
//! levels either side of it* rather than a hand-written inverse. A
//! [`LevelDefinition`] is a token grid plus ten small fields, so the pair is
//! cheap - and it is the only form that is reversible without an argument: the
//! inverse of "take the top row away" has to carry the blocks that stood on it
//! anyway, which is a snapshot of that row under another name. One [`Edit`]
//! covers paints, erases, resizes and settings alike, and nothing downstream
//! has to know which of those it is looking at.
//!
//! A drag is one entry, not one per cell, because `c0007` already records a
//! stroke as a whole: [`finish_stroke`](super::finish_stroke) is what turns a
//! finished one into an [`Edit`], and everything else an author can do calls it
//! first - so a resize made mid-drag is its own entry rather than something the
//! drag swallows.
//!
//! What is on screen is the bar under the settings panel, laid out and
//! hit-tested against its own rectangles exactly as `c0010`'s panel is.

use bevy::asset::{AssetEvent, AssetServer, Assets};
use bevy::color::palettes::css::{DIM_GRAY, GOLD, WHITE};
use bevy::ecs::change_detection::DetectChangesMut;
use bevy::ecs::message::MessageReader;
use bevy::log::{info, warn};
use bevy::prelude::{ButtonInput, Color, Commands, Component, Entity, KeyCode, MouseButton, Query, Rect, Res, ResMut, Resource, Vec2, With};
use bevy::text::Justify;
use bevy::ui::{BackgroundColor, GlobalZIndex};
use bevy::window::{PrimaryWindow, Window};

use crate::level::asset::LevelAsset;
use crate::level::LevelDefinition;

use super::settings::{panel_node, panel_rect, panel_text, BUTTON_BACKGROUND, COLUMN_GAP, PANEL_BACKGROUND, PANEL_ORIGIN, PANEL_PADDING, PANEL_Z, ROW_FONT_SIZE, ROW_HEIGHT, ROW_INSET, ROW_WIDTH, TITLE_FONT_SIZE, TITLE_HEIGHT};
use super::save::LastSave;
use super::{commanding, finish_stroke, EditorEntity, EditorLevel, PaintStroke, PendingRemoval};

/// How far back the editor can go.
///
/// A cap rather than a promise of forever: an entry is a whole level, and an
/// afternoon of painting is tens of thousands of them. A hundred is more steps
/// than an author holds in their head, and the oldest are the ones nobody ever
/// walks back to.
const MAX_HISTORY: usize = 100;

/// One edit, as the level either side of it.
#[derive(Debug, Clone, PartialEq)]
pub struct Edit {
    /// What the author did, worded as the history bar says it - "painting 3
    /// cells".
    pub what: String,

    pub before: LevelDefinition,
    pub after: LevelDefinition,
}

/// Which way through the history a step goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryStep {
    Undo,
    Redo,
}

/// The steps, in the order they are on screen.
pub const HISTORY_STEPS: [HistoryStep; 2] = [HistoryStep::Undo, HistoryStep::Redo];

impl HistoryStep {
    /// What the step is called on screen.
    pub fn label(&self) -> &'static str {
        match self {
            HistoryStep::Undo => "Undo",
            HistoryStep::Redo => "Redo",
        }
    }

    /// The press that takes it, as the bar writes it.
    ///
    /// The short one of the two the editor answers to - `Ctrl+Shift+Z` redoes
    /// as well, and is what a hand coming from anywhere else reaches for, but
    /// it is twice as wide as the column it would have to fit in. `Ctrl` is
    /// what is written; [`step_asked_for`] takes the Mac's `Cmd` for it too.
    pub fn shortcut(&self) -> &'static str {
        match self {
            HistoryStep::Undo => "Ctrl+Z",
            HistoryStep::Redo => "Ctrl+Y",
        }
    }

    /// What the bar says when there is nothing this way to go to.
    fn nothing(&self) -> &'static str {
        match self {
            HistoryStep::Undo => "nothing to undo",
            HistoryStep::Redo => "nothing to redo",
        }
    }

    /// The level an edit leaves behind when it is stepped this way.
    fn side_of(&self, edit: &Edit) -> LevelDefinition {
        match self {
            HistoryStep::Undo => edit.before.clone(),
            HistoryStep::Redo => edit.after.clone(),
        }
    }
}

/// Everything the author has done to the level under edit, and everything they
/// have taken back.
///
/// Not cleared when the editor is left: [`EditorLevel`] survives a trip out to
/// the menu with its unsaved edits, and a history that did not survive with it
/// would leave those edits stuck. What *does* clear it is the level file
/// changing on disk underneath - see [`editor_watch_the_file`].
#[derive(Resource, Debug, Default)]
pub struct EditHistory {
    done: Vec<Edit>,
    undone: Vec<Edit>,
}

impl EditHistory {
    /// Puts an edit from `before` to `after` in, and says whether there was one
    /// at all.
    ///
    /// A new edit is what discards the redo stack: once the author has done
    /// something else, the branch they took back is no longer a branch this
    /// history could get to.
    pub fn record(&mut self, what: impl Into<String>, before: LevelDefinition, after: &LevelDefinition) -> bool {
        if before == *after {
            return false;
        }

        self.undone.clear();
        self.done.push(Edit { what: what.into(), before, after: after.clone() });

        if self.done.len() > MAX_HISTORY {
            self.done.remove(0);
        }

        true
    }

    /// The edit a step would take, if there is one that way.
    pub fn next(&self, step: HistoryStep) -> Option<&Edit> {
        match step {
            HistoryStep::Undo => self.done.last(),
            HistoryStep::Redo => self.undone.last(),
        }
    }

    /// Moves one edit from the done pile to the undone one, or back, and hands
    /// it over so the caller can put the level it names on screen.
    fn take(&mut self, step: HistoryStep) -> Option<Edit> {
        let (from, to) = match step {
            HistoryStep::Undo => (&mut self.done, &mut self.undone),
            HistoryStep::Redo => (&mut self.undone, &mut self.done),
        };

        let edit = from.pop()?;
        to.push(edit.clone());

        Some(edit)
    }

    pub fn clear(&mut self) {
        self.done.clear();
        self.undone.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.done.is_empty() && self.undone.is_empty()
    }

    /// How far the history reaches either way, as (done, undone).
    ///
    /// What the editor itself asks is [`EditHistory::next`] - the one edit a
    /// step would take. This is for saying "and there are three more behind it"
    /// in a test, which is not something anything on screen needs to know.
    #[cfg(test)]
    pub fn depth(&self) -> (usize, usize) {
        (self.done.len(), self.undone.len())
    }
}

/// Puts an edit in the history, and only tells the world about it if there was
/// one.
///
/// The bar on screen hangs off a change to the resource, as the settings panel
/// hangs off a change to the level: a click that edited nothing must not look
/// like an edit, or the whole bar is rebuilt every time an author lands on a
/// cell that already holds what the brush is set to.
pub fn remember(
    history: &mut ResMut<EditHistory>,
    what: impl Into<String>,
    before: LevelDefinition,
    after: &LevelDefinition,
) -> bool {
    let recorded = history.bypass_change_detection().record(what, before, after);

    if recorded {
        history.set_changed();
    }

    recorded
}


// --- taking a step --------------------------------------------------------

/// Walks the history from the keyboard.
pub fn editor_undo_redo(
    keys: Res<ButtonInput<KeyCode>>,
    mut stroke: ResMut<PaintStroke>,
    mut history: ResMut<EditHistory>,
    mut pending: ResMut<PendingRemoval>,
    mut editor_level: ResMut<EditorLevel>,
) {
    let Some(step) = step_asked_for(&keys) else { return; };

    take_a_step(step, &mut stroke, &mut history, &mut pending, &mut editor_level);
}

/// The step an author is asking for.
///
/// `Ctrl+Z` and `Ctrl+Shift+Z`, and `Cmd` for `Ctrl` on the Mac this game is
/// built on. `Ctrl+Y` is there because half the world's editors put redo on it
/// and the key is free.
fn step_asked_for(keys: &ButtonInput<KeyCode>) -> Option<HistoryStep> {
    if !commanding(keys) {
        return None;
    }

    if keys.just_pressed(KeyCode::KeyY) {
        return Some(HistoryStep::Redo);
    }

    if !keys.just_pressed(KeyCode::KeyZ) {
        return None;
    }

    Some(match keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        true => HistoryStep::Redo,
        false => HistoryStep::Undo,
    })
}

/// Walks the history from the bar on screen.
///
/// The press rather than the hold, as the settings panel reads its buttons: a
/// held mouse button would run the whole history in a fifth of a second.
pub fn editor_history_click(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut stroke: ResMut<PaintStroke>,
    mut history: ResMut<EditHistory>,
    mut pending: ResMut<PendingRemoval>,
    mut editor_level: ResMut<EditorLevel>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(cursor) = windows.iter().next().and_then(|window| window.cursor_position()) else { return; };
    let Some(step) = step_at(cursor) else { return; };

    take_a_step(step, &mut stroke, &mut history, &mut pending, &mut editor_level);
}

/// One step through the history, however it was asked for.
fn take_a_step(
    step: HistoryStep,
    stroke: &mut ResMut<PaintStroke>,
    history: &mut ResMut<EditHistory>,
    pending: &mut ResMut<PendingRemoval>,
    editor_level: &mut ResMut<EditorLevel>,
) {
    // Whatever the mouse is in the middle of painting is an edit of its own,
    // and it is the one this step steps back from - not something to be undone
    // together with the edit before it, and not something to be recorded again
    // afterwards from a level that has since moved.
    finish_stroke(stroke, history, &editor_level.level);

    let Some(edit) = history.bypass_change_detection().take(step) else {
        info!("{}", step.nothing());
        return;
    };

    history.set_changed();

    // An undo is not an answer to "remove this row?", and what the warning
    // counted is out of date the moment the level moves.
    pending.set_if_neq(PendingRemoval(None));

    editor_level.level = step.side_of(&edit);

    info!("{}: {}", step.label(), edit.what);
}


// --- the file underneath --------------------------------------------------

/// Drops the history when the level file changes on disk underneath the editor.
///
/// `c0004` hot-reloads level files, so the file the editor opened can be
/// rewritten by a hand edit - or by another copy of the game - while the editor
/// is up. The level under edit is left alone, because it is what the author has
/// been working on; the history is not, because every entry in it is a level
/// that was true of a file that no longer exists.
pub fn editor_watch_the_file(
    mut events: MessageReader<AssetEvent<LevelAsset>>,
    editor_level: Res<EditorLevel>,
    level_assets: Res<Assets<LevelAsset>>,
    last_save: Res<LastSave>,
    mut history: ResMut<EditHistory>,
) {
    let source = editor_level.source.as_ref().map(|handle| handle.id());

    // Counted rather than `any`, so that every message is read: one left behind
    // would be found again next frame and drop a history that has since been
    // built back up.
    let changed = events
        .read()
        .filter(|event| matches!(event, AssetEvent::Modified { id } if Some(*id) == source))
        .count()
        > 0;

    if !changed || history.is_empty() {
        return;
    }

    // The editor's own save comes back through the watcher exactly as a hand
    // edit does - `c0012` writes into the directory `c0004` is watching - and
    // the file system cannot tell us which it was. What it says can: a file that
    // holds what we last wrote to it is our own write coming home, and the
    // history is still every bit as true of it as it was a moment ago.
    if last_save.is_what_the_file_now_says(&editor_level, &level_assets) {
        return;
    }

    warn!(
        "{} changed on disk - the undo history is no longer about that file, so it has been dropped",
        editor_level.source_path().unwrap_or_else(|| "the level".to_string())
    );

    history.clear();
}


// --- the bar on screen ----------------------------------------------------

/// How far under the settings panel the bar sits.
const BAR_GAP: f32 = 8.0;

/// How wide the button that takes a step is - as little as "Undo" needs, so
/// that the rest of the row is free to say what the step would take back
/// without wrapping. "removing the bottom row" is the longest thing it has to
/// hold.
const STEP_WIDTH: f32 = 84.0;

/// Marks everything the bar draws, so a changed history can take the whole of it
/// down and put it up again saying the new thing.
#[derive(Component)]
pub struct HistoryBar;

/// What a step would take back, on screen, tagged with the step it belongs to.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HistoryEntry(pub HistoryStep);

/// One step's row, as rectangles on the window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HistoryRow {
    pub step: HistoryStep,
    pub button: Rect,
    pub what: Rect,
}

/// The bar's footprint: under the settings panel, in the same column and the
/// same width.
///
/// What the editor keeps its hands off, as it does the panel above it - a click
/// in here is aimed at the history, and the cell that happens to be underneath
/// is not what the author meant.
pub fn history_rect() -> Rect {
    let top = panel_rect().max.y + BAR_GAP;
    let height = PANEL_PADDING + TITLE_HEIGHT + HISTORY_STEPS.len() as f32 * ROW_HEIGHT + PANEL_PADDING;

    Rect::new(
        PANEL_ORIGIN.x,
        top,
        PANEL_ORIGIN.x + ROW_WIDTH + 2.0 * PANEL_PADDING,
        top + height,
    )
}

fn bar_title_rect() -> Rect {
    let left = history_rect().min.x + PANEL_PADDING;
    let top = history_rect().min.y + PANEL_PADDING;

    Rect::new(left, top, left + ROW_WIDTH, top + TITLE_HEIGHT)
}

/// The rows of the bar, laid out once and used twice: this is what the bar
/// draws, and what a click is read against.
pub fn history_rows() -> Vec<HistoryRow> {
    HISTORY_STEPS
        .iter()
        .enumerate()
        .map(|(index, step)| history_row(*step, index))
        .collect()
}

fn history_row(step: HistoryStep, index: usize) -> HistoryRow {
    let left = history_rect().min.x + PANEL_PADDING;
    let top = history_rect().min.y + PANEL_PADDING + TITLE_HEIGHT + index as f32 * ROW_HEIGHT;
    let bottom = top + ROW_HEIGHT;

    // The button is kept clear of the rows above and below it, so two of them
    // do not run into one another.
    let button = Rect::new(left, top + ROW_INSET, left + STEP_WIDTH, bottom - ROW_INSET);
    let what = Rect::new(button.max.x + COLUMN_GAP, top, left + ROW_WIDTH, bottom);

    HistoryRow { step, button, what }
}

/// The step a click at `pixel` is aimed at.
///
/// `None` for a click anywhere else, including inside the bar but not on a
/// button - the bar swallows those, it does not act on them.
pub fn step_at(pixel: Vec2) -> Option<HistoryStep> {
    history_rows()
        .into_iter()
        .find(|row| row.button.contains(pixel))
        .map(|row| row.step)
}

/// Puts the bar on screen, saying what the history holds right now.
///
/// The whole bar is rebuilt rather than the one row that moved, as the settings
/// panel is: six nodes is nothing, and this only runs on a frame the history
/// actually changed.
pub fn editor_show_history(
    history: Res<EditHistory>,
    shown: Query<Entity, With<HistoryBar>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for entity in &shown {
        commands.entity(entity).despawn();
    }

    commands.spawn((
        panel_node(history_rect()),
        BackgroundColor(PANEL_BACKGROUND),
        GlobalZIndex(PANEL_Z),
        HistoryBar,
        EditorEntity,
    ));

    // The shortcuts are said once, in the title, rather than on each button:
    // a button wide enough to hold its own shortcut is a button that leaves no
    // room for what it would take back.
    commands.spawn((
        panel_text(
            bar_title_rect(),
            &format!(
                "History - {} / {}",
                HistoryStep::Undo.shortcut(),
                HistoryStep::Redo.shortcut(),
            ),
            TITLE_FONT_SIZE,
            GOLD.into(),
            Justify::Left,
            &asset_server,
        ),
        HistoryBar,
        EditorEntity,
    ));

    for row in history_rows() {
        let next = history.next(row.step);

        // A step with nothing to take is drawn dimmed rather than left off: a
        // button that comes and goes is one an author has to look for, and the
        // shortcut it names is worth reading even when it would do nothing.
        let (button_colour, what_colour): (Color, Color) = match next {
            Some(_) => (GOLD.into(), WHITE.into()),
            None => (DIM_GRAY.into(), DIM_GRAY.into()),
        };

        commands.spawn((
            panel_text(row.button, row.step.label(), ROW_FONT_SIZE, button_colour, Justify::Center, &asset_server),
            BackgroundColor(BUTTON_BACKGROUND),
            HistoryBar,
            EditorEntity,
        ));

        commands.spawn((
            panel_text(
                row.what,
                next.map(|edit| edit.what.as_str()).unwrap_or_else(|| row.step.nothing()),
                ROW_FONT_SIZE,
                what_colour,
                Justify::Left,
                &asset_server,
            ),
            HistoryEntry(row.step),
            HistoryBar,
            EditorEntity,
        ));
    }
}
