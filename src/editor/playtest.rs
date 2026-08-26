//! Playing the level under edit, and coming back to it.
//!
//! The round trip is what makes the editor worth using rather than a thing you
//! save from and restart around, and it is made of three promises:
//!
//! **The match plays what is on screen, not what is on disk.** The level under
//! edit is handed to the asset collection as a level of its own, and
//! [`Levels::start_playtest`] points [`Levels::current_handle`] at it - which is
//! where the arena, the blocks and the win criteria all ask what they are
//! playing. Unsaved edits are therefore played by construction: nothing reads
//! the file. The campaign underneath - `handles` and `current_level` - is never
//! touched, so playtesting cannot move a player's place in the game.
//!
//! **The way home is the editor's own.** A playtest that ended, however it
//! ended, goes straight back to [`GameState::Editor`] rather than through
//! `PostMatch` and on to the next level. [`EditorLevel`] is a resource and was
//! never in the entities the match despawned, so it is still there when the
//! editor comes back up, unsaved edits and all.
//!
//! **The stage is cleared on the way in.** The game takes a match apart at
//! `OnExit(PostMatch)`, because the results screen is shown over the match that
//! produced it. A playtest never goes near `PostMatch`, so [`playtest_teardown`]
//! is the editor doing that job itself - everything a match puts on the table,
//! named in one place.

use bevy::asset::{AssetServer, Assets};
use bevy::color::palettes::css::{GOLD, SILVER};
use bevy::log::info;
use bevy::prelude::{ButtonInput, Commands, Component, Entity, KeyCode, MessageWriter, MouseButton, NextState, Or, Query, Rect, Res, ResMut, Resource, With};
use bevy::text::Justify;
use bevy::ui::{BackgroundColor, GlobalZIndex};
use bevy::window::{PrimaryWindow, Window};

use crate::arena::Arena;
use crate::ball::Ball;
use crate::block::Block;
use crate::events::GameFlowEvent;
use crate::level::asset::LevelAsset;
use crate::level::Levels;
use crate::pickups::Pickup;
use crate::player::{Player, PlayerState};
use crate::points::PointsDisplay;
use crate::ship::Ship;
use crate::state::GameState;
use crate::ui::stats::MatchStatsUI;
use crate::ui::Environment3d;

use super::save::{save_rect, SaveReport};
use super::settings::{panel_node, panel_text, BUTTON_BACKGROUND, COLUMN_GAP, PANEL_BACKGROUND, PANEL_ORIGIN, PANEL_PADDING, PANEL_Z, ROW_FONT_SIZE, ROW_HEIGHT, ROW_INSET, ROW_WIDTH, TITLE_FONT_SIZE, TITLE_HEIGHT};
use super::{EditorEntity, EditorLevel};

/// The hand a playtest deals the player.
///
/// The same three balls `game_start` deals a new game, because an author
/// playing their own level is playing the game - and dealt again for every
/// playtest, so the second one does not start on the empty hand the first one
/// ended with.
const PLAYTEST_BALLS: i32 = 3;

/// The key that starts one, as every editor with a game behind it spells it.
const PLAYTEST_KEY: KeyCode = KeyCode::F5;

/// How the last playtest ended, so the editor can say so when it comes back up.
///
/// Read off the player rather than reported by the match: winning and losing
/// already leave [`PlayerState`] behind them, and a playtest the author walked
/// out of leaves it where the start of the match set it.
#[derive(Resource, Debug, Default, PartialEq, Eq)]
pub struct LastPlaytest(pub Option<PlaytestEnd>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaytestEnd {
    Won,
    Lost,

    /// The author stopped playing and came back to work.
    Left,
}

impl PlaytestEnd {
    /// How the panel puts it.
    fn message(&self) -> &'static str {
        match self {
            PlaytestEnd::Won => "the last playtest was won",
            PlaytestEnd::Lost => "the last playtest was lost",
            PlaytestEnd::Left => "back from the last playtest",
        }
    }

    /// What the player was left in when the playtest ended.
    fn of(state: PlayerState) -> Self {
        match state {
            PlayerState::HasWon => PlaytestEnd::Won,
            PlayerState::HasLost => PlaytestEnd::Lost,
            PlayerState::Open => PlaytestEnd::Left,
        }
    }
}

/// Whether the game is playing a level out of the editor rather than out of the
/// campaign.
///
/// A run condition rather than a check inside the systems that want it, because
/// one of them lives in `main.rs`: `Escape` is the way out of a playtest, and it
/// is also what quits the game.
pub fn playtesting(levels: Res<Levels>) -> bool {
    levels.is_playtesting()
}


// --- there ----------------------------------------------------------------

/// Starts a match on the level as it stands.
///
/// The level is *copied* into the asset collection: the match must not be able
/// to reach back into the editor's own level, and an author who paints while a
/// ball is in the air is editing the next playtest rather than this one.
fn start_playtest(
    editor_level: &EditorLevel,
    levels: &mut Levels,
    level_assets: &mut Assets<LevelAsset>,
    players: &mut Query<&mut Player>,
    game_flow: &mut MessageWriter<GameFlowEvent>,
) {
    let handle = level_assets.add(LevelAsset(editor_level.level.clone()));
    levels.start_playtest(handle);

    // The hand the menu would have dealt. A playtest is not the campaign, so
    // whatever the player was carrying - points, balls, the state a previous
    // playtest ended in - is not what this one starts on.
    if let Ok(mut player) = players.single_mut() {
        player.reset();
        player.balls_available = PLAYTEST_BALLS;
    }

    // Through the normal start, so that a playtest waits for the same things a
    // match does and arrives in `InMatch` by the same road.
    game_flow.write(GameFlowEvent::StartMatch);

    info!("playtesting the level under edit");
}

/// [`PLAYTEST_KEY`], from the keyboard.
pub fn editor_playtest_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    editor_level: Res<EditorLevel>,
    mut levels: ResMut<Levels>,
    mut level_assets: ResMut<Assets<LevelAsset>>,
    mut players: Query<&mut Player>,
    mut game_flow: MessageWriter<GameFlowEvent>,
) {
    if !keys.just_pressed(PLAYTEST_KEY) {
        return;
    }

    start_playtest(&editor_level, &mut levels, &mut level_assets, &mut players, &mut game_flow);
}

/// The same, from the panel's button.
pub fn editor_playtest_click(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    report: Res<SaveReport>,
    editor_level: Res<EditorLevel>,
    mut levels: ResMut<Levels>,
    mut level_assets: ResMut<Assets<LevelAsset>>,
    mut players: Query<&mut Player>,
    mut game_flow: MessageWriter<GameFlowEvent>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(cursor) = windows.iter().next().and_then(|window| window.cursor_position()) else { return; };

    if !playtest_row(&report).button.contains(cursor) {
        return;
    }

    start_playtest(&editor_level, &mut levels, &mut level_assets, &mut players, &mut game_flow);
}


// --- and back again -------------------------------------------------------

/// `Escape` out of a playtest, back to the level it was made of.
///
/// The same key that leaves the editor for the menu, one level down: it is what
/// the author already knows means "I am done here". `close_on_esc` stands down
/// for the length of a playtest so that the two are not one press - see
/// [`playtesting`].
pub fn playtest_leave(
    keys: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        game_state.set(GameState::Editor);
    }
}

/// Hands the campaign back, and remembers how the playtest went.
///
/// Runs on the way *into* the editor rather than out of the match, so that a
/// playtest is over exactly when the editor is up - there is no frame in
/// between in which the game is neither playing nor editing.
///
/// Dropping the handle is what takes the level the playtest was made of out of
/// the asset collection: it was a copy of the level under edit, made for one
/// match, and nothing is coming back to it.
pub fn playtest_end(
    mut levels: ResMut<Levels>,
    players: Query<&Player>,
    mut last: ResMut<LastPlaytest>,
) {
    let Some(_played) = levels.stop_playtest() else { return; };

    let end = players.single().map_or(PlaytestEnd::Left, |player| PlaytestEnd::of(player.state));
    info!("{}", end.message());

    last.0 = Some(end);
}

/// Clears the match away on the way into the editor.
///
/// Everything a match puts on the table, named in one place, because the game's
/// own teardown happens at `OnExit(PostMatch)` and a playtest never goes there.
/// Two things a match leaves behind are missing on purpose: the `Match` marker
/// itself and the particle effects go at `OnExit(InMatch)`, which a playtest
/// does pass through.
///
/// Nothing the editor spawns is in here - the editor's blocks are
/// [`EditorBlock`](super::EditorBlock)s and its panels are
/// [`EditorEntity`](super::EditorEntity)s - so entering the editor from the
/// menu, with no match to clear, despawns nothing.
pub fn playtest_teardown(
    leftovers: Query<
        Entity,
        Or<(
            With<Arena>,
            With<Ship>,
            With<Ball>,
            With<Block>,
            With<Pickup>,
            With<PointsDisplay>,
            With<MatchStatsUI>,
            With<Environment3d>,
        )>,
    >,
    mut commands: Commands,
) {
    for leftover in &leftovers {
        commands.entity(leftover).despawn();
    }
}


// --- the panel ------------------------------------------------------------

/// How far under the file panel the playtest panel sits.
const PANEL_GAP: f32 = 8.0;

/// How wide the button that does the thing is - the same as the file panel's,
/// so the column reads as a column.
const BUTTON_WIDTH: f32 = 84.0;

/// Marks everything the panel draws, so a new one can replace the whole of it.
#[derive(Component)]
pub struct PlaytestPanel;

/// The panel's one row, as rectangles on the window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaytestRow {
    pub button: Rect,
    pub what: Rect,
}

/// The panel's footprint: under the file panel, in the same column and the same
/// width.
///
/// Takes the report because the panel above it grows downwards with what the
/// last save had to say, and this one sits under whatever height that came to.
///
/// What the editor keeps its hands off, as it does the three panels above it - a
/// click in here is aimed at the playtest, and the cell that happens to be
/// underneath is not what the author meant.
pub fn playtest_rect(report: &SaveReport) -> Rect {
    let top = save_rect(report).max.y + PANEL_GAP;
    let height = PANEL_PADDING + TITLE_HEIGHT + ROW_HEIGHT + PANEL_PADDING;

    Rect::new(
        PANEL_ORIGIN.x,
        top,
        PANEL_ORIGIN.x + ROW_WIDTH + 2.0 * PANEL_PADDING,
        top + height,
    )
}

fn panel_title_rect(report: &SaveReport) -> Rect {
    let left = playtest_rect(report).min.x + PANEL_PADDING;
    let top = playtest_rect(report).min.y + PANEL_PADDING;

    Rect::new(left, top, left + ROW_WIDTH, top + TITLE_HEIGHT)
}

/// The row, laid out once and used twice: this is what the panel draws, and
/// what a click is read against.
pub fn playtest_row(report: &SaveReport) -> PlaytestRow {
    let left = playtest_rect(report).min.x + PANEL_PADDING;
    let top = playtest_rect(report).min.y + PANEL_PADDING + TITLE_HEIGHT;
    let bottom = top + ROW_HEIGHT;

    PlaytestRow {
        button: Rect::new(left, top + ROW_INSET, left + BUTTON_WIDTH, bottom - ROW_INSET),
        what: Rect::new(left + BUTTON_WIDTH + COLUMN_GAP, top, left + ROW_WIDTH, bottom),
    }
}

/// What the row says next to its button: how the last playtest went, or what
/// pressing it would do when there has not been one yet.
fn what(last: &LastPlaytest) -> &'static str {
    match last.0 {
        Some(end) => end.message(),
        None => "play the level as it stands",
    }
}

/// Puts the panel on screen.
pub fn editor_show_playtest(
    report: Res<SaveReport>,
    last: Res<LastPlaytest>,
    shown: Query<Entity, With<PlaytestPanel>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for entity in &shown {
        commands.entity(entity).despawn();
    }

    commands.spawn((
        panel_node(playtest_rect(&report)),
        BackgroundColor(PANEL_BACKGROUND),
        GlobalZIndex(PANEL_Z),
        PlaytestPanel,
        EditorEntity,
    ));

    // The shortcut is said in the title, as the two panels above say theirs.
    commands.spawn((
        panel_text(panel_title_rect(&report), "Playtest - F5", TITLE_FONT_SIZE, GOLD.into(), Justify::Left, &asset_server),
        PlaytestPanel,
        EditorEntity,
    ));

    let row = playtest_row(&report);

    commands.spawn((
        panel_text(row.button, "Play", ROW_FONT_SIZE, GOLD.into(), Justify::Center, &asset_server),
        BackgroundColor(BUTTON_BACKGROUND),
        PlaytestPanel,
        EditorEntity,
    ));

    commands.spawn((
        panel_text(row.what, what(&last), ROW_FONT_SIZE, SILVER.into(), Justify::Left, &asset_server),
        PlaytestPanel,
        EditorEntity,
    ));
}


#[cfg(test)]
mod tests {
    use super::*;

    use crate::editor::save::{Complaint, Report};

    /// A report with `lines` lines in it, which is what makes the file panel
    /// above this one grow.
    fn report_of(complaints: usize) -> SaveReport {
        SaveReport(Some(Report {
            outcome: "Saved levels/level0.ron".to_string(),
            failed: false,
            complaints: vec![Complaint::NothingToBreak; complaints],
        }))
    }

    /// The panel is the bottom of the editor's column, and it has to stay under
    /// the panel above it however much that one has to say.
    #[test]
    fn the_panel_sits_under_the_file_panel_whatever_the_report_says() {
        for complaints in [0, 1, 5] {
            let report = report_of(complaints);

            assert!(
                playtest_rect(&report).min.y >= save_rect(&report).max.y,
                "the panels overlap with {complaints} complaints on screen",
            );
        }
    }

    /// A report that grows takes the panel down with it, rather than the panel
    /// staying put and being written over.
    #[test]
    fn the_panel_moves_down_as_the_report_grows() {
        assert!(playtest_rect(&report_of(3)).min.y > playtest_rect(&report_of(0)).min.y);
    }

    /// Everything a click can be aimed at is inside what the editor keeps its
    /// hands off - a button hanging over the edge would be a button that paints
    /// the cell behind it.
    #[test]
    fn the_button_is_inside_the_panel() {
        let report = report_of(0);
        let panel = playtest_rect(&report);
        let row = playtest_row(&report);

        for corner in [row.button.min, row.button.max, row.what.min, row.what.max] {
            assert!(panel.contains(corner), "{corner:?} is outside {panel:?}");
        }
    }

    #[test]
    fn the_row_says_how_the_last_playtest_went() {
        assert_eq!(what(&LastPlaytest(None)), "play the level as it stands");
        assert_eq!(what(&LastPlaytest(Some(PlaytestEnd::Won))), "the last playtest was won");
        assert_eq!(what(&LastPlaytest(Some(PlaytestEnd::Lost))), "the last playtest was lost");
        assert_eq!(what(&LastPlaytest(Some(PlaytestEnd::Left))), "back from the last playtest");
    }

    /// How a playtest ended is read off the player, because winning and losing
    /// leave their mark there and walking out leaves the player as the start of
    /// the match set them.
    #[test]
    fn the_player_says_how_the_playtest_ended() {
        assert_eq!(PlaytestEnd::of(PlayerState::HasWon), PlaytestEnd::Won);
        assert_eq!(PlaytestEnd::of(PlayerState::HasLost), PlaytestEnd::Lost);
        assert_eq!(PlaytestEnd::of(PlayerState::Open), PlaytestEnd::Left);
    }
}
