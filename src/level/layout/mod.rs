use bevy::math::{Rect, Vec2};
use bevy::prelude::default;
use crate::block::{Block, BlockBehaviour, BlockType};
use crate::block::trigger::{TriggerGroup, TriggerType};

use crate::config::{BLOCK_DEPTH, BLOCK_WIDTH, BLOCK_WIDTH_H};

/// Half a cell, as the offset from a cell centre to its corner.
fn cell_half_extent() -> Vec2 {
    Vec2::new(BLOCK_WIDTH_H, BLOCK_DEPTH / 2.0)
}


/// World position of the centre of grid cell (`col`, `row`) in a grid `cols`
/// columns wide with `gap` between cells.
///
/// This is the single place a block's world position is derived from its grid
/// coordinates - `generate_block_grid`, `interpret_grid` and the editor all go
/// through it, so they cannot disagree about where a cell is.
///
/// Row 0 is the bottom row, column 0 the leftmost. The grid is centred on x = 0,
/// with an even column count offset by half a cell so the seam - not a cell
/// centre - sits on the axis.
///
/// The steps are accumulated rather than multiplied on purpose: the two loops
/// this replaces added them up, and `y0 + row as f32 * y_step` differs from that
/// by an ULP or two from row 2 on. Stepping keeps every shipped level's block
/// positions bit-identical to what they were before this became shared.
pub fn cell_to_world(col: usize, row: usize, cols: usize, gap: f32) -> Vec2 {
    let x_step = BLOCK_WIDTH + gap;
    let y_step = BLOCK_DEPTH + gap;

    let cols_h = (cols / 2) as f32;

    let mut x = 0.0;
    if cols % 2 == 1 {
        x -= cols_h * x_step;
    } else {
        x -= cols_h * x_step - gap / 2.0 - BLOCK_WIDTH_H;
    }
    for _ in 0..col {
        x += x_step;
    }

    let mut y = -30.0 - 4.0 * y_step;
    for _ in 0..row {
        y += y_step;
    }

    Vec2::new(x, y)
}


/// The inverse of [`cell_to_world`]: the cell a world position falls in, or
/// `None` if it falls outside the grid.
///
/// A position is assigned to the cell whose centre is nearest, so the gap
/// between two cells belongs to the nearer of them. The grid is bounded left and
/// right by `cols` and below by row 0; it is open upwards, since the caller owns
/// the row count and a level can grow rows.
pub fn world_to_cell(pos: Vec2, cols: usize, gap: f32) -> Option<(usize, usize)> {
    let x_step = BLOCK_WIDTH + gap;
    let y_step = BLOCK_DEPTH + gap;

    let origin = cell_to_world(0, 0, cols, gap);

    let col = ((pos.x - origin.x) / x_step).round();
    let row = ((pos.y - origin.y) / y_step).round();

    if col < 0.0 || row < 0.0 || col >= cols as f32 {
        return None;
    }

    Some((col as usize, row as usize))
}


/// The rectangle a `cols` x `rows` grid covers on the ground plane, cell edges
/// included.
///
/// This is the area an editor has to keep on screen, so it is the outer edge of
/// the outermost cells rather than the centres [`cell_to_world`] returns. A grid
/// with no cells has no extent, and collapses to the point cell (0, 0) would
/// occupy.
pub fn grid_bounds(cols: usize, rows: usize, gap: f32) -> Rect {
    let origin = cell_to_world(0, 0, cols.max(1), gap);

    if cols == 0 || rows == 0 {
        return Rect::from_corners(origin, origin);
    }

    let far = cell_to_world(cols - 1, rows - 1, cols, gap);

    Rect::from_corners(origin - cell_half_extent(), far + cell_half_extent())
}


/// The size of a token grid, as (columns, rows).
///
/// Columns come off the first line, because that is where [`interpret_grid`]
/// takes them from - a ragged grid is the writer's problem, not this one's.
/// Lines with nothing on them are not rows: a layout written with a trailing
/// newline is as many rows as it looks like.
pub fn grid_dimensions(layout: &str) -> (usize, usize) {
    let rows: Vec<&str> = layout
        .split('\n')
        .filter(|line| slots(line).next().is_some())
        .collect();

    let cols = rows.first().map_or(0, |line| slots(line).count());

    (cols, rows.len())
}


/// A token grid of `cols` x `rows` empty cells - what a new level starts as.
pub fn empty_grid(cols: usize, rows: usize) -> String {
    let row = vec![EMPTY_SLOT; cols].join(" ");

    (0..rows).map(|_| row.as_str()).collect::<Vec<&str>>().join("\n")
}


/// The token for a cell with no block in it.
pub const EMPTY_SLOT: &str = "..";


/// The slots of one line of a token grid, skipping the padding that separates
/// them - the same thing [`interpret_grid`] walks.
fn slots(line: &str) -> impl Iterator<Item = &str> {
    line.split(' ').filter(|slot| slot.len() >= 2)
}


pub fn generate_block_grid(
    rows: usize,
    cols: usize,
    gap: f32,
)   -> Vec<Vec2>

{
    let mut res = vec![];

    for row in 0..rows {
        for col in 0..cols {
            res.push(cell_to_world(col, row, cols, gap));
        }
    }

    res
}


/// Reads one block off the token grid, or `None` for an empty slot.
///
/// Each 2 to 4 character tuple describes one block, `..` is an empty slot, and
/// slots are separated by spaces.
///
/// 1st character - how many hits can a block take:
///   * `A` = 1
///   * `B` = 2
///   * `C` = 3
///   * `D` = 1, only from the top
///   * `Z` = unbreakable
///
/// `Z` is used for obstacles and does not count as a block when determining
/// whether the player has finished the level.
///
/// 2nd character - what behaviour does the block have:
///   * `A` - Nothing
///   * `B` - Spinner - which is kinda useless I guess
///   * `C` - Vanisher - questionable as well
///   * `D` - Repulsor
///   * `E` - Evader, first movement to the right
///   * `F` - Evader, first movement to the left
///   * `G` - Evader, first movement up
///   * `H` - Evader, first movement down
///   * `I` - Portal - use this as a trigger target. Teleports the ball from the
///     trigger to itself, preserving momentum
///
/// 3rd character (optional) - trigger type:
///   * `A` - Start trigger
///   * `B` - Stop trigger
///   * `C` - StartStop trigger
///   * `R` - Receiver that starts stopped
///   * `S` - Receiver that starts started
///
/// 4th character (mandatory if the 3rd exists) - trigger group, `0..=9`.
pub fn make_block(b_type: char, b_beh: char, b_trigger: Option<char>, b_trigger_group: Option<char>, pos: Vec2) -> Option<Block> {
    let t = match b_type  {
        'A' => BlockType::Simple,
        'B' => BlockType::Hardling,
        'C' => BlockType::Concrete,
        'D' => BlockType::SimpleTop,
        'Z' => BlockType::Obstacle,

        '.' => return None,

        _ => BlockType::Simple
    };

    let b = match b_beh {
        'A' => BlockBehaviour::SittingDuck,
        'B' => BlockBehaviour::Spinner,
        'C' => BlockBehaviour::Vanisher,
        'D' => BlockBehaviour::Repuslor,
        'E' => BlockBehaviour::EvaderR(50.0),
        'F' => BlockBehaviour::EvaderL(50.0),
        'G' => BlockBehaviour::EvaderU(50.0),
        'H' => BlockBehaviour::EvaderD(50.0),
        'I' => BlockBehaviour::Portal,

        '.' => return None,

        _ => BlockBehaviour::SittingDuck
    };

    let tt = if let Some(t) = b_trigger {
      match t {
          'A' => Some(TriggerType::Start),
          'B' => Some(TriggerType::Stop),
          'C' => Some(TriggerType::StartStop),
          'R' => Some(TriggerType::ReceiverStartingInactive),
          'S' => Some(TriggerType::ReceiverStartingActive),

          _ => None
      }
    } else {
        None
    };

    let tg = if let Some(g) = b_trigger_group {
      match g {

          '0'..='9' => Some((g as TriggerGroup) - 48),

          _ => None
      }
    } else {
        None
    };

    Some(Block {
        behaviour: b,
        block_type: t,
        position: pos,
        trigger_type: tt,
        trigger_group: tg,
        ..default()
    })
}


/// The token [`make_block`] would read this block back out of - the inverse of
/// the format documented above.
///
/// The trigger is a type *and* a group or it is neither: a trigger character
/// with no group character behind it is a token [`make_block`] cannot finish
/// reading, so a group the format has no room for drops the trigger rather than
/// writing half of one.
///
/// Evader speeds do not survive the trip, because the format has no room for
/// them either - [`make_block`] reads every evader back at 50.0, which is the
/// speed every authored level has always had.
pub fn block_token(
    block_type: &BlockType,
    behaviour: &BlockBehaviour,
    trigger: Option<(&TriggerType, TriggerGroup)>,
) -> String {
    let mut token = String::with_capacity(4);

    token.push(match block_type {
        BlockType::Simple => 'A',
        BlockType::Hardling => 'B',
        BlockType::Concrete => 'C',
        BlockType::SimpleTop => 'D',
        BlockType::Obstacle => 'Z',
    });

    token.push(match behaviour {
        BlockBehaviour::SittingDuck => 'A',
        BlockBehaviour::Spinner => 'B',
        BlockBehaviour::Vanisher => 'C',
        BlockBehaviour::Repuslor => 'D',
        BlockBehaviour::EvaderR(_) => 'E',
        BlockBehaviour::EvaderL(_) => 'F',
        BlockBehaviour::EvaderU(_) => 'G',
        BlockBehaviour::EvaderD(_) => 'H',
        BlockBehaviour::Portal => 'I',
    });

    if let Some((trigger_type, group)) = trigger {
        if let Some(group) = char::from_digit(group as u32, 10) {
            token.push(match trigger_type {
                TriggerType::Start => 'A',
                TriggerType::Stop => 'B',
                TriggerType::StartStop => 'C',
                TriggerType::ReceiverStartingInactive => 'R',
                TriggerType::ReceiverStartingActive => 'S',
            });
            token.push(group);
        }
    }

    token
}


/// The layout with cell (`col`, `row`) written as `token`.
///
/// Every row comes back the same width, padded with [`EMPTY_SLOT`] out to the
/// widest one: [`interpret_grid`] takes the whole grid's column count off the
/// first line, so a ragged grid centres its rows on a width they do not have.
/// Painting a ragged layout squares it up, which moves its blocks - that is the
/// repair, not the damage.
///
/// A write outside the grid is not a write: the layout comes back untouched.
/// Growing the grid is [`grow`]'s job, not the brush's.
pub fn set_cell(layout: &str, col: usize, row: usize, token: &str) -> String {
    let mut rows = slot_grid(layout);

    if row >= rows.len() || col >= rows[row].len() {
        return layout.to_string();
    }

    rows[row][col] = token;

    write_grid(&rows)
}


/// One side of the grid, as the author looking at it names it.
///
/// [`Edge::Top`] is row 0 - the first line of the layout, the far end of the
/// arena, and the top of the screen under the editor's camera and the game's
/// alike. [`cell_to_world`] calls row 0 the bottom row because it works in world
/// coordinates, where z grows towards the player; it is the same row, named from
/// the other side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Top,
    Bottom,
    Left,
    Right,
}


/// The layout as rows of slots, squared up: every row padded with [`EMPTY_SLOT`]
/// out to the width of the widest one.
///
/// This is how every write to the grid reads it, which is what keeps the grids
/// they write square - [`interpret_grid`] takes the whole grid's column count
/// off the first line, so a ragged grid centres its rows on a width they do not
/// have. A layout squared up here has moved the blocks of its short rows; that
/// is the repair of a grid that was already lying about where they were.
fn slot_grid(layout: &str) -> Vec<Vec<&str>> {
    let mut rows: Vec<Vec<&str>> = layout
        .split('\n')
        .map(|line| slots(line).collect::<Vec<&str>>())
        .filter(|row| !row.is_empty())
        .collect();

    let width = rows.iter().map(Vec::len).max().unwrap_or(0);

    for row in &mut rows {
        row.resize(width, EMPTY_SLOT);
    }

    rows
}


/// The inverse of [`slot_grid`]: rows of slots as a token grid again.
fn write_grid(rows: &[Vec<&str>]) -> String {
    rows.iter()
        .map(|row| row.join(" "))
        .collect::<Vec<String>>()
        .join("\n")
}


/// Whether a slot holds a block, or is one of the ways of writing an empty cell.
///
/// [`make_block`] reads a `.` in either of the first two characters as nothing
/// there, so this has to agree with it: what is counted here is what would
/// disappear off the screen.
fn slot_holds_a_block(slot: &str) -> bool {
    let mut chars = slot.chars();

    !matches!(chars.next(), Some('.') | None) && !matches!(chars.next(), Some('.') | None)
}


/// The layout with one more row or column, added at `edge`.
///
/// The new cells are empty, and every cell that was there is still there - what
/// moves is the *index* of the cells beyond the new one, since a row added at
/// the top pushes the rest of the grid down one.
///
/// A grid with no cells in it grows into a single cell whichever side it is
/// grown from: there is no row to widen and no width to add a row of.
pub fn grow(layout: &str, edge: Edge) -> String {
    let mut rows = slot_grid(layout);

    if rows.is_empty() {
        return EMPTY_SLOT.to_string();
    }

    let width = rows[0].len();

    match edge {
        Edge::Top => rows.insert(0, vec![EMPTY_SLOT; width]),
        Edge::Bottom => rows.push(vec![EMPTY_SLOT; width]),
        Edge::Left => for row in &mut rows { row.insert(0, EMPTY_SLOT) },
        Edge::Right => for row in &mut rows { row.push(EMPTY_SLOT) },
    }

    write_grid(&rows)
}


/// The layout with the row or column at `edge` taken away, and whatever was
/// standing on it with it.
///
/// The last row and the last column stay: a grid shrunk away to nothing would
/// have nothing left on screen to aim at, and no cell to grow back from.
pub fn shrink(layout: &str, edge: Edge) -> String {
    let mut rows = slot_grid(layout);

    if !can_shrink(rows.first().map_or(0, Vec::len), rows.len(), edge) {
        return layout.to_string();
    }

    match edge {
        Edge::Top => { rows.remove(0); }
        Edge::Bottom => { rows.pop(); }
        Edge::Left => for row in &mut rows { row.remove(0); },
        Edge::Right => for row in &mut rows { row.pop(); },
    }

    write_grid(&rows)
}


/// Whether a `cols` x `rows` grid has an `edge` to spare.
pub fn can_shrink(cols: usize, rows: usize, edge: Edge) -> bool {
    match edge {
        Edge::Top | Edge::Bottom => rows > 1,
        Edge::Left | Edge::Right => cols > 1,
    }
}


/// How many blocks are standing on `edge` - what taking it away would cost.
pub fn blocks_on_edge(layout: &str, edge: Edge) -> usize {
    let rows = slot_grid(layout);

    let counted: Vec<&str> = match edge {
        Edge::Top => rows.first().cloned().unwrap_or_default(),
        Edge::Bottom => rows.last().cloned().unwrap_or_default(),
        Edge::Left => rows.iter().filter_map(|row| row.first().copied()).collect(),
        Edge::Right => rows.iter().filter_map(|row| row.last().copied()).collect(),
    };

    counted.iter().filter(|slot| slot_holds_a_block(slot)).count()
}


/// A `cols` x `rows` token grid of the same block over and over - what a
/// [`FilledGrid`](crate::level::TargetLayout::FilledGrid) says, written out in
/// the format, so single cells of it can be painted and drawn.
pub fn filled_grid(
    cols: usize,
    rows: usize,
    block_type: &BlockType,
    behaviour: &BlockBehaviour,
) -> String {
    let token = block_token(block_type, behaviour, None);
    let row = vec![token.as_str(); cols].join(" ");

    (0..rows).map(|_| row.as_str()).collect::<Vec<&str>>().join("\n")
}


pub fn interpret_grid(layout: &String, gap: f32) -> Option<Vec<Block>> {

    let mut res = vec![];

    let lines: Vec<&str> = layout.split("\n").collect();

    let first = lines.get(0).unwrap().split(" ");

    let cols = first.collect::<Vec<&str>>().len();

    for (row, line) in lines.iter().enumerate() {

        let slots = line.split(" ");

        let mut col = 0;

        for slot in slots {
            if slot.len() < 2 {
                continue;
            }

            let pos = cell_to_world(col, row, cols, gap);
            col += 1;

            let b_type = slot.chars().nth(0).unwrap();
            let b_beh = slot.chars().nth(1).unwrap();
            let b_trigger_type = slot.chars().nth(2);
            let b_trigger_group = slot.chars().nth(3);

            match make_block(b_type, b_beh, b_trigger_type, b_trigger_group, pos) {
                None => {}
                Some(block) => res.push(block)
            }
        }
    }

    Some(res)
}




#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::BLOCK_GAP;

    /// `generate_block_grid` exactly as it read before `cell_to_world` existed.
    fn legacy_block_grid(rows: usize, cols: usize, gap: f32) -> Vec<Vec2> {
        let mut y = -30.0 - 4.0 * (BLOCK_DEPTH + &gap);
        let y_step = BLOCK_DEPTH + &gap;

        let x_step = BLOCK_WIDTH + &gap;
        let cols_h = (cols / 2) as f32;

        let mut res = vec![];

        for _ in 0..rows {
            let mut x = 0.0;
            if &cols % 2 == 1 {
                x -= &cols_h * &x_step;
            } else {
                x -= &cols_h * &x_step - &gap / 2.0 - BLOCK_WIDTH_H;
            }

            for _ in 0..cols {
                res.push(Vec2::new(x, y));

                x += &x_step;
            }

            y += &y_step;
        };

        res
    }

    /// The positions `interpret_grid` produced before it shared the conversion.
    fn legacy_layout_positions(layout: &String, gap: f32) -> Vec<Vec2> {
        let mut res = vec![];

        let lines: Vec<&str> = layout.split("\n").collect();
        let first = lines.get(0).unwrap().split(" ");
        let cols = first.collect::<Vec<&str>>().len();

        let x_step = BLOCK_WIDTH + gap;
        let cols_h = (cols / 2) as f32;

        let mut y = -30.0 - 4.0 * (BLOCK_DEPTH + &gap);
        let y_step = BLOCK_DEPTH + &gap;

        for line in lines {
            let mut x = 0.0;
            if &cols % 2 == 1 {
                x -= &cols_h * &x_step;
            } else {
                x -= &cols_h * &x_step - &gap / 2.0 - BLOCK_WIDTH_H;
            }

            for slot in line.split(" ") {
                if slot.len() < 2 {
                    continue;
                }

                let pos_x = x;
                x += &x_step;

                if make_block(
                    slot.chars().nth(0).unwrap(),
                    slot.chars().nth(1).unwrap(),
                    slot.chars().nth(2),
                    slot.chars().nth(3),
                    Vec2::new(pos_x, y),
                ).is_some() {
                    res.push(Vec2::new(pos_x, y));
                }
            }
            y += y_step;
        }

        res
    }

    /// Layouts shaped like the shipped levels: odd and even widths, the leading
    /// space on continuation lines, triggers, obstacles and empty slots.
    fn sample_layouts() -> Vec<String> {
        vec![
"AA .. .. .. .. .. .. .. .. AA
 .. .. .. .. .. .. .. .. .. ..
 .. .. .. .. AB .. .. .. .. ..
 .. .. .. .. .. .. .. .. .. ..
 AA .. .. .. .. .. .. .. .. AA".to_string(),

"AA AB AA AB AA AB AA
 CA .. ZA .. ZA .. CA
 AA AIA1 .. .. .. AAR1 AA".to_string(),

"AA AH AA".to_string(),

"AA AA AA AA AA AA AA AA AA AA AA".to_string(),

"BA".to_string(),
        ]
    }

    #[test]
    fn works() {
        let a_level =
"AA .. .. .. .. .. .. .. .. AA
 .. .. .. .. .. .. .. .. .. ..
 .. .. .. .. AB .. .. .. .. ..
 .. .. .. .. .. .. .. .. .. ..
 AA .. .. .. .. .. .. .. .. AA".to_string();

        if let Some(res) = interpret_grid(&a_level, 10.0) {
            assert_eq!(res.len(), 5);

            for b in res {
                println!("{:?}", b);
            }
        } else {
            assert!(false)
        }



    }

    #[test]
    fn round_trips_every_cell_of_an_odd_and_an_even_grid() {
        for cols in [11, 10] {
            for row in 0..8 {
                for col in 0..cols {
                    let pos = cell_to_world(col, row, cols, BLOCK_GAP);

                    assert_eq!(
                        world_to_cell(pos, cols, BLOCK_GAP),
                        Some((col, row)),
                        "cell ({col}, {row}) of a {cols} wide grid at {pos:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn round_trips_for_every_column_count_a_level_can_have() {
        for cols in 1..=12 {
            for row in 0..4 {
                for col in 0..cols {
                    let pos = cell_to_world(col, row, cols, BLOCK_GAP);

                    assert_eq!(world_to_cell(pos, cols, BLOCK_GAP), Some((col, row)));
                }
            }
        }
    }

    #[test]
    fn world_to_cell_snaps_to_the_nearest_cell_centre() {
        let cols = 7;
        let step = BLOCK_WIDTH + BLOCK_GAP;
        let centre = cell_to_world(3, 2, cols, BLOCK_GAP);

        // Anywhere inside the block, and in the gap either side of it.
        for dx in [-0.49 * step, -0.2 * step, 0.0, 0.2 * step, 0.49 * step] {
            assert_eq!(
                world_to_cell(Vec2::new(centre.x + dx, centre.y), cols, BLOCK_GAP),
                Some((3, 2))
            );
        }
    }

    #[test]
    fn world_to_cell_rejects_positions_outside_the_grid() {
        let cols = 10;
        let x_step = BLOCK_WIDTH + BLOCK_GAP;
        let y_step = BLOCK_DEPTH + BLOCK_GAP;

        let left = cell_to_world(0, 0, cols, BLOCK_GAP);
        let right = cell_to_world(cols - 1, 0, cols, BLOCK_GAP);

        assert_eq!(world_to_cell(Vec2::new(left.x - x_step, left.y), cols, BLOCK_GAP), None);
        assert_eq!(world_to_cell(Vec2::new(right.x + x_step, right.y), cols, BLOCK_GAP), None);
        assert_eq!(world_to_cell(Vec2::new(left.x, left.y - y_step), cols, BLOCK_GAP), None);
    }

    #[test]
    fn filled_grid_positions_are_unchanged() {
        for cols in 1..=12 {
            for rows in 0..=8 {
                assert_eq!(
                    generate_block_grid(rows, cols, BLOCK_GAP),
                    legacy_block_grid(rows, cols, BLOCK_GAP),
                    "{rows}x{cols} grid"
                );
            }
        }
    }

    #[test]
    fn sparse_grid_positions_are_unchanged() {
        for layout in sample_layouts() {
            let positions: Vec<Vec2> = interpret_grid(&layout, BLOCK_GAP)
                .unwrap()
                .iter()
                .map(|b| b.position)
                .collect();

            assert_eq!(positions, legacy_layout_positions(&layout, BLOCK_GAP), "layout:\n{layout}");
        }
    }

    /// The bounds have to hold every cell of the grid whole, or an editor
    /// framed on them would clip the outermost blocks in half.
    #[test]
    fn grid_bounds_hold_every_cell_of_the_grid() {
        for cols in 1..=12 {
            for rows in 1..=8 {
                let bounds = grid_bounds(cols, rows, BLOCK_GAP);

                for row in 0..rows {
                    for col in 0..cols {
                        let centre = cell_to_world(col, row, cols, BLOCK_GAP);

                        for corner in [Vec2::new(1.0, 1.0), Vec2::new(-1.0, -1.0)] {
                            let point = centre + corner * cell_half_extent();

                            assert!(
                                bounds.contains(point),
                                "{cols}x{rows} grid: cell ({col}, {row}) corner {point:?} outside {bounds:?}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn grid_bounds_of_a_grid_with_no_cells_have_no_extent() {
        let bounds = grid_bounds(0, 0, BLOCK_GAP);

        assert_eq!(bounds.size(), Vec2::ZERO);
    }

    #[test]
    fn grid_dimensions_read_the_shape_of_a_layout() {
        assert_eq!(grid_dimensions("AA AA AA"), (3, 1));

        assert_eq!(
            grid_dimensions(
"AA .. AA
 .. AA ..
 AA .. AA"),
            (3, 3)
        );

        // The trailing newline `level0.ron` is written with is not a row.
        assert_eq!(grid_dimensions("AA AA\n AA AA\n"), (2, 2));

        // Triggers make a slot longer without making it another column.
        assert_eq!(grid_dimensions("ZIR1 .. AAA1"), (3, 1));

        assert_eq!(grid_dimensions(""), (0, 0));
    }

    /// What `grid_dimensions` reports has to be what the grid actually spawns
    /// as, or the editor and the game would disagree about where a cell is.
    #[test]
    fn grid_dimensions_agree_with_the_grid_that_gets_spawned() {
        for layout in sample_layouts() {
            let (cols, rows) = grid_dimensions(&layout);
            let bounds = grid_bounds(cols, rows, BLOCK_GAP);

            for block in interpret_grid(&layout, BLOCK_GAP).unwrap() {
                assert!(
                    bounds.contains(block.position),
                    "block at {:?} outside the {cols}x{rows} bounds {bounds:?} of:\n{layout}",
                    block.position
                );
            }
        }
    }

    /// Every token the format defines, read in and written back out again. If
    /// `block_token` and `make_block` ever disagree about a letter, an author's
    /// level changes shape the next time they touch a cell of it.
    #[test]
    fn every_token_the_format_defines_survives_the_round_trip() {
        let mut checked = 0;

        for block_type in "ABCDZ".chars() {
            for behaviour in "ABCDEFGHI".chars() {
                let mut triggers: Vec<Option<(char, char)>> = vec![None];
                for trigger_type in "ABCRS".chars() {
                    for group in "0123456789".chars() {
                        triggers.push(Some((trigger_type, group)));
                    }
                }

                for trigger in triggers {
                    let token: String = match trigger {
                        None => format!("{block_type}{behaviour}"),
                        Some((t, g)) => format!("{block_type}{behaviour}{t}{g}"),
                    };

                    let block = make_block(
                        block_type,
                        behaviour,
                        trigger.map(|(t, _)| t),
                        trigger.map(|(_, g)| g),
                        Vec2::ZERO,
                    )
                        .expect("every letter of the format describes a block");

                    assert_eq!(
                        block_token(
                            &block.block_type,
                            &block.behaviour,
                            block.trigger_type.as_ref().zip(block.trigger_group),
                        ),
                        token
                    );

                    checked += 1;
                }
            }
        }

        assert_eq!(checked, 5 * 9 * (1 + 5 * 10), "the whole format has to have been walked");
    }

    /// A trigger the format cannot write is not written at all: half a trigger -
    /// a type character with no group behind it - is a token `make_block` reads
    /// back as a trigger with no group, which is not what was asked for.
    #[test]
    fn a_trigger_group_the_format_has_no_room_for_writes_no_trigger() {
        let token = block_token(
            &BlockType::Simple,
            &BlockBehaviour::SittingDuck,
            Some((&TriggerType::Start, 10)),
        );

        assert_eq!(token, "AA");
    }

    /// The evader speed the format has no room for comes back as the 50.0 every
    /// authored level has always had, rather than as a different block.
    #[test]
    fn an_evader_keeps_its_direction_but_not_its_speed() {
        let token = block_token(&BlockType::Simple, &BlockBehaviour::EvaderU(120.0), None);

        assert_eq!(token, "AG");
        assert_eq!(
            make_block('A', 'G', None, None, Vec2::ZERO).unwrap().behaviour,
            BlockBehaviour::EvaderU(50.0)
        );
    }

    #[test]
    fn set_cell_writes_one_cell_and_leaves_the_rest() {
        let layout = "AA .. AA\n.. AA ..";

        assert_eq!(set_cell(layout, 1, 0, "CB"), "AA CB AA\n.. AA ..");
        assert_eq!(set_cell(layout, 0, 1, "ZIR3"), "AA .. AA\nZIR3 AA ..");
        assert_eq!(set_cell(layout, 2, 1, EMPTY_SLOT), layout, "it was empty already");
        assert_eq!(set_cell(layout, 0, 0, EMPTY_SLOT), ".. .. AA\n.. AA ..");
    }

    /// The shipped levels indent every line after the first, which is padding
    /// rather than a column - what comes back has to be the same grid.
    #[test]
    fn set_cell_keeps_the_shape_of_a_layout_written_by_hand() {
        let layout =
"AA AA AA
 AA AA AA";

        let painted = set_cell(layout, 2, 1, "CA");

        assert_eq!(grid_dimensions(&painted), (3, 2));
        assert_eq!(painted, "AA AA AA\nAA AA CA");
    }

    /// `interpret_grid` takes the column count off the first line, so rows of
    /// different lengths put their blocks in places the grid does not have.
    /// Painting squares the grid up.
    #[test]
    fn set_cell_squares_up_a_ragged_grid() {
        let painted = set_cell("AA AA\nAA AA AA AA\nAA", 0, 0, "ZA");

        assert_eq!(painted, "ZA AA .. ..\nAA AA AA AA\nAA .. .. ..");
        assert_eq!(grid_dimensions(&painted), (4, 3));
    }

    #[test]
    fn set_cell_outside_the_grid_writes_nothing() {
        let layout = "AA .. AA\n.. AA ..";

        assert_eq!(set_cell(layout, 3, 0, "CA"), layout, "past the last column");
        assert_eq!(set_cell(layout, 0, 2, "CA"), layout, "past the last row");
        assert_eq!(set_cell("", 0, 0, "CA"), "", "a layout with no cells in it");
    }

    /// The token grid a `FilledGrid` describes has to spawn exactly the blocks
    /// the `FilledGrid` itself would, or painting one cell of a filled level
    /// would move the other twenty-four.
    #[test]
    fn a_filled_grid_written_as_tokens_holds_the_blocks_it_describes() {
        for (cols, rows) in [(5, 5), (4, 3), (1, 1)] {
            let layout = filled_grid(cols, rows, &BlockType::Concrete, &BlockBehaviour::Spinner);

            assert_eq!(grid_dimensions(&layout), (cols, rows));

            let blocks = interpret_grid(&layout, BLOCK_GAP).unwrap();
            let positions: Vec<Vec2> = blocks.iter().map(|block| block.position).collect();

            assert_eq!(positions, generate_block_grid(rows, cols, BLOCK_GAP), "{cols}x{rows}");

            for block in &blocks {
                assert_eq!(block.block_type, BlockType::Concrete);
                assert_eq!(block.behaviour, BlockBehaviour::Spinner);
            }
        }
    }

    #[test]
    fn an_empty_grid_is_the_size_it_was_asked_for_and_holds_no_blocks() {
        let layout = empty_grid(9, 6);

        assert_eq!(grid_dimensions(&layout), (9, 6));
        assert_eq!(interpret_grid(&layout, BLOCK_GAP).unwrap().len(), 0);
    }

    // --- resizing ----------------------------------------------------------

    /// The grid an author sees, with a block in every corner so a row or column
    /// arriving or leaving cannot be mistaken for the grid staying put.
    fn corners() -> &'static str {
"AA .. BA
 .. CA ..
 DA .. ZA"
    }

    #[test]
    fn a_row_can_be_added_at_the_top_and_at_the_bottom() {
        assert_eq!(
            grow(corners(), Edge::Top),
            ".. .. ..\nAA .. BA\n.. CA ..\nDA .. ZA"
        );

        assert_eq!(
            grow(corners(), Edge::Bottom),
            "AA .. BA\n.. CA ..\nDA .. ZA\n.. .. .."
        );
    }

    #[test]
    fn a_column_can_be_added_at_the_left_and_at_the_right() {
        assert_eq!(
            grow(corners(), Edge::Left),
            ".. AA .. BA\n.. .. CA ..\n.. DA .. ZA"
        );

        assert_eq!(
            grow(corners(), Edge::Right),
            "AA .. BA ..\n.. CA .. ..\nDA .. ZA .."
        );
    }

    #[test]
    fn a_row_can_be_taken_off_the_top_and_off_the_bottom() {
        assert_eq!(shrink(corners(), Edge::Top), ".. CA ..\nDA .. ZA");
        assert_eq!(shrink(corners(), Edge::Bottom), "AA .. BA\n.. CA ..");
    }

    #[test]
    fn a_column_can_be_taken_off_the_left_and_off_the_right() {
        assert_eq!(shrink(corners(), Edge::Left), ".. BA\nCA ..\n.. ZA");
        assert_eq!(shrink(corners(), Edge::Right), "AA ..\n.. CA\nDA ..");
    }

    /// Every cell that is kept keeps what was in it - which is the same thing as
    /// saying a grid grown at an edge and shrunk at it again is the grid it was.
    #[test]
    fn growing_an_edge_and_taking_it_away_again_is_the_grid_it_was() {
        for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
            for layout in sample_layouts() {
                let squared = set_cell(&layout, 0, 0, slot(&layout, 0, 0));

                assert_eq!(
                    shrink(&grow(&layout, edge), edge),
                    squared,
                    "{edge:?} of:\n{layout}"
                );
            }
        }
    }

    /// What a cell holds, so a test can talk about cells rather than substrings.
    fn slot<'a>(layout: &'a str, col: usize, row: usize) -> &'a str {
        slot_grid(layout)[row][col]
    }

    /// The card's "existing blocks keep their position relative to the cells
    /// that are retained", read off the grid: growing at the top moves every
    /// block down a row and leaves its column alone.
    #[test]
    fn the_cells_that_are_kept_hold_what_they_held() {
        let grown = grow(corners(), Edge::Top);

        for row in 0..3 {
            for col in 0..3 {
                assert_eq!(slot(&grown, col, row + 1), slot(corners(), col, row));
            }
        }

        let grown = grow(corners(), Edge::Left);

        for row in 0..3 {
            for col in 0..3 {
                assert_eq!(slot(&grown, col + 1, row), slot(corners(), col, row));
            }
        }

        // The edges that are appended to move nothing at all.
        for edge in [Edge::Bottom, Edge::Right] {
            let grown = grow(corners(), edge);

            for row in 0..3 {
                for col in 0..3 {
                    assert_eq!(slot(&grown, col, row), slot(corners(), col, row), "{edge:?}");
                }
            }
        }
    }

    /// A resize writes the whole grid out, so a hand-written layout with rows of
    /// different lengths comes back square - `interpret_grid` reads the column
    /// count off the first line, and a ragged grid lies to it.
    #[test]
    fn a_resized_grid_is_padded_out_to_one_width() {
        let ragged = "AA AA\nAA AA AA AA\nAA";

        assert_eq!(grow(ragged, Edge::Right), "AA AA .. .. ..\nAA AA AA AA ..\nAA .. .. .. ..");
        assert_eq!(grid_dimensions(&grow(ragged, Edge::Top)), (4, 4));
        assert_eq!(shrink(ragged, Edge::Bottom), "AA AA .. ..\nAA AA AA AA");
    }

    /// A grid always keeps a cell: shrunk away to nothing there would be nothing
    /// left on screen to aim at, and no way back.
    #[test]
    fn a_grid_never_shrinks_away_to_nothing() {
        for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
            assert_eq!(shrink("AA", edge), "AA", "{edge:?}");
            assert_eq!(shrink("", edge), "", "{edge:?}");
        }

        assert_eq!(shrink("AA BA", Edge::Top), "AA BA", "the only row of a wide grid");
        assert_eq!(shrink("AA\nBA", Edge::Left), "AA\nBA", "the only column of a tall grid");
    }

    /// The other end of the same rule: a grid with no cells in it grows into one
    /// cell, whichever side it is grown from.
    #[test]
    fn growing_a_grid_with_no_cells_gives_it_one() {
        for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
            assert_eq!(grow("", edge), EMPTY_SLOT, "{edge:?}");
            assert_eq!(grid_dimensions(&grow("", edge)), (1, 1), "{edge:?}");
        }
    }

    /// What the editor warns with: how much of the level an author is about to
    /// lose. Empty cells are not blocks, and a trigger does not make a slot two.
    #[test]
    fn blocks_on_an_edge_are_counted_before_it_is_taken_away() {
        let layout =
"AA .. AAA1
 .. CA ..
 .. .. ..";

        assert_eq!(blocks_on_edge(layout, Edge::Top), 2);
        assert_eq!(blocks_on_edge(layout, Edge::Bottom), 0);
        assert_eq!(blocks_on_edge(layout, Edge::Left), 1);
        assert_eq!(blocks_on_edge(layout, Edge::Right), 1);

        assert_eq!(blocks_on_edge("", Edge::Top), 0, "a grid with no cells has no edge");
    }

    /// The count has to be the blocks that actually disappear, or the warning is
    /// a number an author cannot check against the screen.
    #[test]
    fn the_count_warned_about_is_the_blocks_that_are_lost() {
        for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
            for layout in sample_layouts() {
                let before = interpret_grid(&layout, BLOCK_GAP).unwrap().len();
                let after = interpret_grid(&shrink(&layout, edge), BLOCK_GAP).unwrap().len();

                if grid_dimensions(&layout) == grid_dimensions(&shrink(&layout, edge)) {
                    continue;
                }

                assert_eq!(
                    before - after,
                    blocks_on_edge(&layout, edge),
                    "{edge:?} of:\n{layout}"
                );
            }
        }
    }

}
