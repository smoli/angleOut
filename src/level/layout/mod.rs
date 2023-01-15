use bevy::math::Vec2;
use bevy::utils::default;
use crate::block::{Block, BlockBehaviour, BlockType};
use crate::config::{BLOCK_DEPTH, BLOCK_WIDTH, BLOCK_WIDTH_H};


pub fn generate_block_grid(
    rows: usize,
    cols: usize,
    gap: f32,
)   -> Vec<Vec2>

{
    let mut y = -30.0 - 4.0 * (BLOCK_DEPTH + gap);
    let y_step = BLOCK_DEPTH + gap;

    let x_step = BLOCK_WIDTH + gap;
    let cols_h = (cols / 2) as f32;

    let mut res = vec![];

    for _ in 0..rows {
        let mut x = 0.0;
        if cols % 2 == 1 {
            x -= cols_h * x_step;
        } else {
            x -= cols_h * x_step - gap / 2.0 - BLOCK_WIDTH_H;
        }

        for _ in 0..cols {
            res.push(Vec2::new(x, y));
            x += x_step;
        }

        y += y_step;
    };

    res
}


pub fn make_block(b_type: char, b_beh: char, pos: Vec2) -> Option<Block> {
    let t = match b_type  {
        'A' => BlockType::Simple,
        'B' => BlockType::Hardling,
        'C' => BlockType::Concrete,

        '.' => return None,

        _ => BlockType::Simple
    };

    let b = match b_beh {
        'A' => BlockBehaviour::SittingDuck,
        'B' => BlockBehaviour::Spinner,
        'C' => BlockBehaviour::Vanisher,
        'D' => BlockBehaviour::Repuslor,

        '.' => return None,

        _ => BlockBehaviour::SittingDuck
    };

    Some(Block {
        behaviour: b,
        block_type: t,
        position: pos,
        ..default()
    })
}


pub fn interpret_grid(layout: &String, cols: usize, gap: f32) -> Option<Vec<Block>> {

    let mut res = vec![];

    let mut lines:Vec<&str> = layout.split("\n").collect();
    let mut line_count = lines.len();

    let x_step = BLOCK_WIDTH + gap;
    let cols_h = (cols / 2) as f32;

    let mut y = -30.0 - 4.0 * (BLOCK_DEPTH + gap);
    let y_step = BLOCK_DEPTH + gap;

    for line in lines {
        line_count += 1;

        let mut x = 0.0;
        if cols % 2 == 1 {
            x -= cols_h * x_step;
        } else {
            x -= cols_h * x_step - gap / 2.0 - BLOCK_WIDTH_H;
        }


        let slots = line.split(" ");

        for slot in slots {
            if slot.len() != 2 {
                continue;
            }

            let pos_x = x;
            x += x_step;

            let b_type = slot.chars().nth(0).unwrap();
            let b_beh = slot.chars().nth(1).unwrap();


            match make_block(b_type, b_beh, Vec2::new(pos_x, y)) {
                None => {}
                Some(block) => res.push(block)
            }

        }
        y += y_step;
    }


    Some(res)
}


// .. = Empty slot
// <Type><Behaviour>
// Ignore spaces

// Types
// A = Simple
// B = Hardling
// C = Conctete

// Behaviour
// A = SittingDuck
// B = Spinner
// At max 10 wide


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn works() {
        let a_level =
"AA .. .. .. .. .. .. .. .. AA
 .. .. .. .. .. .. .. .. .. ..
 .. .. .. .. AB .. .. .. .. ..
 .. .. .. .. .. .. .. .. .. ..
 AA .. .. .. .. .. .. .. .. AA".to_string();

        if let Some(res) = interpret_grid(&a_level, 10, 3.0) {
            assert_eq!(res.len(), 5);

            for b in res {
                println!("{:?}", b);
            }
        } else {
            assert!(false)
        }



    }


}