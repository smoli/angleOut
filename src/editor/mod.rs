//! The level editor.
//!
//! [`GameState::Editor`] is a screen of its own, entered from the main menu and
//! left back to it with `Escape`. What it edits is [`EditorLevel`] - a resource
//! rather than a set of entities, so the level under edit survives the state
//! changes the playtest round trip (`c0013`) will put it through, and the editor
//! is found exactly as it was left.
//!
//! This is the room the editor lives in: the state, the way in and out, the
//! camera framing the play field and the mouse pointer that the game otherwise
//! hides. Picking cells (`c0006`) and painting them (`c0007`) go on top.

use std::f32::consts::FRAC_PI_2;

use bevy::app::{App, Plugin, Update};
use bevy::asset::Assets;
use bevy::camera::{OrthographicProjection, Projection, ScalingMode};
use bevy::color::palettes::css::{DIM_GRAY, GRAY};
use bevy::light::CascadeShadowConfigBuilder;
use bevy::log::info;
use bevy::prelude::{default, in_state, ButtonInput, Camera3d, Color, Commands, Component, DirectionalLight, Entity, Gizmos, Isometry3d, IntoScheduleConfigs, KeyCode, NextState, OnEnter, OnExit, Quat, Query, Rect, Res, ResMut, Resource, Transform, UVec2, Vec2, Vec3, With};
use bevy::window::{CursorOptions, PrimaryWindow};

use crate::config::{ARENA_HEIGHT, ARENA_WIDTH, BLOCK_DEPTH, BLOCK_GAP, BLOCK_WIDTH};
use crate::level::asset::LevelAsset;
use crate::level::layout::{empty_grid, grid_bounds, grid_dimensions};
use crate::level::TargetLayout::{Custom, FilledGrid, SparseGrid};
use crate::level::{LevelDefinition, Levels};
use crate::state::GameState;

/// How far above the play field the editor camera sits. Only the near and far
/// planes care - the projection is orthographic, so this does not change how
/// much of the world is on screen.
const EDITOR_CAMERA_HEIGHT: f32 = 200.0;

/// How much room the editor leaves around the play field, so the outermost
/// cells are not flush against the window edge.
const EDITOR_VIEW_MARGIN: f32 = BLOCK_WIDTH;

/// The size of the grid a level starts as when there is no level to open.
const NEW_LEVEL_COLS: usize = 9;
const NEW_LEVEL_ROWS: usize = 6;

/// The level the editor is working on.
///
/// Inserted the first time the editor is entered and never removed: leaving the
/// editor - for the menu, or for a playtest - has to find the level, unsaved
/// edits and all, on the way back in.
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct EditorLevel {
    /// The asset path the level was opened from, or `None` for a level that has
    /// never been on disk. This is where `c0012` will save it back to.
    pub source: Option<String>,

    pub level: LevelDefinition,
}

impl EditorLevel {
    /// A level with nothing in it yet - what "new level" means.
    pub fn blank() -> Self {
        EditorLevel {
            source: None,
            level: LevelDefinition {
                targets: SparseGrid(empty_grid(NEW_LEVEL_COLS, NEW_LEVEL_ROWS), BLOCK_GAP),
                ..default()
            },
        }
    }

    /// The grid being edited, as (columns, rows, gap).
    ///
    /// `None` for a level whose targets are not a grid at all - the `Custom`
    /// layouts are built in code, so there are no cells to show or paint.
    pub fn grid(&self) -> Option<(usize, usize, f32)> {
        match &self.level.targets {
            FilledGrid(cols, rows, _, _, gap) => Some((*cols, *rows, *gap)),
            SparseGrid(layout, gap) => {
                let (cols, rows) = grid_dimensions(layout);
                Some((cols, rows, *gap))
            }
            Custom(_) => None,
        }
    }
}

/// Marks everything the editor spawns, so leaving can take all of it with it.
#[derive(Component)]
pub struct EditorEntity;

/// The slice of the world the editor keeps on screen.
///
/// The arena, plus a margin: the block grid reaches a little past the arena's
/// near edge - row 0 is centred at z = -68, where the arena stops at -70 - and
/// grids grow rows from there towards the paddle.
pub fn editor_view() -> Rect {
    Rect::from_center_size(
        Vec2::ZERO,
        Vec2::new(ARENA_WIDTH, ARENA_HEIGHT) + 2.0 * Vec2::splat(EDITOR_VIEW_MARGIN),
    )
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                OnEnter(GameState::Editor),
                (editor_open, editor_setup, editor_show_cursor),
            )

            .add_systems(
                Update,
                (editor_leave, editor_draw_grid).run_if(in_state(GameState::Editor)),
            )

            .add_systems(
                OnExit(GameState::Editor),
                (editor_teardown, editor_hide_cursor),
            )
        ;
    }
}

/// Puts a level in front of the editor - once.
///
/// Every entry after the first finds [`EditorLevel`] already there and leaves it
/// alone, which is what makes a trip out to the menu (and, later, into a
/// playtest) non-destructive.
fn editor_open(
    editor_level: Option<Res<EditorLevel>>,
    levels: Res<Levels>,
    level_assets: Res<Assets<LevelAsset>>,
    mut commands: Commands,
) {
    if editor_level.is_some() {
        return;
    }

    commands.insert_resource(open_current_level(&levels, &level_assets));
}

/// The level the editor opens on: the one the campaign is pointing at, or a
/// blank grid when there is none to be had - an empty campaign, or a level file
/// that has not arrived from the asset server yet.
fn open_current_level(levels: &Levels, level_assets: &Assets<LevelAsset>) -> EditorLevel {
    let Some(level) = levels.get_current_level(level_assets) else {
        info!("No level to open - the editor starts on a blank grid");
        return EditorLevel::blank();
    };

    EditorLevel {
        source: levels
            .current_handle()
            .and_then(|handle| handle.path())
            .map(|path| path.to_string()),
        level: level.clone(),
    }
}

/// The editor's own camera and light.
///
/// Straight down and orthographic, so a cell is the same size wherever it is on
/// the grid and the mouse points at exactly the cell it looks like it points at.
/// `AutoMin` is what makes the framing a promise rather than a hope: whatever
/// the window's aspect ratio, at least [`editor_view`] is on screen.
fn editor_setup(mut commands: Commands) {
    let view = editor_view();

    commands.spawn((
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: view.width(),
                min_height: view.height(),
            },
            ..OrthographicProjection::default_3d()
        }),
        // The nudge off the axis keeps `looking_at` from having to make sense of
        // a view direction parallel to its own up vector.
        Transform::from_xyz(0.0, EDITOR_CAMERA_HEIGHT, 0.00001).looking_at(Vec3::ZERO, Vec3::Y),
        EditorEntity,
    ));

    const HALF_SIZE: f32 = 300.0;
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.7, 0.7, 1.0),
            shadow_maps_enabled: false,
            illuminance: 75_000.0 / 2.0,
            ..default()
        },
        CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 2.0 * HALF_SIZE,
            ..default()
        }
            .build(),
        Transform::from_xyz(0.0, 200.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
        EditorEntity,
    ));
}

/// Draws the cells of the level under edit, inside the frame the camera holds.
///
/// Gizmos rather than meshes: neither the cell grid nor the frame is part of the
/// level, and the grid changes shape every time a row or a column is added
/// (`c0008`). The frame is what shows that a grid still fits on screen.
fn editor_draw_grid(editor_level: Res<EditorLevel>, mut gizmos: Gizmos) {
    let Some((cols, rows, gap)) = editor_level.grid() else { return; };

    if cols == 0 || rows == 0 {
        return;
    }

    let centre = grid_bounds(cols, rows, gap).center();

    gizmos
        .grid(
            Isometry3d::new(
                Vec3::new(centre.x, 0.0, centre.y),
                Quat::from_rotation_x(FRAC_PI_2),
            ),
            UVec2::new(cols as u32, rows as u32),
            Vec2::new(BLOCK_WIDTH + gap, BLOCK_DEPTH + gap),
            DIM_GRAY,
        )
        // Gizmo grids leave their outer edges off, which would draw the grid one
        // cell short on every side.
        .outer_edges();

    let view = editor_view();
    gizmos.rect(
        Isometry3d::new(Vec3::ZERO, Quat::from_rotation_x(FRAC_PI_2)),
        view.size(),
        GRAY,
    );
}

/// Back to the menu.
///
/// `Escape` is the way out, which is why `close_on_esc` in `main.rs` stands down
/// while the editor is up - quitting the game outright is not what a level
/// author means by "back".
fn editor_leave(
    keys: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        game_state.set(GameState::InGame);
    }
}

fn editor_teardown(mut commands: Commands, editor_entities: Query<Entity, With<EditorEntity>>) {
    for entity in &editor_entities {
        commands.entity(entity).despawn();
    }
}

/// The game hides the pointer at startup (`primary_cursor_options` in
/// `main.rs`); an editor driven by the mouse needs it back.
fn editor_show_cursor(cursor: Query<&mut CursorOptions, With<PrimaryWindow>>) {
    set_cursor_visible(cursor, true);
}

fn editor_hide_cursor(cursor: Query<&mut CursorOptions, With<PrimaryWindow>>) {
    set_cursor_visible(cursor, false);
}

fn set_cursor_visible(mut cursor: Query<&mut CursorOptions, With<PrimaryWindow>>, visible: bool) {
    for mut options in &mut cursor {
        options.visible = visible;
    }
}


#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::fs;

    use super::*;

    use bevy::app::App;
    use bevy::asset::{AssetApp, AssetPlugin};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::gizmos::config::{DefaultGizmoConfigGroup, GizmoConfig, GizmoConfigStore};
    use bevy::input::InputPlugin;
    use bevy::prelude::State;
    use bevy::state::app::{AppExtStates, StatesPlugin};
    use bevy::MinimalPlugins;

    use crate::config::BLOCK_GAP;
    use crate::level::campaign;

    /// Just enough app to walk in and out of the editor: the states it hangs
    /// off, the asset collection it opens levels from, the keyboard it listens
    /// to, and a primary window with the pointer hidden - as `WindowPlugin`
    /// leaves it.
    fn editor_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), StatesPlugin, InputPlugin));
        // `Gizmos` reads the config store, and the editor draws its grid with
        // gizmos every frame. The rest of `GizmoPlugin` wants a renderer, which
        // a headless app has no use for.
        app.init_resource::<GizmoConfigStore>();
        app.world_mut()
            .resource_mut::<GizmoConfigStore>()
            .insert(GizmoConfig::default(), DefaultGizmoConfigGroup);
        app.init_asset::<LevelAsset>();
        app.insert_state(GameState::InGame);
        app.insert_resource(Levels { handles: vec![], current_level: 0 });
        app.add_plugins(EditorPlugin);

        app.world_mut().spawn((
            CursorOptions { visible: false, ..default() },
            PrimaryWindow,
        ));

        app
    }

    /// An app whose campaign holds one level, the way the game's does.
    fn app_playing(level: LevelDefinition) -> App {
        let mut app = editor_app();

        let handle = app.world_mut().resource_mut::<Assets<LevelAsset>>().add(LevelAsset(level));
        app.insert_resource(Levels { handles: vec![handle], current_level: 0 });

        app
    }

    fn sparse(layout: &str) -> LevelDefinition {
        LevelDefinition {
            targets: SparseGrid(layout.to_string(), BLOCK_GAP),
            ..default()
        }
    }

    fn go_to(app: &mut App, state: GameState) {
        app.world_mut().resource_mut::<NextState<GameState>>().set(state);
        app.update();
    }

    fn state(app: &App) -> GameState {
        *app.world().resource::<State<GameState>>().get()
    }

    fn cursor_is_visible(app: &mut App) -> bool {
        let world = app.world_mut();
        let mut cursors = world.query::<&CursorOptions>();
        cursors.iter(world).all(|options| options.visible)
    }

    fn editor_entities(app: &mut App) -> usize {
        let world = app.world_mut();
        let mut entities = world.query_filtered::<Entity, With<EditorEntity>>();
        entities.iter(world).count()
    }

    fn editor_level(app: &App) -> &EditorLevel {
        app.world().resource::<EditorLevel>()
    }

    #[test]
    fn the_pointer_is_there_while_the_editor_is_and_gone_again_afterwards() {
        let mut app = editor_app();
        assert!(!cursor_is_visible(&mut app), "the game hides the pointer");

        go_to(&mut app, GameState::Editor);
        assert!(cursor_is_visible(&mut app), "an editor driven by the mouse needs the pointer");

        go_to(&mut app, GameState::InGame);
        assert!(!cursor_is_visible(&mut app), "and the game wants it hidden again");
    }

    #[test]
    fn leaving_the_editor_takes_everything_it_spawned_with_it() {
        let mut app = editor_app();

        go_to(&mut app, GameState::Editor);
        assert!(editor_entities(&mut app) > 0, "the editor has to have spawned something");

        go_to(&mut app, GameState::InGame);
        assert_eq!(editor_entities(&mut app), 0);
    }

    #[test]
    fn escape_leaves_the_editor_for_the_menu() {
        let mut app = editor_app();
        go_to(&mut app, GameState::Editor);

        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Escape);

        // Run the system directly: `InputPlugin` clears `just_pressed` in
        // `PreUpdate`, so a key pressed from a test never survives to `Update`.
        app.world_mut().run_system_once(editor_leave).unwrap();
        app.update();

        assert_eq!(state(&app), GameState::InGame);
    }

    #[test]
    fn the_editor_opens_the_level_the_campaign_is_on() {
        let mut app = app_playing(sparse("AA AA AA"));

        go_to(&mut app, GameState::Editor);

        assert_eq!(editor_level(&app).level, sparse("AA AA AA"));
        assert_eq!(editor_level(&app).grid(), Some((3, 1, BLOCK_GAP)));
    }

    #[test]
    fn the_editor_starts_on_a_blank_grid_when_there_is_no_level_to_open() {
        let mut app = editor_app();

        go_to(&mut app, GameState::Editor);

        assert_eq!(editor_level(&app), &EditorLevel::blank());
        assert_eq!(editor_level(&app).source, None, "a new level has never been on disk");
        assert_eq!(editor_level(&app).grid(), Some((NEW_LEVEL_COLS, NEW_LEVEL_ROWS, BLOCK_GAP)));
    }

    /// The point of holding the level in a resource: `c0013` will send the
    /// editor through `InMatch` and back, and unsaved edits have to be waiting
    /// on the other side.
    #[test]
    fn the_level_under_edit_survives_a_trip_out_of_the_editor_and_back() {
        let mut app = app_playing(sparse("AA AA AA"));
        go_to(&mut app, GameState::Editor);

        let edited = sparse("AA .. AA\n .. AA ..");
        app.world_mut().resource_mut::<EditorLevel>().level = edited.clone();

        go_to(&mut app, GameState::InGame);
        go_to(&mut app, GameState::Editor);

        assert_eq!(
            editor_level(&app).level,
            edited,
            "re-entering the editor must not throw the edits away and re-open the file"
        );
    }

    /// The camera has to show at least [`editor_view`] whatever the window's
    /// aspect ratio is, which is exactly what `AutoMin` promises.
    #[test]
    fn the_editor_camera_frames_the_view() {
        let mut app = editor_app();
        go_to(&mut app, GameState::Editor);

        let view = editor_view();
        let world = app.world_mut();
        let mut cameras = world.query_filtered::<&Projection, With<EditorEntity>>();

        let projections: Vec<&Projection> = cameras.iter(world).collect();
        assert_eq!(projections.len(), 1, "the editor spawns exactly one camera");

        match projections[0] {
            Projection::Orthographic(ortho) => match ortho.scaling_mode {
                ScalingMode::AutoMin { min_width, min_height } => {
                    assert_eq!(Vec2::new(min_width, min_height), view.size());
                }
                other => panic!("the framing has to survive any window shape, got {other:?}"),
            },
            other => panic!("expected an orthographic editor camera, got {other:?}"),
        }
    }

    fn levels_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(campaign::LEVELS_DIR)
    }

    fn level_files() -> Vec<PathBuf> {
        let mut files: Vec<PathBuf> = fs::read_dir(levels_dir())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "ron"))
            .filter(|path| path.file_name().unwrap() != campaign::CAMPAIGN_FILE)
            .collect();
        files.sort();
        files
    }

    /// "The whole grid area visible" against the levels that actually exist:
    /// every shipped level, from the 3 wide one to the 11 wide one, has to fit
    /// in the frame the editor opens with.
    #[test]
    fn every_shipped_level_fits_in_the_editor_view() {
        let view = editor_view();
        let mut checked = 0;

        for path in level_files() {
            let editor_level = EditorLevel {
                source: None,
                level: campaign::load_level(&path).unwrap(),
            };

            let Some((cols, rows, gap)) = editor_level.grid() else { continue; };
            let bounds = grid_bounds(cols, rows, gap);

            assert!(
                view.contains(bounds.min) && view.contains(bounds.max),
                "{}: a {cols}x{rows} grid covers {bounds:?}, outside the editor's {view:?}",
                path.display()
            );

            checked += 1;
        }

        assert!(checked >= 8, "only {checked} level files were checked - is the directory right?");
    }
}
