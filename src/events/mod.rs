use bevy::app::{App, Plugin, Update};
use bevy::log::error;
use bevy::prelude::{in_state, Assets, AssetServer, Commands, Entity, IntoScheduleConfigs, Local, Message, MessageReader, MessageWriter, NextState, Query, Res, ResMut, Vec3};
use crate::ball::Ball;
use crate::block::{BlockBehaviour, BlockType};
use crate::labels::SystemLabels;
use crate::level::asset::LevelAsset;
use crate::level::{LevelReadiness, Levels, RequestTag, WinCriteria};
use crate::pickups::{Pickup, PickupType};
use crate::player::{Player, PlayerState};
use crate::points::{PointsDisplay, PointsDisplayRequest};
use crate::powerups::{Bouncer, PowerUpData};
use crate::r#match::state::MatchState;
use crate::state::GameState;

#[derive(Message, Debug)]
pub enum GameFlowEvent {
    StartGame,
    StartMatch,

    /// Open the level editor. Nothing to wait for - the editor opens whatever
    /// level it can get and copes with there being none.
    OpenEditor,

    PlayerWins,
    PlayerLooses,

    NextLevel,

    EndGame,
}

#[derive(Message)]
pub enum MatchEvent {
    Start,
    BallSpawned,
    BallLaunched,
    BallGrabbed,
    BallLost,
    BlockLost,
    BounceOffPaddle,
    BounceOffWall,
    BlockHit(Vec3, BlockType, BlockBehaviour),
    PickedUp(PickupType),
}


pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_message::<MatchEvent>()
            .add_message::<GameFlowEvent>()
            .add_systems(Update, game_flow_handler)
            .add_systems(
                Update,
                match_event_handler
                    .after(SystemLabels::UpdateWorld)
                    .in_set(SystemLabels::UpdateState)
                    .run_if(in_state(GameState::InMatch)),
            )

        ;
    }
}


#[derive(PartialEq, Debug)]
enum LevelEndState {
    Won,
    Lost,
    Undecided,
}


fn check_win_criteria(
    win_criteria: &WinCriteria,
    player: &Player,
    stats: &MatchState,
) -> LevelEndState {

    if stats.blocks == 0 {
        match win_criteria {
            WinCriteria::BlockHitPercentage(pct) => {

                let result = (stats.blocks_hit as f32) / (stats.blocks_hit as f32 + stats.blocks_lost as f32);
                return if result >= *pct {
                    LevelEndState::Won
                } else {
                    LevelEndState::Lost
                }
            }
        };
    }

    if player.total_ball_count() == 0 && stats.blocks > 0 {
        return LevelEndState::Lost;
    }

    LevelEndState::Undecided
}

fn match_event_handler(
    mut commands: Commands,
    mut events: MessageReader<MatchEvent>,
    mut match_state: ResMut<MatchState>,
    mut players: Query<(Entity, &mut Player, &mut Bouncer)>,
    levels: Res<Levels>,
    level_assets: Res<Assets<LevelAsset>>,
    mut game_flow: MessageWriter<GameFlowEvent>,
) {
    let Ok((player_entity, mut player, mut bouncer)) = players.single_mut() else { return; };

    let Some(level) = levels.get_current_level(&level_assets) else { return; };

    for ev in events.read() {
        match ev {
            MatchEvent::Start => {
                match_state.reset();
            }

            MatchEvent::BallSpawned => {
                //info!("Executing ball spawn request");
                if player.balls_available > 0 && player.balls_carried == 0 && player.balls_in_play < level.simultaneous_balls {
                    commands
                        .spawn(Ball::default())
                        .insert(RequestTag)
                    ;
                    player.ball_spawned();
                }
            }

            MatchEvent::BallLaunched => {
                player.ball_launched();
            }

            MatchEvent::BallLost => {
                //info!("Ball Lost");
                player.ball_lost();
                match_state.ball_lost();
            }

            MatchEvent::BounceOffPaddle => {
                match_state.add_paddle_bounce();
                if !bouncer.available() {
                    game_flow.write(GameFlowEvent::PlayerLooses);
                } else {
                    bouncer.use_one();
                }
            }

            MatchEvent::BounceOffWall => {
                match_state.add_wall_hit();
            }

            MatchEvent::BlockHit(p, block_type, behaviour) => {
                let (_, awarded) = match_state.add_block_hit(block_type, behaviour);

                commands.spawn(PointsDisplay {
                    text: awarded.to_string(),
                    position: p.clone(),
                }).insert(PointsDisplayRequest);
            }

            MatchEvent::BlockLost => {
                match_state.block_lost();
            }

            MatchEvent::BallGrabbed => {
                player.ball_grabbed();
            }

            MatchEvent::PickedUp(pt) => {
                commands.entity(player_entity)
                    .insert(Pickup {
                        spawn_position: Default::default(),
                        pickup_type: *pt,
                    });

                //info!("Player picked up {:?}", pt)
            }
        }

        match check_win_criteria(&level.win_criteria, &player, &match_state) {
            LevelEndState::Won => { game_flow.write(GameFlowEvent::PlayerWins); }
            LevelEndState::Lost => { game_flow.write(GameFlowEvent::PlayerLooses); }
            LevelEndState::Undecided => {}
        }
    }
}

/// Drives the state machine off [`GameFlowEvent`]s.
///
/// `StartMatch` is the one transition that has to wait for something: the levels
/// come off the asset server now, so on the very first match the file may still
/// be in flight. Holding the transition for a frame or two is what keeps a match
/// from starting in front of a level that has not arrived - see
/// [`Levels::readiness`].
///
/// Winning and losing have two ends each, because a playtest is not a match of
/// the campaign: it hands the player no points and goes home to the editor,
/// where a campaign match banks them and goes on to the results screen. The
/// player's *state* is set either way - it is how
/// [`playtest`](crate::editor::playtest) knows how the level it was just handed
/// back went.
fn game_flow_handler(
    mut players: Query<&mut Player>,
    mut events: MessageReader<GameFlowEvent>,
    match_state: ResMut<MatchState>,
    levels: Res<Levels>,
    level_assets: Res<Assets<LevelAsset>>,
    asset_server: Res<AssetServer>,
    mut match_pending: Local<bool>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for ev in events.read() {
        match ev {
            GameFlowEvent::StartGame => {
                game_state.set(GameState::InGame);
            }

            GameFlowEvent::StartMatch => {
                *match_pending = true;
            }

            GameFlowEvent::OpenEditor => {
                game_state.set(GameState::Editor);
            }

            GameFlowEvent::PlayerWins => {
                if let Ok(mut player) = players.single_mut() {
                    //info!("Player wins!");
                    player.state = PlayerState::HasWon;

                    if levels.is_playtesting() {
                        game_state.set(GameState::Editor);
                    } else {
                        player.player_has_won(match_state.points);
                        //info!("Player now has {} points", player.points);
                        game_state.set(GameState::PostMatch);
                    }
                };
            }

            GameFlowEvent::NextLevel => {
                game_state.set(GameState::NextLevel);
            }

            GameFlowEvent::PlayerLooses => {
                if let Ok(mut player) = players.single_mut() {
                    //info!("Player looses!");
                    player.state = PlayerState::HasLost;

                    game_state.set(match levels.is_playtesting() {
                        true => GameState::Editor,
                        false => GameState::PostMatch,
                    });
                };
            }

            GameFlowEvent::EndGame => {}
        }
    };

    if *match_pending {
        match levels.readiness(&level_assets, &asset_server) {
            LevelReadiness::Loading => {}

            LevelReadiness::Ready => {
                *match_pending = false;
                game_state.set(GameState::InMatch);
            }

            // A level file that is missing or will not parse used to be a
            // startup panic. Now that it can also be a typo made in a text
            // editor while the game runs, fall back to the menu instead: the
            // asset server keeps watching the file, so fixing it and pressing
            // start again works, where an empty match would just be stuck.
            LevelReadiness::Unavailable => {
                error!(
                    "Level {} could not be loaded - back to the menu",
                    levels.current_level
                );
                *match_pending = false;
                game_state.set(GameState::InGame);
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use bevy::app::App;
    use bevy::asset::{AssetApp, AssetPlugin};
    use bevy::prelude::{default, Handle, State};
    use bevy::state::app::{AppExtStates, StatesPlugin};
    use bevy::MinimalPlugins;

    use crate::level::{LevelDefinition, WinCriteria};
    use crate::player::Player;
    use crate::r#match::state::MatchState;

    // --- where a match ends up --------------------------------------------

    /// Just enough app to drive the state machine: the states, the level
    /// collection the handler reads what is being played out of, and a player
    /// for it to hand the result to.
    fn flow_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), StatesPlugin));
        app.init_asset::<LevelAsset>();
        app.insert_state(GameState::InMatch);
        app.insert_resource(MatchState { points: 500, ..default() });
        app.add_plugins(EventsPlugin);
        app.world_mut().spawn(Player { balls_available: 1, ..default() });

        app
    }

    /// A campaign of one level, and the handle of a level of the editor's own
    /// standing beside it.
    fn levels_of(app: &mut App) -> (Levels, Handle<LevelAsset>) {
        let mut assets = app.world_mut().resource_mut::<Assets<LevelAsset>>();
        let campaign = assets.add(LevelAsset(LevelDefinition::default()));
        let under_edit = assets.add(LevelAsset(LevelDefinition::default()));

        (Levels { handles: vec![campaign], current_level: 0, ..default() }, under_edit)
    }

    fn tell_the_game(app: &mut App, event: GameFlowEvent) {
        app.world_mut().write_message(event);

        // One frame for the handler to read it, one for the transition to land.
        app.update();
        app.update();
    }

    fn state(app: &App) -> GameState {
        *app.world().resource::<State<GameState>>().get()
    }

    fn player(app: &mut App) -> &Player {
        let world = app.world_mut();
        let mut players = world.query::<&Player>();

        players.iter(world).next().expect("the app spawned one")
    }

    /// `c0013`: winning a playtest is not winning a match of the campaign. It
    /// goes home to the editor rather than on to the results screen, and it
    /// banks nothing on the way.
    #[test]
    fn winning_a_playtest_goes_back_to_the_editor_with_nothing_banked() {
        let mut app = flow_app();
        let (mut levels, under_edit) = levels_of(&mut app);
        levels.start_playtest(under_edit);
        app.insert_resource(levels);

        tell_the_game(&mut app, GameFlowEvent::PlayerWins);

        assert_eq!(state(&app), GameState::Editor);
        assert_eq!(player(&mut app).points, 0, "a playtest is not worth points");
        assert_eq!(player(&mut app).state, PlayerState::HasWon, "but it is worth knowing");
    }

    #[test]
    fn losing_a_playtest_goes_back_to_the_editor_too() {
        let mut app = flow_app();
        let (mut levels, under_edit) = levels_of(&mut app);
        levels.start_playtest(under_edit);
        app.insert_resource(levels);

        tell_the_game(&mut app, GameFlowEvent::PlayerLooses);

        assert_eq!(state(&app), GameState::Editor);
        assert_eq!(player(&mut app).state, PlayerState::HasLost);
    }

    /// And the flow the campaign has always taken is untouched by any of it.
    #[test]
    fn winning_a_match_of_the_campaign_still_goes_to_the_results_screen() {
        let mut app = flow_app();
        let (levels, _) = levels_of(&mut app);
        app.insert_resource(levels);

        tell_the_game(&mut app, GameFlowEvent::PlayerWins);

        assert_eq!(state(&app), GameState::PostMatch);
        assert_eq!(player(&mut app).points, 500, "the match is banked");
    }

    #[test]
    fn losing_a_match_of_the_campaign_still_goes_to_the_results_screen() {
        let mut app = flow_app();
        let (levels, _) = levels_of(&mut app);
        app.insert_resource(levels);

        tell_the_game(&mut app, GameFlowEvent::PlayerLooses);

        assert_eq!(state(&app), GameState::PostMatch);
    }

    /// A playtest is played on the editor's level, so the match starts on it -
    /// the handler waits for a level to be ready, and the one the editor handed
    /// over is ready the moment it is handed over.
    #[test]
    fn a_playtest_starts_a_match_on_the_level_it_was_handed() {
        let mut app = flow_app();
        let (mut levels, under_edit) = levels_of(&mut app);
        levels.start_playtest(under_edit);
        app.insert_resource(levels);

        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Editor);
        app.update();

        tell_the_game(&mut app, GameFlowEvent::StartMatch);

        assert_eq!(state(&app), GameState::InMatch);
    }

    #[test]
    fn just_loosing_a_ball() {
        let stats = MatchState {
            blocks: 1,
            blocks_hit: 1,
            blocks_lost: 1,
            ..Default::default()
        };

        let player = Player {
            balls_available: 1,
            balls_lost: 1,
            ..Default::default()
        };

        let crit = WinCriteria::BlockHitPercentage(1.0);

        // A block still stands and the player still has a ball, so the level is not over.
        assert_eq!(check_win_criteria(&crit, &player, &stats), LevelEndState::Undecided);
    }
}
