use bevy::math::Vec2;
use bevy::prelude::default;
use crate::block::{Block, BlockBehaviour, BlockType};
use crate::block::trigger::{TriggerGroup, TriggerType};

use crate::config::{BLOCK_DEPTH, BLOCK_WIDTH, BLOCK_WIDTH_H};


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
}
