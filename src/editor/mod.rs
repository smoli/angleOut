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
//! hides - plus the cell the pointer is over, which is what makes the grid
//! something you can aim at. Painting it (`c0007`) goes on top.
//!
//! A level is more than its grid, and the rest of it lives in
//! [`settings`] - the panel in the corner that a click steps a setting on.

use std::f32::consts::FRAC_PI_2;

use bevy::app::{App, Plugin, Update};
use bevy::asset::{AssetServer, Assets};
use bevy::camera::{OrthographicProjection, Projection, ScalingMode};
use bevy::color::palettes::css::{DIM_GRAY, GRAY, ORANGE_RED, YELLOW};
use bevy::ecs::change_detection::DetectChangesMut;
use bevy::gltf::{Gltf, GltfMesh};
use bevy::light::CascadeShadowConfigBuilder;
use bevy::pbr::MeshMaterial3d;
use bevy::log::{info, warn};
use bevy::prelude::{default, in_state, resource_changed, resource_exists_and_changed, ButtonInput, Camera, Camera3d, Color, Commands, Component, DirectionalLight, Entity, Gizmos, GlobalTransform, InfinitePlane3d, Isometry3d, IntoScheduleConfigs, KeyCode, Mesh3d, MouseButton, NextState, Node, OnEnter, OnExit, PositionType, Quat, Query, Ray3d, Rect, Res, ResMut, Resource, Text, TextColor, TextFont, Transform, UVec2, Val, Vec2, Vec3, Visibility, With, Without};
use bevy::text::FontSize;
use bevy::window::{CursorOptions, PrimaryWindow, Window};

use crate::block::trigger::{TriggerGroup, TriggerType};
use crate::block::{block_material, Block, BlockBehaviour, BlockType, BLOCK_MESH};
use crate::config::{ARENA_HEIGHT, ARENA_WIDTH, BLOCK_DEPTH, BLOCK_GAP, BLOCK_WIDTH};
use crate::level::asset::LevelAsset;
use crate::level::layout::{block_token, blocks_on_edge, can_shrink, cell_to_world, empty_grid, filled_grid, grid_bounds, grid_dimensions, grow, interpret_grid, set_cell, shrink, world_to_cell, Edge, EMPTY_SLOT};
use crate::level::TargetLayout::{Custom, FilledGrid, SparseGrid};
use crate::level::{LevelDefinition, Levels, TargetLayout};
use crate::editor::settings::{panel_rect, setting_at, spawn_settings_panel, SettingsPanel};
use crate::materials::block::BlockMaterial;
use crate::state::GameState;
use crate::MyAssetPack;

pub mod settings;

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

/// How far above the ground plane the hover highlight is drawn, so it does not
/// fight the grid gizmo underneath it for the same pixels.
const HOVER_LIFT: f32 = 0.1;

/// The same, for the edge an author has been warned about - above the hover, so
/// the cell being pointed at still shows through when it is one of them.
const WARNING_LIFT: f32 = 0.2;

/// The font the editor writes in - the game's own.
const EDITOR_FONT: &str = "fonts/Orbitron-Regular.ttf";

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

    /// The blocks the level under edit is made of, wherever they come from.
    ///
    /// A `FilledGrid` is read through the token grid that says the same thing,
    /// so there is one path from a level to its blocks rather than one per
    /// layout kind. A `Custom` level is built in code and has none to show.
    pub fn blocks(&self) -> Vec<Block> {
        match &self.level.targets {
            SparseGrid(layout, gap) => interpret_grid(layout, *gap),

            FilledGrid(cols, rows, block_type, behaviour, gap) => {
                interpret_grid(&filled_grid(*cols, *rows, block_type, behaviour), *gap)
            }

            Custom(_) => None,
        }
            .unwrap_or_default()
    }

    /// Writes `token` into cell (`col`, `row`), and says whether that changed
    /// the level at all.
    ///
    /// The answer is what keeps change detection honest: erasing an empty cell
    /// is a click that edits nothing, and a level that did not change must not
    /// look to the rest of the editor as though it did.
    fn paint_cell(&mut self, col: usize, row: usize, token: &str) -> bool {
        let spread = self.spread_filled_grid();

        let Some(layout) = self.layout_mut() else { return spread; };

        let painted = set_cell(layout, col, row, token);
        if painted == *layout {
            return spread;
        }

        *layout = painted;
        true
    }

    /// Writes a `FilledGrid` out as the token grid that says the same thing, so
    /// a single cell of it can be painted - a grid with one cell changed is no
    /// longer "the same block everywhere". Says whether it had to.
    fn spread_filled_grid(&mut self) -> bool {
        let spread = match &self.level.targets {
            FilledGrid(cols, rows, block_type, behaviour, gap) => {
                SparseGrid(filled_grid(*cols, *rows, block_type, behaviour), *gap)
            }

            _ => return false,
        };

        self.level.targets = spread;
        true
    }

    /// Adds a row or a column at `edge`, and says whether it did.
    ///
    /// Refused when the grid it would make is one the editor could no longer
    /// show whole - see [`grid_fits_the_view`].
    fn grow_grid(&mut self, edge: Edge) -> bool {
        let Some((cols, rows, gap)) = self.grid() else { return false; };

        let grown = match edge {
            Edge::Top | Edge::Bottom => (cols.max(1), rows + 1),
            Edge::Left | Edge::Right => (cols + 1, rows.max(1)),
        };

        if !grid_fits_the_view(grown.0, grown.1, gap) {
            return false;
        }

        self.spread_filled_grid();

        let Some(layout) = self.layout_mut() else { return false; };
        *layout = grow(layout, edge);

        true
    }

    /// Takes the row or column at `edge` away, and whatever was standing on it
    /// with it. Says whether it did.
    ///
    /// Nothing here asks whether the author meant it - that is
    /// [`take_edge_away`]'s, so that the warning is between the press and this
    /// rather than inside it.
    fn shrink_grid(&mut self, edge: Edge) -> bool {
        if !self.can_shrink(edge) {
            return false;
        }

        self.spread_filled_grid();

        let Some(layout) = self.layout_mut() else { return false; };
        *layout = shrink(layout, edge);

        true
    }

    /// Whether the grid has an `edge` to spare. A level that is not a grid at
    /// all has not.
    fn can_shrink(&self, edge: Edge) -> bool {
        self.grid()
            .is_some_and(|(cols, rows, _)| can_shrink(cols, rows, edge))
    }

    /// How many blocks taking `edge` away would take with it.
    ///
    /// A `FilledGrid` is counted through the token grid that says the same
    /// thing, as [`EditorLevel::blocks`] reads it - one path from a level to
    /// what is standing on it, rather than one per layout kind.
    fn blocks_on_edge(&self, edge: Edge) -> usize {
        match &self.level.targets {
            SparseGrid(layout, _) => blocks_on_edge(layout, edge),

            FilledGrid(cols, rows, block_type, behaviour, _) => {
                blocks_on_edge(&filled_grid(*cols, *rows, block_type, behaviour), edge)
            }

            Custom(_) => 0,
        }
    }

    /// The token grid being edited, if the level is one.
    fn layout_mut(&mut self) -> Option<&mut String> {
        match &mut self.level.targets {
            SparseGrid(layout, _) => Some(layout),
            _ => None,
        }
    }
}

/// Marks everything the editor spawns, so leaving can take all of it with it.
#[derive(Component)]
pub struct EditorEntity;

/// Marks the camera cells are picked through.
///
/// Picking asks a camera, not *the* camera: the game's own camera is tilted
/// ([`TILTED_CAMERA`](crate::config::TILTED_CAMERA)) where the editor's looks
/// straight down, and `c0013`'s playtest round trip will have both alive in the
/// same run. Which one the pointer is read through has to be said out loud.
#[derive(Component)]
pub struct EditorCamera;

/// The grid cell the mouse is over, or `None` when the pointer is not over one.
///
/// Recomputed every frame the editor is up and cleared on the way out. `c0007`
/// paints whatever ends up in here; until then it is only drawn.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct HoveredCell(pub Option<(usize, usize)>);

/// What painting a cell writes into it.
///
/// One brush is a whole token: the block, how it behaves and the trigger it
/// takes part in - plus the mode that clears a cell instead of filling one.
/// `c0009` gives it a palette to be set from; until then it is set in code and
/// driven by the mouse.
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct Brush {
    pub block_type: BlockType,
    pub behaviour: BlockBehaviour,

    /// The trigger the painted block takes part in: a type *and* the group it
    /// belongs to. One field rather than two, because half a trigger is not a
    /// trigger - a type with no group is a token `make_block` cannot read - so
    /// the invalid brush cannot be built in the first place.
    pub trigger: Option<(TriggerType, TriggerGroup)>,

    /// Clear the cell rather than fill it.
    pub erase: bool,
}

impl Default for Brush {
    fn default() -> Self {
        Brush {
            block_type: BlockType::Simple,
            behaviour: BlockBehaviour::SittingDuck,
            trigger: None,
            erase: false,
        }
    }
}

impl Brush {
    /// The brush that empties cells.
    pub fn erasing() -> Self {
        Brush { erase: true, ..default() }
    }

    /// The token this brush writes into a cell.
    pub fn token(&self) -> String {
        if self.erase {
            return EMPTY_SLOT.to_string();
        }

        block_token(
            &self.block_type,
            &self.behaviour,
            self.trigger.as_ref().map(|(trigger, group)| (trigger, *group)),
        )
    }
}

/// The row or column an author has asked to remove and been warned about,
/// waiting for the press that confirms it.
///
/// Cleared by anything else the author does, because the warning is about the
/// level as it stood when it was given: a press has to confirm the warning on
/// screen, not one from before an edit.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PendingRemoval(pub Option<DoomedEdge>);

/// An edge that is one press away from being taken away, and what it holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DoomedEdge {
    pub edge: Edge,

    /// How many blocks go with it. Never 0 - an edge with nothing standing on it
    /// costs nothing and goes on the first press.
    pub blocks: usize,
}

impl DoomedEdge {
    /// What the author is told, in the one place it is worded.
    fn warning(&self) -> String {
        format!(
            "The {} holds {} block{} - press {} again to remove it",
            edge_name(self.edge),
            self.blocks,
            if self.blocks == 1 { "" } else { "s" },
            edge_shortcut(self.edge),
        )
    }
}

/// The row or column an author names with an arrow key.
fn edge_name(edge: Edge) -> &'static str {
    match edge {
        Edge::Top => "top row",
        Edge::Bottom => "bottom row",
        Edge::Left => "left column",
        Edge::Right => "right column",
    }
}

/// The press that would take it away, as the author has to type it.
fn edge_shortcut(edge: Edge) -> &'static str {
    match edge {
        Edge::Top => "Shift+Up",
        Edge::Bottom => "Shift+Down",
        Edge::Left => "Shift+Left",
        Edge::Right => "Shift+Right",
    }
}

/// The warning on screen, so it can be taken off again.
#[derive(Component)]
pub struct EditorWarning;

/// The paint in progress: from a mouse button going down to it coming up again.
///
/// A drag is *one* edit and not one per cell, so the stroke is recorded as a
/// whole from the start - `c0011` has a single undo entry to hang off a
/// finished one rather than a hundred identical ones to collapse afterwards.
#[derive(Resource, Debug, Default)]
pub struct PaintStroke(pub Option<Stroke>);

#[derive(Debug)]
pub struct Stroke {
    /// The layout as it stood when the button went down.
    pub before: TargetLayout,

    /// The cells this stroke has painted, in the order the pointer reached
    /// them. Also what keeps it from writing the same cell over and over while
    /// the pointer rests on it.
    pub cells: Vec<(usize, usize)>,
}

/// A block of the level under edit, on screen.
///
/// Not a game block: it carries what the token says and how it looks, and none
/// of the collider, hit points or behaviour that make a block something a ball
/// can play against.
#[derive(Component, Debug)]
pub struct EditorBlock(pub Block);

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
            .init_resource::<HoveredCell>()
            .init_resource::<Brush>()
            .init_resource::<PaintStroke>()
            .init_resource::<PendingRemoval>()

            .add_systems(
                OnEnter(GameState::Editor),
                (editor_open, editor_setup, editor_show_cursor),
            )

            .add_systems(
                Update,
                (
                    editor_leave,
                    editor_draw_grid,
                    // One chain, because each link needs the one before it to
                    // have run this frame: the highlight would trail the pointer
                    // by a frame, a painted cell would take a frame to show up as
                    // a block, and a block would spend that frame with no mesh
                    // on it - which is the whole grid blinking once per cell
                    // painted.
                    (
                        editor_pick_cell,
                        // Before painting, so that the click that stepped a
                        // setting is not also a click on whatever is behind the
                        // panel - and before the two systems that put the level
                        // back on screen, so a setting changed this frame shows
                        // this frame.
                        editor_settings_click,
                        editor_paint,
                        editor_resize,
                        // `resource_exists_and_changed` rather than
                        // `resource_changed`: a run condition is evaluated every
                        // frame whether or not the `in_state` beside it holds,
                        // and outside the editor there is no level for it to
                        // ask about. `PendingRemoval` below is a
                        // `init_resource`, so it is always there to ask.
                        editor_show_blocks.run_if(resource_exists_and_changed::<EditorLevel>),
                        editor_show_settings.run_if(resource_exists_and_changed::<EditorLevel>),
                        editor_dress_blocks,
                        editor_show_warning.run_if(resource_changed::<PendingRemoval>),
                        editor_draw_hover,
                        editor_draw_doomed_edge,
                    )
                        .chain(),
                )
                    .run_if(in_state(GameState::Editor)),
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
    editor_level: Option<ResMut<EditorLevel>>,
    levels: Res<Levels>,
    level_assets: Res<Assets<LevelAsset>>,
    mut commands: Commands,
) {
    // Already open. The level stays exactly as it was left - but the editor it
    // is being drawn into is brand new, and the blocks on screen hang off a
    // change to this resource, so it has to be announced again.
    if let Some(mut editor_level) = editor_level {
        editor_level.set_changed();
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
    info!(
        "Editor: left button paints, right button erases; an arrow key adds a \
         row or column at that edge and Shift+arrow takes it away; the panel \
         top left holds everything about the level that is not its grid; \
         Escape leaves"
    );

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
        EditorCamera,
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

/// The cell a ray points at, or `None` if it misses the grid.
///
/// The ray meets the ground plane the blocks sit on - y = 0 - and the hit point
/// is quantised by [`world_to_cell`], so this is deliberately *not* mesh
/// picking: an empty cell has no geometry to hit, and an editor that could only
/// hover the cells that already hold a block could never fill one in.
///
/// Nothing here knows where the ray came from, which is what keeps picking
/// honest under a camera looking straight down and under the game's tilted one
/// alike.
///
/// [`world_to_cell`] is open upwards - a level can grow rows, so the row count
/// is the caller's to know - and the caller is here: a hit past the last row is
/// off the grid, not on its top row.
pub fn cell_under_ray(ray: Ray3d, cols: usize, rows: usize, gap: f32) -> Option<(usize, usize)> {
    let distance = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))?;
    let hit = ray.get_point(distance);

    // The grid's second axis is world z: `cell_to_world` returns a `Vec2` on the
    // ground plane, not an x/y pair in the air.
    let (col, row) = world_to_cell(Vec2::new(hit.x, hit.z), cols, gap)?;

    (row < rows).then_some((col, row))
}


/// Where a highlight for a cell sits - the cell's own footprint on the ground
/// plane, lifted clear of whatever is drawn underneath it.
fn cell_highlight(col: usize, row: usize, cols: usize, gap: f32, lift: f32) -> Isometry3d {
    let centre = cell_to_world(col, row, cols, gap);

    Isometry3d::new(
        Vec3::new(centre.x, lift, centre.y),
        Quat::from_rotation_x(FRAC_PI_2),
    )
}

/// Where the hover highlight sits: clear of the grid gizmo.
fn hover_highlight(col: usize, row: usize, cols: usize, gap: f32) -> Isometry3d {
    cell_highlight(col, row, cols, gap, HOVER_LIFT)
}


/// Reads the pointer as a grid cell.
fn editor_pick_cell(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    editor_level: Res<EditorLevel>,
    mut hovered: ResMut<HoveredCell>,
) {
    let cell = cell_under_cursor(&windows, &cameras, &editor_level);

    // Only written when it actually moved, so change detection means "the
    // pointer entered a different cell" - which is the signal `c0007` wants.
    if hovered.0 != cell {
        hovered.0 = cell;
    }
}

/// The cell the pointer is over: a cursor position, through the camera, onto the
/// ground plane, onto the grid. Any of those steps can come up empty - the
/// pointer can be outside the window, the camera can be a frame away from having
/// a viewport to project through, and the level being edited need not be a grid
/// at all.
fn cell_under_cursor(
    windows: &Query<&Window, With<PrimaryWindow>>,
    cameras: &Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    editor_level: &EditorLevel,
) -> Option<(usize, usize)> {
    let (cols, rows, gap) = editor_level.grid()?;
    let cursor = windows.iter().next()?.cursor_position()?;

    // The settings panel is in front of the play field, and a click on it is
    // aimed at a setting rather than at the cell it happens to cover. Saying so
    // here rather than in `editor_paint` means the highlight goes too, so the
    // panel does not sit on top of a cell that still looks armed.
    if panel_rect().contains(cursor) {
        return None;
    }

    let (camera, camera_transform) = cameras.iter().next()?;
    let ray = camera.viewport_to_world(camera_transform, cursor).ok()?;

    cell_under_ray(ray, cols, rows, gap)
}

/// Outlines the hovered cell, so the pointer has something to aim with.
fn editor_draw_hover(
    hovered: Res<HoveredCell>,
    editor_level: Res<EditorLevel>,
    mut gizmos: Gizmos,
) {
    let Some((col, row)) = hovered.0 else { return; };
    let Some((cols, _, gap)) = editor_level.grid() else { return; };

    gizmos.rect(
        hover_highlight(col, row, cols, gap),
        Vec2::new(BLOCK_WIDTH, BLOCK_DEPTH),
        YELLOW,
    );
}


/// Paints the hovered cell, for as long as a mouse button is held down.
fn editor_paint(
    buttons: Res<ButtonInput<MouseButton>>,
    hovered: Res<HoveredCell>,
    brush: Res<Brush>,
    mut stroke: ResMut<PaintStroke>,
    mut pending: ResMut<PendingRemoval>,
    mut editor_level: ResMut<EditorLevel>,
) {
    let Some(painting) = brush_in_hand(&buttons, &brush) else {
        // Nothing held any more: whatever was being painted is finished.
        if let Some(done) = stroke.0.take() {
            if !done.cells.is_empty() {
                info!(
                    "painted {} cell(s), level changed: {}",
                    done.cells.len(),
                    done.before != editor_level.level.targets
                );
            }
        }

        return;
    };

    // The stroke starts when the button goes down, not when it first crosses a
    // cell: what it is an edit *from* is the level as it stood at that moment.
    if stroke.0.is_none() {
        stroke.0 = Some(Stroke {
            before: editor_level.level.targets.clone(),
            cells: vec![],
        });

        // Painting is not an answer to "remove this row?" - and it is an edit,
        // so what the warning counted is out of date anyway.
        pending.set_if_neq(PendingRemoval(None));
    }

    let Some((col, row)) = hovered.0 else { return; };

    let stroke = stroke.0.as_mut().expect("the stroke was just started");

    // A drag paints each cell it crosses once. Without this the cell under a
    // resting pointer is rewritten every frame.
    if stroke.cells.contains(&(col, row)) {
        return;
    }

    stroke.cells.push((col, row));

    // Written around change detection, so that a stroke that changes nothing -
    // the erase brush dragged over empty cells - does not respawn every block on
    // screen once a frame.
    let level = editor_level.bypass_change_detection();

    if level.paint_cell(col, row, &painting.token()) {
        editor_level.set_changed();
    }
}

/// The brush the mouse is painting with, or `None` when no button is down.
///
/// The left button paints what the brush says. The right one erases whatever
/// the brush is set to, which is how a cell gets cleared before `c0009`'s
/// palette exists to switch the brush's own erase mode on - and is where a level
/// author's hand goes for it anyway.
fn brush_in_hand(buttons: &ButtonInput<MouseButton>, brush: &Brush) -> Option<Brush> {
    if buttons.pressed(MouseButton::Left) {
        Some(brush.clone())
    } else if buttons.pressed(MouseButton::Right) {
        Some(Brush::erasing())
    } else {
        None
    }
}

/// Puts the level's blocks on screen, and puts them there again every time the
/// level changes - which, while painting, is the frame the cell was painted in.
///
/// The whole grid is rebuilt rather than the one cell that moved: a level is a
/// hundred blocks at the outside, and a paint that changes nothing never gets
/// this far.
fn editor_show_blocks(
    editor_level: Res<EditorLevel>,
    shown: Query<Entity, With<EditorBlock>>,
    mut commands: Commands,
) {
    for entity in &shown {
        commands.entity(entity).despawn();
    }

    for block in editor_level.blocks() {
        let position = block.position;

        commands.spawn((
            EditorBlock(block),
            EditorEntity,
            Transform::from_xyz(position.x, 0.0, position.y),
            Visibility::default(),
        ));
    }
}

/// Dresses the editor's blocks in the game's own mesh and material, as soon as
/// the glTF they come from has loaded.
///
/// Separate from spawning them because the level is on screen the instant it is
/// edited, where the asset it is drawn with arrives whenever it arrives - and,
/// in a test, never.
fn editor_dress_blocks(
    asset_pack: Option<Res<MyAssetPack>>,
    gltfs: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<BlockMaterial>>,
    undressed: Query<(Entity, &EditorBlock), Without<Mesh3d>>,
    mut commands: Commands,
) {
    if undressed.is_empty() {
        return;
    }

    let Some(asset_pack) = asset_pack else { return; };
    let Some(gltf) = gltfs.get(&asset_pack.0) else { return; };
    let Some(gltf_mesh) = gltf.named_meshes.get(BLOCK_MESH) else { return; };
    let Some(gltf_mesh) = gltf_meshes.get(gltf_mesh) else { return; };
    let Some(primitive) = gltf_mesh.primitives.first() else { return; };

    for (entity, block) in &undressed {
        commands.entity(entity).insert((
            Mesh3d(primitive.mesh.clone()),
            MeshMaterial3d(materials.add(block_material(&block.0.block_type, &asset_server))),
        ));
    }
}


/// Adds and removes rows and columns at the edges of the grid.
///
/// An arrow key grows the grid at the edge it points at; `Shift` and an arrow
/// takes that edge away again. Up and down are the grid's own top and bottom -
/// the first and last line of the layout - which is what the author sees at the
/// top and the bottom of the screen.
///
/// An edge with blocks standing on it is not taken away by one press: it is
/// called out first, and the same press again means it. That is this card's half
/// of "warns first or is undoable"; `c0011` brings the undo.
fn editor_resize(
    keys: Res<ButtonInput<KeyCode>>,
    mut pending: ResMut<PendingRemoval>,
    mut editor_level: ResMut<EditorLevel>,
) {
    let Some((edge, taking_away)) = resize_asked_for(&keys) else { return; };

    // Written around change detection, as painting is: a resize that is refused
    // - or one press short of happening - must not look to the rest of the
    // editor like a level that changed.
    let level = editor_level.bypass_change_detection();

    let changed = if taking_away {
        take_edge_away(level, edge, &mut pending)
    } else {
        pending.set_if_neq(PendingRemoval(None));
        level.grow_grid(edge)
    };

    if changed {
        editor_level.set_changed();
    }
}

/// The second half of [`editor_resize`]: the edge goes, or the author is told
/// what it would cost and the next press does it.
fn take_edge_away(
    level: &mut EditorLevel,
    edge: Edge,
    pending: &mut ResMut<PendingRemoval>,
) -> bool {
    if !level.can_shrink(edge) {
        return false;
    }

    let blocks = level.blocks_on_edge(edge);
    let already_warned = pending.0.map(|doomed| doomed.edge) == Some(edge);

    if blocks > 0 && !already_warned {
        let doomed = DoomedEdge { edge, blocks };
        warn!("{}", doomed.warning());
        pending.set_if_neq(PendingRemoval(Some(doomed)));

        return false;
    }

    pending.set_if_neq(PendingRemoval(None));
    level.shrink_grid(edge)
}

/// The resize an author is asking for: an arrow key names an edge, and `Shift`
/// turns adding one into taking it away.
fn resize_asked_for(keys: &ButtonInput<KeyCode>) -> Option<(Edge, bool)> {
    let edge = if keys.just_pressed(KeyCode::ArrowUp) {
        Edge::Top
    } else if keys.just_pressed(KeyCode::ArrowDown) {
        Edge::Bottom
    } else if keys.just_pressed(KeyCode::ArrowLeft) {
        Edge::Left
    } else if keys.just_pressed(KeyCode::ArrowRight) {
        Edge::Right
    } else {
        return None;
    };

    Some((edge, keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])))
}

/// Whether a `cols` x `rows` grid still fits the frame the editor keeps on
/// screen - the one promise the editor makes about the level it is showing, and
/// so the one thing that stops a grid growing.
///
/// Whether every cell of it is somewhere the ball can reach is a different
/// question, and `c0012`'s to warn about.
fn grid_fits_the_view(cols: usize, rows: usize, gap: f32) -> bool {
    let view = editor_view();
    let bounds = grid_bounds(cols, rows, gap);

    view.contains(bounds.min) && view.contains(bounds.max)
}

/// The cells that make up one edge of a `cols` x `rows` grid.
fn edge_cells(edge: Edge, cols: usize, rows: usize) -> Vec<(usize, usize)> {
    match edge {
        Edge::Top => (0..cols).map(|col| (col, 0)).collect(),
        Edge::Bottom => (0..cols).map(|col| (col, rows.saturating_sub(1))).collect(),
        Edge::Left => (0..rows).map(|row| (0, row)).collect(),
        Edge::Right => (0..rows).map(|row| (cols.saturating_sub(1), row)).collect(),
    }
}

/// Draws the row or column an author has been warned about, so the warning
/// points at something rather than describing it.
fn editor_draw_doomed_edge(
    pending: Res<PendingRemoval>,
    editor_level: Res<EditorLevel>,
    mut gizmos: Gizmos,
) {
    let Some(doomed) = pending.0 else { return; };
    let Some((cols, rows, gap)) = editor_level.grid() else { return; };

    for (col, row) in edge_cells(doomed.edge, cols, rows) {
        gizmos.rect(
            cell_highlight(col, row, cols, gap, WARNING_LIFT),
            Vec2::new(BLOCK_WIDTH, BLOCK_DEPTH),
            ORANGE_RED,
        );
    }
}

/// Puts the warning on screen, and takes it away again when there is nothing
/// left to warn about.
fn editor_show_warning(
    pending: Res<PendingRemoval>,
    shown: Query<Entity, With<EditorWarning>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for entity in &shown {
        commands.entity(entity).despawn();
    }

    let Some(doomed) = pending.0 else { return; };

    commands.spawn((
        Text::new(doomed.warning()),
        TextFont {
            font: asset_server.load(EDITOR_FONT).into(),
            font_size: FontSize::Px(24.0),
            ..default()
        },
        TextColor(ORANGE_RED.into()),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(24.0),
            left: Val::Px(24.0),
            ..default()
        },
        EditorWarning,
        EditorEntity,
    ));
}


/// Steps a setting the author clicked a button of.
///
/// The level is written around change detection, as painting is: a click at the
/// end of a setting's range - or anywhere on the panel that is not a button -
/// must not look to the rest of the editor like a level that changed.
fn editor_settings_click(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut editor_level: ResMut<EditorLevel>,
) {
    // The press, not the hold: a stepper walked once per frame the button is
    // down would run the whole range in a fifth of a second.
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(cursor) = windows.iter().next().and_then(|window| window.cursor_position()) else { return; };
    let Some((setting, by)) = setting_at(cursor) else { return; };

    let level = editor_level.bypass_change_detection();

    if setting.step(&mut level.level, by) {
        editor_level.set_changed();
    }
}

/// Puts the settings panel on screen, and puts it there again every time the
/// level changes - which, for a setting stepped by a click, is the frame it was
/// clicked in.
fn editor_show_settings(
    editor_level: Res<EditorLevel>,
    shown: Query<Entity, With<SettingsPanel>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for entity in &shown {
        commands.entity(entity).despawn();
    }

    spawn_settings_panel(&editor_level.level, &asset_server, &mut commands);
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

fn editor_teardown(
    mut commands: Commands,
    editor_entities: Query<Entity, With<EditorEntity>>,
    mut hovered: ResMut<HoveredCell>,
    mut stroke: ResMut<PaintStroke>,
    mut pending: ResMut<PendingRemoval>,
) {
    for entity in &editor_entities {
        commands.entity(entity).despawn();
    }

    // The camera the cell was picked through is going with them.
    hovered.0 = None;

    // A stroke left half-painted is over: the button will have come up
    // somewhere else entirely by the time the editor is back.
    stroke.0 = None;

    // The warning went with the rest of the editor's entities, and a question
    // that is no longer on screen must not be answerable.
    pending.set_if_neq(PendingRemoval(None));
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
    use bevy::camera::{PerspectiveProjection, RenderTargetInfo};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::gizmos::config::{DefaultGizmoConfigGroup, GizmoConfig, GizmoConfigStore};
    use bevy::input::mouse::MouseButtonInput;
    use bevy::input::{ButtonState, InputPlugin};
    use bevy::math::Dir3;
    use bevy::prelude::State;
    use bevy::state::app::{AppExtStates, StatesPlugin};
    use bevy::text::Font;
    use bevy::transform::TransformPlugin;
    use bevy::window::WindowResolution;
    use bevy::MinimalPlugins;

    use crate::config::{BLOCK_GAP, BLOCK_WIDTH_H, CAMERA_TILT, TILTED_CAMERA};
    use crate::editor::settings::{settings_rows, Setting, SettingValue, SETTINGS};
    use crate::level::campaign;
    use crate::level::WinCriteria;
    use crate::pickups::PickupType;

    /// The window every test picks through. Wider than it is tall, as the game's
    /// own is, so a projection that quietly swapped the two axes would show.
    const VIEWPORT: UVec2 = UVec2::new(1600, 800);

    /// Just enough app to walk in and out of the editor: the states it hangs
    /// off, the asset collection it opens levels from, the keyboard it listens
    /// to, and a primary window with the pointer hidden - as `WindowPlugin`
    /// leaves it.
    fn editor_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), StatesPlugin, InputPlugin));
        // Picking reads the camera's `GlobalTransform`, which is nothing but the
        // identity until propagation has run.
        app.add_plugins(TransformPlugin);
        // `Gizmos` reads the config store, and the editor draws its grid with
        // gizmos every frame. The rest of `GizmoPlugin` wants a renderer, which
        // a headless app has no use for.
        app.init_resource::<GizmoConfigStore>();
        app.world_mut()
            .resource_mut::<GizmoConfigStore>()
            .insert(GizmoConfig::default(), DefaultGizmoConfigGroup);
        app.init_asset::<LevelAsset>();
        // The editor draws the level's blocks with the game's own glTF mesh and
        // material, so the collections those live in have to exist - even in a
        // headless test, which never has a glTF for them to hold.
        app.init_asset::<Gltf>();
        app.init_asset::<GltfMesh>();
        app.init_asset::<BlockMaterial>();
        // Same for the font the warning is written in - `TextPlugin` is what
        // registers it in the game, and a headless app has no text to draw.
        app.init_asset::<Font>();
        app.insert_state(GameState::InGame);
        app.insert_resource(Levels { handles: vec![], current_level: 0 });
        app.add_plugins(EditorPlugin);

        // A real `Window`, not just the cursor options: picking starts at
        // `Window::cursor_position`, and that needs a resolution to be inside of.
        app.world_mut().spawn((
            Window {
                resolution: WindowResolution::new(VIEWPORT.x, VIEWPORT.y),
                ..default()
            },
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

    /// A run condition is evaluated every frame, whether or not the `in_state`
    /// next to it holds, so a condition asking after the level under edit has to
    /// survive there not being one - which is every frame of the game before the
    /// editor has ever been opened.
    #[test]
    fn the_editor_asks_nothing_of_a_game_that_has_not_opened_it() {
        let mut app = editor_app();

        for _ in 0..3 {
            app.update();
        }

        assert!(app.world().get_resource::<EditorLevel>().is_none());
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

            assert!(
                grid_fits_the_view(cols, rows, gap),
                "{}: a {cols}x{rows} grid covers {:?}, outside the editor's {view:?}",
                path.display(),
                grid_bounds(cols, rows, gap)
            );

            checked += 1;
        }

        assert!(checked >= 8, "only {checked} level files were checked - is the directory right?");
    }


    // --- picking ---------------------------------------------------------

    /// The ray a camera at `camera` hands back once the pointer is over `target`
    /// on the ground plane - `viewport_to_world`'s output, without the viewport.
    fn ray_at(camera: Vec3, target: Vec2) -> Ray3d {
        let target = Vec3::new(target.x, 0.0, target.y);

        Ray3d {
            origin: camera,
            direction: Dir3::new(target - camera).expect("the camera is not standing on the cell"),
        }
    }

    /// Where the game puts its camera, by `setup_3d_environment`'s own
    /// arithmetic - tilted by [`CAMERA_TILT`] when [`TILTED_CAMERA`] says so.
    fn game_camera_position() -> Vec3 {
        let mut position = Vec3::new(0.0, 200.0, 0.00001);

        if TILTED_CAMERA {
            position = Quat::from_rotation_x(CAMERA_TILT) * position;
        }

        position
    }

    /// Straight down from high above, which is where the editor's own camera is.
    fn from_above(target: Vec2) -> Ray3d {
        ray_at(Vec3::new(target.x, EDITOR_CAMERA_HEIGHT, target.y), target)
    }

    fn cells(cols: usize, rows: usize) -> impl Iterator<Item = (usize, usize)> {
        (0..rows).flat_map(move |row| (0..cols).map(move |col| (col, row)))
    }

    /// A ray aimed at the centre of a cell has to come back with that cell -
    /// every cell, on an odd-column grid and an even-column one, since the two
    /// are centred differently.
    #[test]
    fn a_ray_aimed_at_a_cell_finds_that_cell() {
        for (cols, rows) in [(9, 6), (10, 4), (1, 1), (11, 8)] {
            for (col, row) in cells(cols, rows) {
                let centre = cell_to_world(col, row, cols, BLOCK_GAP);

                assert_eq!(
                    cell_under_ray(from_above(centre), cols, rows, BLOCK_GAP),
                    Some((col, row)),
                    "{cols}x{rows} grid, cell ({col}, {row}) at {centre:?}"
                );
            }
        }
    }

    /// Anywhere inside a cell counts as that cell, not just the exact centre -
    /// an author aiming with a mouse never hits the middle.
    #[test]
    fn anywhere_inside_a_cell_is_that_cell() {
        let (cols, rows) = (9, 6);
        let corner = Vec2::new(BLOCK_WIDTH_H, BLOCK_DEPTH / 2.0) * 0.9;

        for (col, row) in cells(cols, rows) {
            let centre = cell_to_world(col, row, cols, BLOCK_GAP);

            for offset in [corner, -corner, corner * Vec2::new(1.0, -1.0)] {
                assert_eq!(
                    cell_under_ray(from_above(centre + offset), cols, rows, BLOCK_GAP),
                    Some((col, row)),
                    "cell ({col}, {row}) at {centre:?}, offset by {offset:?}"
                );
            }
        }
    }

    /// Off the grid on any side is no cell at all. The row above the last one is
    /// the interesting side: `world_to_cell` is open upwards, so rejecting it is
    /// the editor's own job.
    #[test]
    fn a_ray_that_lands_off_the_grid_finds_no_cell() {
        let (cols, rows) = (9, 6);
        let x_step = BLOCK_WIDTH + BLOCK_GAP;
        let y_step = BLOCK_DEPTH + BLOCK_GAP;

        let first = cell_to_world(0, 0, cols, BLOCK_GAP);
        let last = cell_to_world(cols - 1, rows - 1, cols, BLOCK_GAP);

        for outside in [
            Vec2::new(first.x - x_step, first.y),
            Vec2::new(last.x + x_step, last.y),
            Vec2::new(first.x, first.y - y_step),
            Vec2::new(last.x, last.y + y_step),
        ] {
            assert_eq!(
                cell_under_ray(from_above(outside), cols, rows, BLOCK_GAP),
                None,
                "{outside:?} is outside a {cols}x{rows} grid"
            );
        }
    }

    /// A ray that never reaches the ground plane has no hit point to quantise.
    #[test]
    fn a_ray_that_misses_the_ground_finds_no_cell() {
        let centre = cell_to_world(4, 3, 9, BLOCK_GAP);
        let origin = Vec3::new(centre.x, EDITOR_CAMERA_HEIGHT, centre.y);

        for direction in [Dir3::Y, Dir3::X, Dir3::Z] {
            assert_eq!(
                cell_under_ray(Ray3d { origin, direction }, 9, 6, BLOCK_GAP),
                None,
                "a ray pointing {direction:?} from above the field never lands on it"
            );
        }
    }

    /// The card's tilted-camera criterion at the level of the maths: nothing in
    /// [`cell_under_ray`] assumes the ray came from straight overhead, so the
    /// game's own tilted viewpoint picks the same cells.
    #[test]
    fn picking_is_correct_from_the_tilted_camera() {
        let camera = game_camera_position();
        assert!(camera.z > 1.0, "TILTED_CAMERA is off - this test would prove nothing");

        for (cols, rows) in [(9, 6), (10, 4)] {
            for (col, row) in cells(cols, rows) {
                let centre = cell_to_world(col, row, cols, BLOCK_GAP);

                assert_eq!(
                    cell_under_ray(ray_at(camera, centre), cols, rows, BLOCK_GAP),
                    Some((col, row)),
                    "{cols}x{rows} grid, cell ({col}, {row}) at {centre:?} from {camera:?}"
                );
            }
        }
    }

    /// The highlight is drawn on the cell that was picked, flat on the ground
    /// plane and clear of the grid gizmo underneath it.
    #[test]
    fn the_highlight_sits_on_the_hovered_cell() {
        let cols = 9;

        for (col, row) in cells(cols, 6) {
            let centre = cell_to_world(col, row, cols, BLOCK_GAP);
            let highlight = hover_highlight(col, row, cols, BLOCK_GAP);

            assert_eq!(
                Vec3::from(highlight.translation),
                Vec3::new(centre.x, HOVER_LIFT, centre.y),
                "cell ({col}, {row})"
            );
        }
    }


    // --- picking, end to end through a camera and a window ----------------

    /// What `camera_system` does the frame a camera meets a window, minus the
    /// renderer a headless test has none of: hand the camera its viewport and
    /// the projection matrix that goes with it. Until it has run,
    /// `viewport_to_world` has no viewport to read and nothing to project
    /// through.
    fn give_the_camera_its_viewport(app: &mut App) {
        let viewport = window_size(app);
        let world = app.world_mut();
        let mut cameras =
            world.query_filtered::<(&mut Camera, &mut Projection), With<EditorCamera>>();

        for (mut camera, mut projection) in cameras.iter_mut(world) {
            camera.computed.target_info = Some(RenderTargetInfo {
                physical_size: viewport,
                scale_factor: 1.0,
            });

            let size = viewport.as_vec2();
            projection.update(size.x, size.y);
            camera.computed.clip_from_view = projection.get_clip_from_view();
        }
    }

    /// The window the test is pointing at, in physical pixels - which, at a
    /// scale factor of 1, is also the pixel space `Window::cursor_position` and
    /// the settings panel both work in.
    fn window_size(app: &mut App) -> UVec2 {
        let world = app.world_mut();
        let mut windows = world.query_filtered::<&Window, With<PrimaryWindow>>();
        let window = windows.iter(world).next().expect("the test app has a window");

        window.physical_size()
    }

    /// Squeezes the window into `size` and hands the camera the viewport that
    /// goes with it, as `camera_system` would the frame a window was resized.
    fn resize_the_window(app: &mut App, size: UVec2) {
        {
            let world = app.world_mut();
            let mut windows = world.query_filtered::<&mut Window, With<PrimaryWindow>>();

            for mut window in windows.iter_mut(world) {
                window.resolution = WindowResolution::new(size.x, size.y);
            }
        }

        app.update();
        give_the_camera_its_viewport(app);
    }

    /// An app sitting in the editor with a camera that can be projected through.
    fn app_in_the_editor(level: LevelDefinition) -> App {
        let mut app = app_playing(level);

        go_to(&mut app, GameState::Editor);
        give_the_camera_its_viewport(&mut app);

        app
    }

    /// Swaps in the game's own camera - perspective, and tilted by
    /// [`CAMERA_TILT`] - where the editor's straight-down one was.
    fn use_the_game_camera(app: &mut App) {
        let camera = {
            let world = app.world_mut();
            let mut cameras = world.query_filtered::<Entity, With<EditorCamera>>();
            cameras.iter(world).next().expect("the editor spawns a camera")
        };

        app.world_mut().entity_mut(camera).insert((
            Projection::Perspective(PerspectiveProjection::default()),
            Transform::from_translation(game_camera_position()).looking_at(Vec3::ZERO, Vec3::Y),
        ));

        // Propagate the new transform before anything projects through it.
        app.update();
        give_the_camera_its_viewport(app);
    }

    fn put_the_pointer_at(app: &mut App, pixel: Option<Vec2>) {
        let world = app.world_mut();
        let mut windows = world.query_filtered::<&mut Window, With<PrimaryWindow>>();

        for mut window in windows.iter_mut(world) {
            window.set_physical_cursor_position(pixel.map(|pixel| pixel.as_dvec2()));
        }
    }

    /// Points at a spot on the ground plane the way an author would: works out
    /// which pixel it shows up at and puts the pointer there. `false` if that
    /// spot is not on screen at all.
    fn point_at(app: &mut App, target: Vec2) -> bool {
        let pixel = {
            let world = app.world_mut();
            let mut cameras =
                world.query_filtered::<(&Camera, &GlobalTransform), With<EditorCamera>>();
            let (camera, transform) =
                cameras.iter(world).next().expect("the editor spawns a camera");

            camera
                .world_to_viewport(transform, Vec3::new(target.x, 0.0, target.y))
                .ok()
        };

        let Some(pixel) = pixel else { return false; };

        let viewport = window_size(app).as_vec2();

        if pixel.x < 0.0 || pixel.y < 0.0 || pixel.x >= viewport.x || pixel.y >= viewport.y {
            return false;
        }

        put_the_pointer_at(app, Some(pixel));
        app.update();

        true
    }

    fn hovered(app: &App) -> Option<(usize, usize)> {
        app.world().resource::<HoveredCell>().0
    }

    /// The whole chain, through a real camera and a real window: the pixel a
    /// cell shows up at has to be the pixel that picks it, for every cell.
    #[test]
    fn the_pointer_finds_the_cell_it_is_over() {
        let mut app = app_in_the_editor(sparse(&empty_grid(9, 6)));

        for (col, row) in cells(9, 6) {
            let centre = cell_to_world(col, row, 9, BLOCK_GAP);

            assert!(point_at(&mut app, centre), "cell ({col}, {row}) is off screen");
            assert_eq!(hovered(&app), Some((col, row)), "cell ({col}, {row}) at {centre:?}");
        }
    }

    /// The reason this is not mesh picking: there is nothing to hit over an
    /// empty cell, and an editor that could only hover the cells that already
    /// hold a block could never fill one in.
    #[test]
    fn an_empty_cell_hovers_just_like_a_full_one() {
        let mut app = app_in_the_editor(sparse("AA .. AA\n.. AA ..\n.. .. .."));

        for (col, row) in [(1, 0), (0, 1), (2, 1), (0, 2), (1, 2), (2, 2)] {
            let centre = cell_to_world(col, row, 3, BLOCK_GAP);

            assert!(point_at(&mut app, centre), "cell ({col}, {row}) is off screen");
            assert_eq!(
                hovered(&app),
                Some((col, row)),
                "({col}, {row}) holds no block, and still has to be hoverable"
            );
        }
    }

    /// Off the grid, and off the window, are both "no cell" - and the highlight
    /// that was showing goes with it.
    #[test]
    fn the_pointer_hovers_nothing_when_it_is_not_over_a_cell() {
        let mut app = app_in_the_editor(sparse(&empty_grid(9, 6)));

        let centre = cell_to_world(4, 3, 9, BLOCK_GAP);
        assert!(point_at(&mut app, centre));
        assert_eq!(hovered(&app), Some((4, 3)), "something has to be hovered first");

        let x_step = BLOCK_WIDTH + BLOCK_GAP;
        let beyond_the_last_column = cell_to_world(8, 3, 9, BLOCK_GAP) + Vec2::new(x_step, 0.0);
        assert!(point_at(&mut app, beyond_the_last_column));
        assert_eq!(hovered(&app), None, "a spot on screen but off the grid is not a cell");

        assert!(point_at(&mut app, centre));
        assert_eq!(hovered(&app), Some((4, 3)));

        put_the_pointer_at(&mut app, None);
        app.update();
        assert_eq!(hovered(&app), None, "a pointer outside the window is over nothing");
    }

    /// The card's tilted-camera criterion end to end: the same round trip, asked
    /// through the game's own perspective camera instead of the editor's.
    #[test]
    fn picking_is_correct_through_the_tilted_camera() {
        let mut app = app_in_the_editor(sparse(&empty_grid(9, 6)));
        use_the_game_camera(&mut app);

        let mut checked = 0;

        for (col, row) in cells(9, 6) {
            let centre = cell_to_world(col, row, 9, BLOCK_GAP);

            if !point_at(&mut app, centre) {
                continue;
            }

            assert_eq!(hovered(&app), Some((col, row)), "cell ({col}, {row}) at {centre:?}");
            checked += 1;
        }

        assert_eq!(checked, 9 * 6, "every cell of the grid has to be on screen to have been tested");
    }

    // --- painting ---------------------------------------------------------

    fn hold(app: &mut App, button: MouseButton) {
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(button);
    }

    /// Lets the button go and gives the editor the frame it needs to notice.
    fn let_go(app: &mut App, button: MouseButton) {
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().release(button);
        app.update();
    }

    /// Drags the pointer over `cells` with `button` held, the way an author
    /// does: one press, a move per cell, one release.
    fn drag_over(app: &mut App, button: MouseButton, cells: &[(usize, usize)]) {
        let (cols, _, gap) = editor_level(app).grid().expect("a grid to paint on");

        hold(app, button);

        for &(col, row) in cells {
            assert!(
                point_at(app, cell_to_world(col, row, cols, gap)),
                "cell ({col}, {row}) is off screen"
            );
        }

        let_go(app, button);
    }

    /// A left click on each of `cells`, brush as it stands.
    fn paint(app: &mut App, cells: &[(usize, usize)]) {
        drag_over(app, MouseButton::Left, cells);
    }

    fn use_brush(app: &mut App, brush: Brush) {
        app.insert_resource(brush);
    }

    /// The token grid under edit, which is what every paint ends up in.
    fn layout(app: &App) -> String {
        match &editor_level(app).level.targets {
            SparseGrid(layout, _) => layout.clone(),
            other => panic!("the level under edit is not a token grid: {other:?}"),
        }
    }

    /// The blocks on screen, as (where they are, what they are).
    fn blocks_on_screen(app: &mut App) -> Vec<(Vec3, BlockType)> {
        let world = app.world_mut();
        let mut blocks = world.query::<(&EditorBlock, &Transform)>();

        let mut shown: Vec<(Vec3, BlockType)> = blocks
            .iter(world)
            .map(|(block, transform)| (transform.translation, block.0.block_type.clone()))
            .collect();

        shown.sort_by(|a, b| a.0.to_array().partial_cmp(&b.0.to_array()).unwrap());
        shown
    }

    fn at(col: usize, row: usize, cols: usize) -> Vec3 {
        let centre = cell_to_world(col, row, cols, BLOCK_GAP);

        Vec3::new(centre.x, 0.0, centre.y)
    }

    /// The brush an editor opens with paints the plainest thing the format has,
    /// and does not erase.
    #[test]
    fn the_brush_starts_on_a_plain_block() {
        let mut app = app_in_the_editor(sparse(&empty_grid(3, 2)));
        let brush = app.world().resource::<Brush>();

        assert_eq!(brush, &Brush::default());
        assert_eq!(brush.token(), "AA");
        assert!(!brush.erase);
        assert_eq!(Brush::erasing().token(), EMPTY_SLOT);
    }

    #[test]
    fn clicking_a_cell_writes_the_brush_into_the_level() {
        let mut app = app_in_the_editor(sparse(&empty_grid(3, 2)));

        use_brush(&mut app, Brush {
            block_type: BlockType::Concrete,
            behaviour: BlockBehaviour::Spinner,
            ..default()
        });

        paint(&mut app, &[(1, 0)]);

        assert_eq!(layout(&app), ".. CB ..\n.. .. ..");
    }

    /// The whole token, trigger and all - a trigger is what the brush is *for*
    /// as much as the block is.
    #[test]
    fn a_brush_with_a_trigger_writes_the_whole_token() {
        let mut app = app_in_the_editor(sparse(&empty_grid(2, 1)));

        use_brush(&mut app, Brush {
            block_type: BlockType::Simple,
            behaviour: BlockBehaviour::Portal,
            trigger: Some((TriggerType::ReceiverStartingInactive, 3)),
            ..default()
        });

        paint(&mut app, &[(0, 0)]);

        assert_eq!(layout(&app), "AIR3 ..");
    }

    /// Despite the name, `Z` is an ordinary cell of the grid that happens to be
    /// unbreakable - `LEVEL4` is built out of them - so it is an ordinary brush.
    #[test]
    fn an_obstacle_is_an_ordinary_brush() {
        let mut app = app_in_the_editor(sparse(&empty_grid(3, 1)));

        use_brush(&mut app, Brush { block_type: BlockType::Obstacle, ..default() });
        paint(&mut app, &[(0, 0), (2, 0)]);

        assert_eq!(layout(&app), "ZA .. ZA");
        assert_eq!(
            blocks_on_screen(&mut app),
            vec![(at(0, 0, 3), BlockType::Obstacle), (at(2, 0, 3), BlockType::Obstacle)]
        );
    }

    #[test]
    fn the_erase_brush_clears_a_cell_back_to_empty() {
        let mut app = app_in_the_editor(sparse("AA CB AA"));

        use_brush(&mut app, Brush::erasing());
        paint(&mut app, &[(1, 0)]);

        assert_eq!(layout(&app), "AA .. AA");
        assert_eq!(blocks_on_screen(&mut app).len(), 2, "the erased block is off screen too");
    }

    /// The right button erases whatever the brush is set to, which is how a cell
    /// is cleared before `c0009`'s palette can switch the brush's own mode.
    #[test]
    fn the_right_button_erases_whatever_the_brush_is_set_to() {
        let mut app = app_in_the_editor(sparse("AA CB AA"));

        use_brush(&mut app, Brush { block_type: BlockType::Hardling, ..default() });
        drag_over(&mut app, MouseButton::Right, &[(0, 0), (1, 0)]);

        assert_eq!(layout(&app), ".. .. AA");
    }

    /// "Immediately" means the frame it was painted in, not the one after: the
    /// block is on screen before the button has even come up again.
    #[test]
    fn a_painted_cell_shows_up_as_a_block_the_same_frame() {
        let mut app = app_in_the_editor(sparse(&empty_grid(3, 2)));
        assert_eq!(blocks_on_screen(&mut app), vec![], "an empty grid has nothing to show");

        hold(&mut app, MouseButton::Left);
        assert!(point_at(&mut app, cell_to_world(1, 1, 3, BLOCK_GAP)));

        assert_eq!(blocks_on_screen(&mut app), vec![(at(1, 1, 3), BlockType::Simple)]);

        let_go(&mut app, MouseButton::Left);
    }

    /// The level the editor opened is on screen from the start, and is back on
    /// screen after a trip out and in - which is the round trip `c0013` makes.
    #[test]
    fn the_blocks_of_the_level_under_edit_are_on_screen_every_time_it_is_opened() {
        let mut app = app_in_the_editor(sparse("AA .. AA"));

        assert_eq!(
            blocks_on_screen(&mut app),
            vec![(at(0, 0, 3), BlockType::Simple), (at(2, 0, 3), BlockType::Simple)]
        );

        go_to(&mut app, GameState::InGame);
        assert_eq!(blocks_on_screen(&mut app), vec![], "the editor takes its blocks with it");

        go_to(&mut app, GameState::Editor);
        assert_eq!(
            blocks_on_screen(&mut app),
            vec![(at(0, 0, 3), BlockType::Simple), (at(2, 0, 3), BlockType::Simple)],
            "and puts them back on the way in"
        );
    }

    /// One press, a run of cells, one release - no click per cell.
    #[test]
    fn dragging_paints_every_cell_it_crosses() {
        let mut app = app_in_the_editor(sparse(&empty_grid(4, 3)));

        paint(&mut app, &[(0, 1), (1, 1), (2, 1), (3, 1)]);

        assert_eq!(layout(&app), ".. .. .. ..\nAA AA AA AA\n.. .. .. ..");
        assert_eq!(blocks_on_screen(&mut app).len(), 4);
    }

    /// A drag is one edit rather than one per cell, which is what `c0011` hangs
    /// a single undo entry off.
    #[test]
    fn a_drag_is_one_edit() {
        let mut app = app_in_the_editor(sparse(&empty_grid(3, 2)));
        let before = editor_level(&app).level.targets.clone();

        hold(&mut app, MouseButton::Left);

        for (col, row) in [(0, 0), (1, 0), (2, 0)] {
            assert!(point_at(&mut app, cell_to_world(col, row, 3, BLOCK_GAP)));
        }

        let stroke = app.world().resource::<PaintStroke>();
        let stroke = stroke.0.as_ref().expect("a button held down is a stroke in progress");

        assert_eq!(stroke.cells, vec![(0, 0), (1, 0), (2, 0)], "one stroke, all three cells");
        assert_eq!(stroke.before, before, "the stroke knows what the level was before it");

        let_go(&mut app, MouseButton::Left);

        assert!(
            app.world().resource::<PaintStroke>().0.is_none(),
            "the button coming up ends the stroke"
        );
    }

    /// Crossing a cell again in the same drag does not paint it again - the
    /// pointer wanders back over cells it has already been on all the time.
    #[test]
    fn crossing_a_cell_twice_in_one_drag_writes_it_once() {
        let mut app = app_in_the_editor(sparse(&empty_grid(2, 1)));

        hold(&mut app, MouseButton::Left);
        assert!(point_at(&mut app, cell_to_world(0, 0, 2, BLOCK_GAP)));

        // Switching the brush mid-drag is the only way to tell a second write
        // from the first one.
        use_brush(&mut app, Brush { block_type: BlockType::Concrete, ..default() });

        assert!(point_at(&mut app, cell_to_world(1, 0, 2, BLOCK_GAP)));
        assert!(point_at(&mut app, cell_to_world(0, 0, 2, BLOCK_GAP)));

        let_go(&mut app, MouseButton::Left);

        assert_eq!(layout(&app), "AA CA");
        assert!(app.world().resource::<PaintStroke>().0.is_none(), "the stroke is over");
    }

    /// Held down over nothing paints nothing - the pointer leaves the grid on
    /// its way across it all the time.
    #[test]
    fn nothing_is_painted_where_there_is_no_cell() {
        let mut app = app_in_the_editor(sparse(&empty_grid(3, 2)));

        let x_step = BLOCK_WIDTH + BLOCK_GAP;
        let off_the_grid = cell_to_world(2, 1, 3, BLOCK_GAP) + Vec2::new(x_step, 0.0);

        hold(&mut app, MouseButton::Left);
        assert!(point_at(&mut app, off_the_grid));
        assert_eq!(hovered(&app), None, "the pointer has to be off the grid for this to mean anything");

        assert_eq!(layout(&app), empty_grid(3, 2));

        put_the_pointer_at(&mut app, None);
        app.update();
        let_go(&mut app, MouseButton::Left);

        assert_eq!(layout(&app), empty_grid(3, 2));
        assert_eq!(blocks_on_screen(&mut app), vec![]);
    }

    /// The card's round trip: whatever was painted has to come back out of the
    /// layout as the same blocks, in the same places.
    #[test]
    fn the_edited_layout_parses_back_to_the_blocks_that_were_painted() {
        let mut app = app_in_the_editor(sparse(&empty_grid(4, 3)));

        let painted = [
            ((0, 0), Brush {
                block_type: BlockType::Concrete,
                behaviour: BlockBehaviour::Spinner,
                ..default()
            }),
            ((3, 2), Brush {
                block_type: BlockType::Obstacle,
                behaviour: BlockBehaviour::Portal,
                trigger: Some((TriggerType::ReceiverStartingInactive, 4)),
                ..default()
            }),
            ((2, 1), Brush {
                block_type: BlockType::SimpleTop,
                behaviour: BlockBehaviour::EvaderL(50.0),
                trigger: Some((TriggerType::Start, 0)),
                ..default()
            }),
        ];

        for (cell, brush) in &painted {
            use_brush(&mut app, brush.clone());
            paint(&mut app, &[*cell]);
        }

        let blocks = interpret_grid(&layout(&app), BLOCK_GAP)
            .expect("the layout the editor writes has to parse");

        assert_eq!(blocks.len(), painted.len(), "every painted cell, and nothing else");

        for ((col, row), brush) in painted {
            let position = cell_to_world(col, row, 4, BLOCK_GAP);
            let block = blocks
                .iter()
                .find(|block| block.position == position)
                .unwrap_or_else(|| panic!("nothing came back at ({col}, {row})"));

            assert_eq!(block.block_type, brush.block_type);
            assert_eq!(block.behaviour, brush.behaviour);
            assert_eq!(block.trigger_type, brush.trigger.as_ref().map(|(t, _)| t.clone()));
            assert_eq!(block.trigger_group, brush.trigger.map(|(_, group)| group));
        }
    }

    /// Every level file that names no layout is a `FilledGrid` - the default -
    /// and one cell of it changed is no longer "the same block everywhere". It
    /// becomes the token grid that says the same thing, blocks where they were.
    #[test]
    fn painting_a_cell_of_a_filled_grid_turns_it_into_a_token_grid() {
        let filled = LevelDefinition {
            targets: FilledGrid(3, 2, BlockType::Hardling, BlockBehaviour::Vanisher, BLOCK_GAP),
            ..default()
        };

        let mut app = app_in_the_editor(filled);

        assert_eq!(blocks_on_screen(&mut app).len(), 6, "a filled grid is on screen as it is");

        use_brush(&mut app, Brush::erasing());
        paint(&mut app, &[(1, 0)]);

        assert_eq!(layout(&app), "BC .. BC\nBC BC BC");
        assert_eq!(editor_level(&app).grid(), Some((3, 2, BLOCK_GAP)), "the same grid it was");
        assert_eq!(blocks_on_screen(&mut app).len(), 5);
    }

    /// A `Custom` level is built in code: there is no grid to hover and no
    /// layout to write, and a click on it has to be a click on nothing rather
    /// than a panic.
    #[test]
    fn a_level_that_is_not_a_grid_cannot_be_painted() {
        let custom = LevelDefinition {
            targets: Custom("Conveyor".to_string()),
            ..default()
        };

        let mut app = app_in_the_editor(custom.clone());

        hold(&mut app, MouseButton::Left);
        assert!(point_at(&mut app, Vec2::ZERO));
        let_go(&mut app, MouseButton::Left);

        assert_eq!(editor_level(&app).level, custom);
        assert_eq!(blocks_on_screen(&mut app), vec![]);
    }

    /// The camera the cell was picked through leaves with the editor, so the
    /// cell has to go too - `c0007` paints what is hovered, and a stale hover
    /// left over from last time is a paint on the wrong cell.
    #[test]
    fn leaving_the_editor_forgets_the_hovered_cell() {
        let mut app = app_in_the_editor(sparse(&empty_grid(9, 6)));

        assert!(point_at(&mut app, cell_to_world(4, 3, 9, BLOCK_GAP)));
        assert_eq!(hovered(&app), Some((4, 3)));

        go_to(&mut app, GameState::InGame);

        assert_eq!(hovered(&app), None);
    }

    // --- resizing ---------------------------------------------------------

    fn arrow(edge: Edge) -> KeyCode {
        match edge {
            Edge::Top => KeyCode::ArrowUp,
            Edge::Bottom => KeyCode::ArrowDown,
            Edge::Left => KeyCode::ArrowLeft,
            Edge::Right => KeyCode::ArrowRight,
        }
    }

    /// Presses a resize shortcut and gives the editor the frame it needs to
    /// follow it.
    ///
    /// The system is run directly, as the `Escape` test has to run
    /// [`editor_leave`]: `InputPlugin` clears `just_pressed` in `PreUpdate`, so
    /// a key pressed from a test never survives to `Update`. The `update`
    /// afterwards is the rest of the editor - the blocks on screen, the warning
    /// - catching up with what the press did.
    fn press_resize(app: &mut App, edge: Edge, shrinking: bool) {
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            if shrinking {
                keys.press(KeyCode::ShiftLeft);
            }
            keys.press(arrow(edge));
        }

        app.world_mut().run_system_once(editor_resize).unwrap();

        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release_all();
        app.update();
    }

    fn grow_at(app: &mut App, edge: Edge) {
        press_resize(app, edge, false);
    }

    fn shrink_at(app: &mut App, edge: Edge) {
        press_resize(app, edge, true);
    }

    /// What the editor is warning the author about, if anything.
    fn warning(app: &mut App) -> Option<String> {
        let world = app.world_mut();
        let mut warnings = world.query_filtered::<&Text, With<EditorWarning>>();

        warnings.iter(world).next().map(|text| text.0.clone())
    }

    fn grid(app: &App) -> (usize, usize) {
        let (cols, rows, _) = editor_level(app).grid().expect("a grid to resize");

        (cols, rows)
    }

    #[test]
    fn an_arrow_key_adds_a_row_or_a_column_at_the_edge_it_points_at() {
        for (edge, grown) in [
            (Edge::Top, ".. ..\nAA BA"),
            (Edge::Bottom, "AA BA\n.. .."),
            (Edge::Left, ".. AA BA"),
            (Edge::Right, "AA BA .."),
        ] {
            let mut app = app_in_the_editor(sparse("AA BA"));

            grow_at(&mut app, edge);

            assert_eq!(layout(&app), grown, "{edge:?}");
        }
    }

    /// One block, in the middle, so that every edge is empty - what taking away
    /// an edge that holds something costs is the next test's.
    #[test]
    fn shift_and_an_arrow_key_takes_that_edge_away_again() {
        for (edge, shrunk) in [
            (Edge::Top, ".. AA ..\n.. .. .."),
            (Edge::Bottom, ".. .. ..\n.. AA .."),
            (Edge::Left, ".. ..\nAA ..\n.. .."),
            (Edge::Right, ".. ..\n.. AA\n.. .."),
        ] {
            let mut app = app_in_the_editor(sparse(".. .. ..\n.. AA ..\n.. .. .."));

            shrink_at(&mut app, edge);

            assert_eq!(layout(&app), shrunk, "{edge:?}");
        }
    }

    /// A grid that grew is a grid the author can aim at: the cell that was off
    /// the grid a moment ago is now the one the pointer finds there.
    #[test]
    fn the_pointer_finds_the_cells_a_resize_added() {
        let mut app = app_in_the_editor(sparse(&empty_grid(3, 2)));

        let new_row = cell_to_world(1, 2, 3, BLOCK_GAP);
        assert!(point_at(&mut app, new_row));
        assert_eq!(hovered(&app), None, "there is no third row yet");

        grow_at(&mut app, Edge::Bottom);

        assert_eq!(grid(&app), (3, 3));
        assert!(point_at(&mut app, new_row));
        assert_eq!(hovered(&app), Some((1, 2)), "the row that was just added");

        shrink_at(&mut app, Edge::Bottom);

        assert!(point_at(&mut app, new_row));
        assert_eq!(hovered(&app), None, "and it is gone again");
    }

    /// The card's "existing blocks keep their position relative to the cells
    /// that are retained", on screen: a row added at the top pushes every block
    /// down a cell, and one added at the bottom moves nothing.
    #[test]
    fn the_blocks_that_are_kept_keep_their_cells() {
        let mut app = app_in_the_editor(sparse("AA .. BA"));

        grow_at(&mut app, Edge::Top);

        assert_eq!(
            blocks_on_screen(&mut app),
            vec![(at(0, 1, 3), BlockType::Simple), (at(2, 1, 3), BlockType::Hardling)],
            "the same two cells, a row further down the grid"
        );

        grow_at(&mut app, Edge::Bottom);

        assert_eq!(
            blocks_on_screen(&mut app),
            vec![(at(0, 1, 3), BlockType::Simple), (at(2, 1, 3), BlockType::Hardling)],
            "a row added below them moves nothing"
        );

        grow_at(&mut app, Edge::Left);

        assert_eq!(
            blocks_on_screen(&mut app),
            vec![(at(1, 1, 4), BlockType::Simple), (at(3, 1, 4), BlockType::Hardling)],
            "a column added on the left moves them one to the right"
        );
    }

    /// An edge with blocks standing on it is not taken away by one press. That
    /// is this card's half of "warns first or is undoable" - `c0011` brings the
    /// undo that would make the second press unnecessary.
    #[test]
    fn an_edge_with_blocks_on_it_is_called_out_before_it_is_taken_away() {
        let mut app = app_in_the_editor(sparse("AA .. AA\n.. CA .."));

        shrink_at(&mut app, Edge::Top);

        assert_eq!(layout(&app), "AA .. AA\n.. CA ..", "the first press takes nothing away");
        assert_eq!(grid(&app), (3, 2));

        let warned = warning(&mut app).expect("the author has to be told what it would cost");
        assert!(warned.contains("2 blocks"), "{warned}");
        assert!(warned.contains("top row"), "{warned}");

        shrink_at(&mut app, Edge::Top);

        assert_eq!(layout(&app), ".. CA ..", "the same press again means it");
        assert_eq!(warning(&mut app), None, "and there is nothing left to warn about");
    }

    /// An edge with nothing on it costs nothing, so it goes on the first press.
    #[test]
    fn an_empty_edge_needs_no_warning() {
        let mut app = app_in_the_editor(sparse(".. .. ..\nAA CA AA"));

        shrink_at(&mut app, Edge::Top);

        assert_eq!(layout(&app), "AA CA AA");
        assert_eq!(warning(&mut app), None);
    }

    /// The warning is about the level as it stood when it was given. Anything
    /// else the author does in between makes it stale, and a stale warning must
    /// not be what a press is taken as confirming.
    #[test]
    fn a_warning_does_not_survive_the_author_doing_something_else() {
        let mut app = app_in_the_editor(sparse("AA .. AA\n.. CA .."));

        shrink_at(&mut app, Edge::Top);
        assert!(warning(&mut app).is_some());

        paint(&mut app, &[(1, 1)]);
        assert_eq!(warning(&mut app), None, "painting is not confirming");

        shrink_at(&mut app, Edge::Top);
        assert_eq!(layout(&app), "AA .. AA\n.. AA ..", "the row is still there, warned about again");
        assert!(warning(&mut app).is_some());

        // A different edge is a different question.
        shrink_at(&mut app, Edge::Left);
        assert_eq!(layout(&app), "AA .. AA\n.. AA ..");

        let warned = warning(&mut app).expect("the left column holds a block too");
        assert!(warned.contains("left column"), "{warned}");
    }

    /// Growing is not a confirmation of anything either.
    #[test]
    fn growing_the_grid_drops_a_warning_rather_than_confirming_it() {
        let mut app = app_in_the_editor(sparse("AA AA\nAA AA"));

        shrink_at(&mut app, Edge::Top);
        assert!(warning(&mut app).is_some());

        grow_at(&mut app, Edge::Bottom);

        assert_eq!(warning(&mut app), None);
        assert_eq!(grid(&app), (2, 3), "the row was added, and none was taken away");
    }

    /// The editor promises to keep the whole grid on screen, so it will not make
    /// one it cannot show.
    #[test]
    fn the_grid_stops_growing_where_the_editor_can_no_longer_show_it() {
        let mut app = app_in_the_editor(sparse("AA"));

        for _ in 0..40 {
            grow_at(&mut app, Edge::Right);
            grow_at(&mut app, Edge::Bottom);
        }

        let (cols, rows) = grid(&app);

        assert!(grid_fits_the_view(cols, rows, BLOCK_GAP), "a {cols}x{rows} grid is off screen");
        assert!(!grid_fits_the_view(cols + 1, rows, BLOCK_GAP), "it stopped short of the edge");
        assert!(!grid_fits_the_view(cols, rows + 1, BLOCK_GAP), "it stopped short of the edge");

        // The shipped levels are 3 to 11 columns wide, so the editor has room
        // for every one of them and then some.
        assert_eq!((cols, rows), (13, 16));
    }

    /// A grid always keeps a cell: there would be nothing left to aim at, and no
    /// way back.
    #[test]
    fn the_last_row_and_the_last_column_stay() {
        let mut app = app_in_the_editor(sparse("AA"));

        for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
            shrink_at(&mut app, edge);

            assert_eq!(layout(&app), "AA", "{edge:?}");
            assert_eq!(warning(&mut app), None, "{edge:?}: nothing to warn about, nothing happens");
        }
    }

    /// Resizing a `FilledGrid` is the same trade painting one is: a grid with a
    /// row of empty cells on it is no longer "the same block everywhere".
    #[test]
    fn resizing_a_filled_grid_turns_it_into_a_token_grid() {
        let filled = LevelDefinition {
            targets: FilledGrid(3, 2, BlockType::Hardling, BlockBehaviour::Vanisher, BLOCK_GAP),
            ..default()
        };

        let mut app = app_in_the_editor(filled);

        grow_at(&mut app, Edge::Bottom);

        assert_eq!(layout(&app), "BC BC BC\nBC BC BC\n.. .. ..");
        assert_eq!(blocks_on_screen(&mut app).len(), 6, "the blocks that were there stayed");
    }

    /// A refused resize leaves the level exactly as it was - a `FilledGrid` that
    /// cannot grow is still a `FilledGrid`.
    #[test]
    fn a_resize_that_does_not_happen_changes_nothing() {
        let filled = LevelDefinition {
            targets: FilledGrid(1, 1, BlockType::Hardling, BlockBehaviour::Vanisher, BLOCK_GAP),
            ..default()
        };

        let mut app = app_in_the_editor(filled.clone());

        shrink_at(&mut app, Edge::Left);

        assert_eq!(editor_level(&app).level, filled);
    }

    /// A `Custom` level is built in code: there is no grid to add a row to.
    #[test]
    fn a_level_that_is_not_a_grid_cannot_be_resized() {
        let custom = LevelDefinition {
            targets: Custom("Conveyor".to_string()),
            ..default()
        };

        let mut app = app_in_the_editor(custom.clone());

        for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
            grow_at(&mut app, edge);
            shrink_at(&mut app, edge);
        }

        assert_eq!(editor_level(&app).level, custom);
        assert_eq!(warning(&mut app), None);
    }

    #[test]
    fn leaving_the_editor_forgets_the_warning() {
        let mut app = app_in_the_editor(sparse("AA AA\nAA AA"));

        shrink_at(&mut app, Edge::Top);
        assert!(warning(&mut app).is_some());

        go_to(&mut app, GameState::InGame);

        assert_eq!(app.world().resource::<PendingRemoval>(), &PendingRemoval(None));
        assert_eq!(warning(&mut app), None, "the warning went with the rest of the editor");
    }

    /// The card's last criterion. Saving is `c0012`'s, so this is the trip a
    /// save will make: the resized level written as RON, read back off disk, and
    /// read out again as the blocks a match would spawn.
    #[test]
    fn a_resized_level_saves_and_reloads_and_plays() {
        let mut app = app_in_the_editor(sparse("AA .. AA\n.. CA ..\nZA .. ZA"));

        grow_at(&mut app, Edge::Right);
        grow_at(&mut app, Edge::Top);

        // The bottom row holds two obstacles, so it takes the second press.
        shrink_at(&mut app, Edge::Bottom);
        shrink_at(&mut app, Edge::Bottom);

        let edited = editor_level(&app).level.clone();
        assert_eq!(grid(&app), (4, 3));

        let path = std::env::temp_dir().join("angleout_c0008_resized.ron");
        fs::write(&path, campaign::level_to_ron(&edited).expect("a level has to serialize"))
            .expect("writing the level");

        let reloaded = campaign::load_level(&path).expect("what was written has to read back");
        fs::remove_file(&path).ok();

        assert_eq!(reloaded, edited, "the resized level survives the trip through disk");

        let SparseGrid(reloaded_layout, gap) = &reloaded.targets else {
            panic!("a resized level is a token grid: {:?}", reloaded.targets);
        };

        let played: Vec<(Vec3, BlockType)> = interpret_grid(reloaded_layout, *gap)
            .expect("the grid a match spawns from")
            .iter()
            .map(|block| (Vec3::new(block.position.x, 0.0, block.position.y), block.block_type.clone()))
            .collect();

        let mut played = played;
        played.sort_by(|a, b| a.0.to_array().partial_cmp(&b.0.to_array()).unwrap());

        assert_eq!(
            played,
            blocks_on_screen(&mut app),
            "the blocks a match spawns have to be the blocks the editor was showing"
        );
    }



    // --- the settings panel -----------------------------------------------

    /// A real mouse press, as the window reports one.
    ///
    /// `InputPlugin` clears `just_pressed` at the top of every frame and fills
    /// it in again from these messages, so a test that only calls
    /// `ButtonInput::press` never has one to offer the frame it matters in.
    fn report_the_button(app: &mut App, button: MouseButton, state: ButtonState) {
        let window = {
            let world = app.world_mut();
            let mut windows = world.query_filtered::<Entity, With<PrimaryWindow>>();
            windows.iter(world).next().expect("the test app has a window")
        };

        app.world_mut().write_message(MouseButtonInput { button, state, window });
    }

    /// A click wherever the pointer is: press, a frame, release, a frame.
    fn click(app: &mut App) {
        report_the_button(app, MouseButton::Left, ButtonState::Pressed);
        app.update();
        report_the_button(app, MouseButton::Left, ButtonState::Released);
        app.update();
    }

    fn button_of(setting: Setting, by: i32) -> Rect {
        let row = settings_rows()
            .into_iter()
            .find(|row| row.setting == setting)
            .expect("every setting has a row");

        if by < 0 { row.down } else { row.up }
    }

    /// Clicks one of the panel's buttons the way an author does: pointer on it,
    /// press, release.
    fn click_setting(app: &mut App, setting: Setting, by: i32) {
        put_the_pointer_at(app, Some(button_of(setting, by).center()));
        click(app);
    }

    /// What the panel is showing for a setting, read off the screen rather than
    /// out of the level - which is the only way to tell that the two agree.
    fn shown_value(app: &mut App, setting: Setting) -> String {
        let world = app.world_mut();
        let mut values = world.query::<(&SettingValue, &Text)>();

        values
            .iter(world)
            .find(|(value, _)| value.0 == setting)
            .map(|(_, text)| text.0.clone())
            .unwrap_or_else(|| panic!("{setting:?} is not on screen"))
    }

    fn panel_parts(app: &mut App) -> usize {
        let world = app.world_mut();
        let mut parts = world.query_filtered::<Entity, With<SettingsPanel>>();

        parts.iter(world).count()
    }

    #[test]
    fn the_settings_panel_is_up_while_the_editor_is_and_goes_with_it() {
        let mut app = editor_app();

        go_to(&mut app, GameState::Editor);
        assert!(panel_parts(&mut app) > 0, "the editor has to have drawn the panel");

        go_to(&mut app, GameState::InGame);
        assert_eq!(panel_parts(&mut app), 0, "the panel goes with the rest of the editor");
    }

    /// The panel opens saying what the level says - all of it, including the
    /// fields no shipped level sets.
    #[test]
    fn every_setting_is_on_screen_saying_what_the_level_holds() {
        let level = LevelDefinition {
            background_asset: "ship3_003.glb#Scene12".to_string(),
            background_scroll_velocity: 20.0,
            simultaneous_balls: 3,
            win_criteria: WinCriteria::BlockHitPercentage(0.5),
            global_pickups: vec![PickupType::MoreBalls(1), PickupType::MoreBalls(1)],
            default_wall_r: false,
            targets: SparseGrid("AA AA".to_string(), BLOCK_GAP),
            ..default()
        };

        let mut app = app_in_the_editor(level.clone());

        for setting in SETTINGS {
            assert_eq!(
                shown_value(&mut app, setting),
                setting.value(&level),
                "{setting:?} on screen"
            );
        }
    }

    /// The card's "changes apply to the in-memory level immediately": the click
    /// is the edit, and the panel says the new thing on the same frame.
    #[test]
    fn clicking_a_button_steps_its_setting_and_the_panel_says_so() {
        let mut app = app_in_the_editor(sparse(&empty_grid(3, 2)));

        click_setting(&mut app, Setting::SimultaneousBalls, 1);
        assert_eq!(editor_level(&app).level.simultaneous_balls, 2);
        assert_eq!(shown_value(&mut app, Setting::SimultaneousBalls), "2");

        click_setting(&mut app, Setting::SimultaneousBalls, -1);
        assert_eq!(editor_level(&app).level.simultaneous_balls, 1);
        assert_eq!(shown_value(&mut app, Setting::SimultaneousBalls), "1");

        click_setting(&mut app, Setting::Grabbers, 1);
        assert_eq!(editor_level(&app).level.global_pickups, vec![PickupType::Grabber(1)]);
        assert_eq!(shown_value(&mut app, Setting::Grabbers), "1");

        click_setting(&mut app, Setting::WallLeft, -1);
        assert!(!editor_level(&app).level.default_wall_l);
        assert_eq!(shown_value(&mut app, Setting::WallLeft), "off");
    }

    /// One click is one step. A stepper walked once per frame the button is down
    /// would run the whole range in a fifth of a second.
    #[test]
    fn holding_a_button_down_steps_its_setting_once() {
        let mut app = app_in_the_editor(sparse(&empty_grid(3, 2)));

        put_the_pointer_at(&mut app, Some(button_of(Setting::SimultaneousBalls, 1).center()));
        report_the_button(&mut app, MouseButton::Left, ButtonState::Pressed);

        for _ in 0..10 {
            app.update();
        }

        assert_eq!(editor_level(&app).level.simultaneous_balls, 2);
    }

    /// A click on the panel is aimed at a setting, so it must not also be a
    /// stroke of paint on whatever the panel happens to be covering.
    #[test]
    fn a_click_on_the_panel_paints_nothing() {
        let mut app = app_in_the_editor(sparse(&empty_grid(3, 2)));
        let before = layout(&app);

        click_setting(&mut app, Setting::ExtraBalls, 1);

        assert_eq!(layout(&app), before, "the grid is not what the click was aimed at");
        assert_eq!(editor_level(&app).level.global_pickups.len(), 1, "and the setting did move");
    }

    /// The panel is drawn in front of the play field, and on a window narrow
    /// enough it covers cells. The pointer over it is over the panel, not over
    /// the cell underneath - so the highlight goes too, rather than the panel
    /// sitting on top of a cell that still looks armed.
    #[test]
    fn the_pointer_over_the_panel_is_not_over_a_cell() {
        let mut app = app_in_the_editor(sparse(&empty_grid(9, 6)));

        // Tall and narrow: the camera keeps the whole play field on screen
        // whatever the window's shape, which on this one puts the far corner of
        // the grid behind the panel.
        resize_the_window(&mut app, UVec2::new(400, 800));

        let mut covered = 0;
        let mut clear = 0;

        for (col, row) in cells(9, 6) {
            let centre = cell_to_world(col, row, 9, BLOCK_GAP);

            if !point_at(&mut app, centre) {
                continue;
            }

            let pixel = cursor(&mut app).expect("the pointer was just put on screen");

            if panel_rect().contains(pixel) {
                assert_eq!(hovered(&app), None, "cell ({col}, {row}) is behind the panel");
                covered += 1;
            } else {
                assert_eq!(hovered(&app), Some((col, row)), "cell ({col}, {row}) is in the open");
                clear += 1;
            }
        }

        assert!(covered > 0, "no cell ended up behind the panel - this proves nothing");
        assert!(clear > 0, "every cell ended up behind the panel - so does this");
    }

    fn cursor(app: &mut App) -> Option<Vec2> {
        let world = app.world_mut();
        let mut windows = world.query_filtered::<&Window, With<PrimaryWindow>>();

        windows.iter(world).next().and_then(|window| window.cursor_position())
    }

    /// The card's other half: what the panel makes, the level file says. This is
    /// the whole path - click, level, RON, back - rather than the level built by
    /// hand that `settings::tests` round-trips.
    #[test]
    fn a_level_edited_through_the_panel_round_trips_through_ron() {
        let mut app = app_in_the_editor(sparse("AA .. AA\n.. AA .."));

        for setting in SETTINGS {
            click_setting(&mut app, setting, 1);
        }

        click_setting(&mut app, Setting::WallLeft, -1);

        let edited = editor_level(&app).level.clone();
        assert_ne!(edited, sparse("AA .. AA\n.. AA .."), "the clicks have to have changed something");

        let written = campaign::level_to_ron(&edited).expect("a level the panel made has to be writable");
        let read_back = campaign::parse_level(&written).unwrap_or_else(|e| panic!("{e}\n{written}"));

        assert_eq!(read_back, edited, "written as:\n{written}");
    }

}
