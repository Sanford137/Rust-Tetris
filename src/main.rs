use  std::cmp;
use std::ops::ControlFlow;
use rand::Rng;
// use rand::prelude::*;
use tetra::graphics::scaling::{ScalingMode, ScreenScaler};
use tetra::graphics::{self, Color, DrawParams, Texture};
use tetra::input::{self, Key};
use tetra::math::Vec2;
// use tetra::window;
use tetra::{Context, ContextBuilder, Event, State};
// use image::GenericImageView;
use num::Integer;

mod filter_none;
use filter_none::{filter_none, filter_none_mut};

const WINDOW_WIDTH: i32 = 300;
const WINDOW_HEIGHT: i32 = 450;

#[derive(PartialEq)]
enum PlayMode {
    Running,
    Paused,
}

fn main() -> tetra::Result {
    ContextBuilder::new("Tetris", WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
        .quit_on_escape(true)
        .resizable(true)
        .build()?
        .run(GameState::new)
}

struct GameState {
    block_texture: Texture,
    scaler: ScreenScaler,
    lines: [Line; 15],
    active_piece: Box<dyn Piece>,
    velocity: f32,
    play_mode: PlayMode,
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        if self.play_mode == PlayMode::Paused {
            return Ok(());
        }

        self.active_piece.blocks_mut().iter_mut().for_each(|block| {
            block.y_pos_top += self.velocity
        });

        let mut collision = false;
        let mut reached_floor = false;

        if self.detect_collisions(&self.active_piece) {
            collision = true
        }

        if !collision {
            self.active_piece.blocks().iter().try_for_each(|active_block| {
                if active_block.y_pos_bottom() > 450 as f32 {
                    reached_floor = true;
                    return ControlFlow::Break(())
                }
                ControlFlow::Continue(())
            });
        }

        if collision || reached_floor {
            self.next_piece()
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::set_canvas(ctx, self.scaler.canvas());
        graphics::clear(ctx, Color::rgba8(255, 255, 255, 225));

        for block in self.active_piece.blocks() {
            self.block_texture.draw(
                ctx,
                DrawParams::new()
                    .position(Vec2::new((block.col * 30) as f32, block.y_pos_top))
                    .color(block.color)
                    .scale(Vec2::new(30 as f32 / 16 as f32, 30 as f32 / 16 as f32))
            )
        };

        self.lines.iter().for_each(|line| {
            filter_none(line.blocks.iter()).for_each(|block| {
                self.block_texture.draw(
                    ctx,
                    DrawParams::new()
                        .position(Vec2::new((block.col * 30) as f32, block.y_pos_top))
                        .color(block.color)
                        .scale(Vec2::new(30 as f32 / 16 as f32, 30 as f32 / 16 as f32))
                )
            })
        });

        graphics::reset_canvas(ctx);
        graphics::clear(ctx, Color::BLACK);
        self.scaler.draw(ctx);

        Ok(())
    }

    fn event(&mut self, _: &mut Context, event: Event) -> tetra::Result {
        match event {
            Event::Resized{ width, height } => {
                self.scaler.set_outer_size(width, height);
            },
            Event::KeyPressed { key: Key::RightShift | Key::LeftShift } => self.toggle_pause(),
            _ => (),
        }

        if self.play_mode == PlayMode::Paused {
            return Ok(())
        }

        match event {
            Event::KeyPressed{ key: key @ (Key::Right | Key::Left) }  => {
                self.active_piece.shift(&self.lines, key);
            }
            Event::KeyPressed{ key: Key::Space } => self.active_piece.rotate(&self.lines),
            Event::KeyPressed { key: Key::Down } => self.drop_piece(),
            _ => (),
        }

        Ok(())
    }
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        Ok(GameState {
            block_texture: Texture::new(ctx, "/Users/sanford/rust_tetris/resources/block.png")?,
            scaler: ScreenScaler::with_window_size(
                ctx,
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
                ScalingMode::ShowAllPixelPerfect,
            )?,

            active_piece: Box::new(Square::new()),
            lines: generate_lines(),
            velocity: 1 as f32,
            play_mode: PlayMode::Running,
        })
    }

    fn next_piece(&mut self) {
        self.active_piece.blocks_mut().iter_mut().for_each(|block| {

            let line_num = block.y_pos_top as i32 / 30;

            block.y_pos_top = (self.lines[line_num as usize].row * 30) as f32;

            self.lines[line_num as usize].blocks[block.col as usize] = Some(*block);
        });

        let mut deleted_rows = vec![];
        for (row, line) in self.lines.iter().enumerate() {
            let mut full = true;
            for block in line.blocks {
                if let None = block {
                    full = false;
                    break
                }
            }
            if full {
                deleted_rows.push(row);
            }
        }
        for row in deleted_rows {
            drop_line(&mut self.lines, row as usize)
        }

        let n = rand::thread_rng().gen_range(0..7);
        self.active_piece = match n {
            0 => Box::new(Square::new()),
            1 => Box::new(Straight::new()),
            2 => Box::new(T::new()),
            3 => Box::new(RightL::new()),
            4 => Box::new(LeftL::new()),
            5 => Box::new(RightSkew::new()),
            6 => Box::new(LeftSkew::new()),
            _ => panic!("unexpected number encountered"),
        };

        self.velocity = 1 as f32;
    }

    fn toggle_pause(&mut self) {
        match self.play_mode {
            PlayMode::Paused => self.play_mode = PlayMode::Running,
            PlayMode::Running => self.play_mode = PlayMode::Paused,
        }
    }

    fn drop_piece(&mut self) {
        self.velocity = f32::max(self.velocity * 3 as f32, 10 as f32)
    }

    fn detect_collisions(&self, shadow_piece: &Box<dyn Piece>) -> bool {
        let mut collision = false;
        self.lines.iter().for_each(|line| {
            filter_none(line.blocks.iter()).try_for_each(|line_block| {
                shadow_piece.blocks().iter().try_for_each(|shadow_block| {
                    if shadow_block.y_pos_bottom() > (line.row * 30) as f32
                        && shadow_block.y_pos_top < (line.row * 30) as f32
                        && line_block.col == shadow_block.col {
                        collision = true;
                        return ControlFlow::Break(())
                    }
                    ControlFlow::Continue(())
                })
            });
        });
        collision
    }
}

fn generate_lines() -> [Line; 15] {
    let mut lines = [ Line{ row: 0, blocks: [None; 10] }; 15];

    for i in 0..lines.len() {
        lines[i].row = i as u32;
    }

    lines
}

fn drop_line(lines: &mut [Line; 15], deleted_row: usize) {
    let max_row = lines.len() - 1;
    for row in (1..=max_row).rev()  {
        if row <= deleted_row  {
            lines[row] = lines[row - 1];
            lines[row].row = row as u32;
            for block in filter_none_mut(lines[row].blocks.iter_mut()) {
                block.y_pos_top += 30 as f32;
            }
        }
    }
    lines[0] = Line{ row: 0, blocks: [None; 10] };
}


#[derive(Clone, Copy)]
struct Block {
    color: Color,
    col: i32,
    y_pos_top: f32,
}

impl Block {
    fn y_pos_bottom(&self) -> f32 {
        self.y_pos_top + 30 as f32
    }
}

#[derive(Clone, Copy)]
struct Line {
    row: u32,
    blocks: [Option<Block>; 10],
}

trait CloneBoxPiece {
    fn clone_box(&self) -> Box<dyn Piece>;
}

impl<T> CloneBoxPiece for T
    where
        T: 'static + Piece + Clone,
{
    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }
}

trait Piece : CloneBoxPiece {
    fn blocks(&self) -> &Vec<Block>;
    fn blocks_mut(&mut self) -> &mut Vec<Block>;
    fn do_rotate(&mut self);

    fn enforce_boundaries_after_rotation(&mut self) {
        let mut piece_shift = 0;
        for block in self.blocks() {
            if block.col < 0 {
                let block_shift = -1 * block.col;
                piece_shift = cmp::max(piece_shift, block_shift)
            }
            if block.col > 9 {
                let block_shift = -1 * (block.col - 9);
                piece_shift = cmp::min(piece_shift, block_shift)
            }
        }
        if piece_shift != 0 {
            for block in self.blocks_mut() {
                block.col += piece_shift
            }
        }
    }

    fn shift(&mut self, lines: &[Line; 15], key: Key) -> bool {
        let mut at_boundary = false;
        self.blocks().iter().try_for_each(|block| {
            if block.col == 0 && key == Key::Left  || block.col == 9 && key == Key::Right {
                at_boundary = true;
                return ControlFlow::Break(())
            }
            ControlFlow::Continue(())
        });
        if at_boundary {
            return false
        }

        let mut blocked = false;
        self.blocks().iter().try_for_each(|block| {
            let cur_row = block.y_pos_top as usize / 30;
            let next_row = cmp::min(cur_row + 1, lines.len() - 1);
            if key == Key::Left {
                if let Some(_) = lines[cur_row].blocks[block.col as usize - 1] {
                    blocked = true;
                }
                if let Some(_) = lines[next_row].blocks[block.col as usize - 1] {
                    blocked = true;
                }
                return ControlFlow::Break(())
            }
            if key == Key::Right {
                if let Some(_) = lines[cur_row].blocks[block.col as usize + 1] {
                    blocked = true;
                }
                if let Some(_) = lines[next_row].blocks[block.col as usize + 1] {
                    blocked = true;
                }
                return ControlFlow::Break(())
            }
            ControlFlow::Continue(())
        });
        if blocked {
            return false
        }

        self.blocks_mut().iter_mut().for_each(|block| {
            match key {
                Key::Left => { block.col -= 1 },
                Key::Right => { block.col += 1 },
                _ => panic!("unexpected key type encountered: {:?} ", key),
            }
        });

        true
    }

    // returns (right_shift, left_shift)
    fn calculate_boundary_shifts(&self) -> (i32, i32) {
        let mut right_shift = 0;
        let mut left_shift = 0;
        for block in self.blocks() {
            if block.col < 0 {
                let block_shift = -1 * block.col;
                right_shift = cmp::max(right_shift, block_shift)
            }
            if block.col > 9 {
                let block_shift = block.col - 9;
                left_shift = cmp::max(left_shift, block_shift)
            }
        }
        (right_shift, left_shift)
    }

    // returns (rightmost_col, leftmost_col)
    fn calculate_edge_cols(&self) -> (i32, i32) {
        let mut rightmost_col = 0;
        let mut leftmost_col = 9;
        for block in self.blocks() {
            rightmost_col = cmp::max(rightmost_col, block.col);
            leftmost_col = cmp::min(leftmost_col, block.col)
        }
        (rightmost_col, leftmost_col)
    }

    // return (right_shift, left_shift)
    fn calculate_collision_shifts(&self, lines: &[Line; 15]) -> (i32, i32) {
        let (rightmost_col, leftmost_col) = self.calculate_edge_cols();
        let width = rightmost_col - leftmost_col + 1;
        let even = Integer::is_even(&width);
        
        let mut center = if !even {
            leftmost_col + width / 2
        } else {
            leftmost_col + width / 2 - 1
        };
        
        let mut right_shift = 0;
        let mut left_shift = 0;
        for line in lines {
            let mut right_shift_row = 0;
            let mut left_shift_row = 0;
            
            for line_block in filter_none(line.blocks.iter()) {
                for block in self.blocks() {
                    if block.y_pos_bottom() > (line.row * 30) as f32
                        && block.y_pos_top < (line.row * 30) as f32
                        && line_block.col == block.col {
                        
                        if even {
                            if block.col <= center {
                                right_shift_row += 1;
                            } else {
                                left_shift_row += 1;
                            }
                        } else {
                            if block.col < center {
                                right_shift_row = 1;
                            } else if block.col > center {
                                left_shift_row = 1;
                            } else {
                                panic!("collision at center of odd-width piece")
                            }
                        }
                    }
                }
            }
            
            right_shift = cmp::max(right_shift, right_shift_row);
            left_shift = cmp::max(left_shift, left_shift_row);
        }

        (right_shift, left_shift)
    }

    fn rotate(&mut self, lines: &[Line; 15])
    {
        let mut shadow = self.clone_box();

        shadow.do_rotate();

        let (boundary_right_shift, boundary_left_shift) = shadow.calculate_boundary_shifts();
        let (collision_right_shift, collision_left_shift) = shadow.calculate_collision_shifts(lines);

        let shift_right = cmp::max(boundary_right_shift, collision_right_shift);
        let shift_left = cmp::max(boundary_left_shift, collision_left_shift);

        if shift_right != 0 && shift_left != 0 {
            return
        }

        let mut shadow_shift_successful = true;
        let mut shadow_shift_right = shift_right;
        let mut shadow_shift_left = shift_left;
        while shadow_shift_right != 0 || shadow_shift_left != 0 {
            if shadow_shift_right != 0 {
                if shadow.shift(lines, Key::Right) {
                    shadow_shift_right -= 1;
                } else {
                    shadow_shift_successful = false;
                    break
                }
            }
            if shadow_shift_left != 0 {
                if shadow.shift(lines, Key::Left) {
                    shadow_shift_left -= 1;
                } else {
                    shadow_shift_successful = false;
                    break
                }
            }
        }

        if !shadow_shift_successful {
            return
        }

        self.do_rotate();

        let mut real_shift_right = shift_right;
        let mut real_shift_left = shift_left;
        while real_shift_right != 0 || real_shift_left != 0 {
            if real_shift_right != 0 {
                self.shift(lines, Key::Right);
                real_shift_right -= 1
            }
            if real_shift_left != 0 {
                self.shift(lines, Key::Left);
                real_shift_left -= 1
            }
        }
    }
}

#[derive(Clone)]
struct Square {
    blocks: Vec<Block>,
}

impl Piece for Square {
    fn blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
    fn blocks_mut(&mut self) -> &mut Vec<Block> { &mut self.blocks }
    fn do_rotate(&mut self) {}
}

impl Square {
    fn new() -> Self {
        Self {
            blocks: vec![
                Block {
                    color: Color::rgba8(245, 40, 145, 204),
                    col: 4,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(245, 40, 145, 204),
                    col: 5,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(245, 40, 145, 204),
                    col: 4,
                    y_pos_top: -60 as f32,
                },
                Block {
                    color: Color::rgba8(245, 40, 145, 204),
                    col: 5,
                    y_pos_top: -60 as f32,
                },
            ],
        }
    }
}

#[derive(Clone)]
struct Straight {
    blocks: Vec<Block>,
    rotation: u32,
}

impl Piece for Straight {
    fn blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
    fn blocks_mut(&mut self) -> &mut Vec<Block> { &mut self.blocks }

    fn do_rotate(&mut self) {
        match self.rotation {
            0 => {
                self.blocks_mut()[0].col += -1;
                self.blocks_mut()[0].y_pos_top += -30 as f32;

                self.blocks_mut()[2].col += 1;
                self.blocks_mut()[2].y_pos_top += 30 as f32;

                self.blocks_mut()[3].col += 2;
                self.blocks_mut()[3].y_pos_top += 60 as f32;
            },
            90 => {
                self.blocks_mut()[0].col += 1;
                self.blocks_mut()[0].y_pos_top += -30 as f32;

                self.blocks_mut()[2].col += -1;
                self.blocks_mut()[2].y_pos_top += 30 as f32;

                self.blocks_mut()[3].col += -2;
                self.blocks_mut()[3].y_pos_top += 60 as f32;
            },
            180 => {
                self.blocks_mut()[0].col += 1;
                self.blocks_mut()[0].y_pos_top += 30 as f32;

                self.blocks_mut()[2].col += -1;
                self.blocks_mut()[2].y_pos_top += -30 as f32;

                self.blocks_mut()[3].col += -2;
                self.blocks_mut()[3].y_pos_top += -60 as f32;
            },
            270 => {
                self.blocks_mut()[0].col += -1;
                self.blocks_mut()[0].y_pos_top += 30 as f32;

                self.blocks_mut()[2].col += 1;
                self.blocks_mut()[2].y_pos_top += -30 as f32;

                self.blocks_mut()[3].col += 2;
                self.blocks_mut()[3].y_pos_top += -60 as f32;
            },
            _ => panic!("received unexpected rotation value: {}", self.rotation),
        }

        self.rotation = (self.rotation + 90).rem_euclid(360);
    }
}

impl Straight {
    fn new() -> Self {
        Self {
            blocks: vec![
                Block {
                    color: Color::rgba8(61, 139, 232, 117),
                    col: 4,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(61, 139, 232, 117),
                    col: 4,
                    y_pos_top: -60 as f32,
                },
                Block {
                    color: Color::rgba8(61, 139, 232, 117),
                    col: 4,
                    y_pos_top: -90 as f32,
                },
                Block {
                    color: Color::rgba8(61, 139, 232, 117),
                    col: 4,
                    y_pos_top: -120 as f32,
                },
            ],
            rotation: 0,
        }
    }
}

#[derive(Clone)]
struct T {
    blocks: Vec<Block>,
    rotation: u32,
}

impl Piece for T {
    fn blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
    fn blocks_mut(&mut self) -> &mut Vec<Block> { &mut self.blocks }

    fn do_rotate(&mut self) {
        match self.rotation {
            0 => {
                self.blocks_mut()[0].col += 1;
                self.blocks_mut()[0].y_pos_top += -30 as f32;

                self.blocks_mut()[2].col += -1;
                self.blocks_mut()[2].y_pos_top += 30 as f32;

                self.blocks_mut()[3].col += 1;
                self.blocks_mut()[3].y_pos_top += 30 as f32;
            },
            90 => {
                self.blocks_mut()[0].col += 1;
                self.blocks_mut()[0].y_pos_top += 30 as f32;

                self.blocks_mut()[2].col += -1;
                self.blocks_mut()[2].y_pos_top += -30 as f32;

                self.blocks_mut()[3].col += -1;
                self.blocks_mut()[3].y_pos_top += 30 as f32;
            },
            180 => {
                self.blocks_mut()[0].col += -1;
                self.blocks_mut()[0].y_pos_top += 30 as f32;

                self.blocks_mut()[2].col += 1;
                self.blocks_mut()[2].y_pos_top += -30 as f32;

                self.blocks_mut()[3].col += -1;
                self.blocks_mut()[3].y_pos_top += -30 as f32;
            },
            270 => {
                self.blocks_mut()[0].col += -1;
                self.blocks_mut()[0].y_pos_top += -30 as f32;

                self.blocks_mut()[2].col += 1;
                self.blocks_mut()[2].y_pos_top += 30 as f32;

                self.blocks_mut()[3].col += 1;
                self.blocks_mut()[3].y_pos_top += -30 as f32;
            },
            _ => panic!("received unexpected rotation value: {}", self.rotation),
        }
        self.enforce_boundaries_after_rotation();
        self.rotation = (self.rotation + 90).rem_euclid(360);
    }
}

impl T {
    fn new() -> Self {
        Self {
            blocks: vec![
                Block {
                    color: Color::rgba8(47, 94, 68, 196),
                    col: 3,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(47, 94, 68, 196),
                    col: 4,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(47, 94, 68, 196),
                    col: 5,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(47, 94, 68, 196),
                    col: 4,
                    y_pos_top: -60 as f32,
                },
            ],
            rotation: 0,
        }
    }
}

#[derive(Clone)]
struct RightL {
    blocks: Vec<Block>,
    rotation: u32,
}

impl Piece for RightL {
    fn blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
    fn blocks_mut(&mut self) -> &mut Vec<Block> { &mut self.blocks }

    fn do_rotate(&mut self) {
        match self.rotation {
            0 => {
                self.blocks_mut()[1].col += -1;
                self.blocks_mut()[1].y_pos_top += 30 as f32;

                self.blocks_mut()[2].col += 1;
                self.blocks_mut()[2].y_pos_top += 30 as f32;

                self.blocks_mut()[3].col += 2;
                self.blocks_mut()[3].y_pos_top += 60 as f32;
            },
            90 => {
                self.blocks_mut()[1].col += -1;
                self.blocks_mut()[1].y_pos_top += -30 as f32;

                self.blocks_mut()[2].col += -1;
                self.blocks_mut()[2].y_pos_top += 30 as f32;

                self.blocks_mut()[3].col += -2;
                self.blocks_mut()[3].y_pos_top += 60 as f32;
            },
            180 => {
                self.blocks_mut()[1].col += 1;
                self.blocks_mut()[1].y_pos_top += -30 as f32;

                self.blocks_mut()[2].col += -1;
                self.blocks_mut()[2].y_pos_top += -30 as f32;

                self.blocks_mut()[3].col += -2;
                self.blocks_mut()[3].y_pos_top += -60 as f32;
            },
            270 => {
                self.blocks_mut()[1].col += 1;
                self.blocks_mut()[1].y_pos_top += 30 as f32;

                self.blocks_mut()[2].col += 1;
                self.blocks_mut()[2].y_pos_top += -30 as f32;

                self.blocks_mut()[3].col += 2;
                self.blocks_mut()[3].y_pos_top += -60 as f32;
            },
            _ => panic!("received unexpected rotation value: {}", self.rotation),
        }
        self.enforce_boundaries_after_rotation();
        self.rotation = (self.rotation + 90).rem_euclid(360);
    }
}

impl RightL {
    fn new() -> Self {
        Self {
            blocks: vec![
                Block {
                    color: Color::rgba8(249, 134, 36, 224),
                    col: 4,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(249, 134, 36, 224),
                    col: 5,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(249, 134, 36, 224),
                    col: 4,
                    y_pos_top: -60 as f32,
                },
                Block {
                    color: Color::rgba8(249, 134, 36, 224),
                    col: 4,
                    y_pos_top: -90 as f32,
                },
            ],
            rotation: 0,
        }
    }
}

#[derive(Clone)]
struct LeftL {
    blocks: Vec<Block>,
    rotation: u32,
}

impl Piece for LeftL {
    fn blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
    fn blocks_mut(&mut self) -> &mut Vec<Block> { &mut self.blocks }

    fn do_rotate(&mut self) {
        match self.rotation {
            0 => {
                self.blocks_mut()[0].col += 1;
                self.blocks_mut()[0].y_pos_top += -30 as f32;

                self.blocks_mut()[2].col += 1;
                self.blocks_mut()[2].y_pos_top += 30 as f32;

                self.blocks_mut()[3].col += 2;
                self.blocks_mut()[3].y_pos_top += 60 as f32;
            },
            90 => {
                self.blocks_mut()[0].col += 1;
                self.blocks_mut()[0].y_pos_top += 30 as f32;

                self.blocks_mut()[2].col += -1;
                self.blocks_mut()[2].y_pos_top += 30 as f32;

                self.blocks_mut()[3].col += -2;
                self.blocks_mut()[3].y_pos_top += 60 as f32;
            },
            180 => {
                self.blocks_mut()[0].col += -1;
                self.blocks_mut()[0].y_pos_top += 30 as f32;

                self.blocks_mut()[2].col += -1;
                self.blocks_mut()[2].y_pos_top += -30 as f32;

                self.blocks_mut()[3].col += -2;
                self.blocks_mut()[3].y_pos_top += -60 as f32;
            },
            270 => {
                self.blocks_mut()[0].col += -1;
                self.blocks_mut()[0].y_pos_top += -30 as f32;

                self.blocks_mut()[2].col += 1;
                self.blocks_mut()[2].y_pos_top += -30 as f32;

                self.blocks_mut()[3].col += 2;
                self.blocks_mut()[3].y_pos_top += -60 as f32;
            },
            _ => panic!("received unexpected rotation value: {}", self.rotation),
        }
        self.enforce_boundaries_after_rotation();
        self.rotation = (self.rotation + 90).rem_euclid(360);
    }
}

impl LeftL {
    fn new() -> Self {
        Self {
            blocks: vec![
                Block {
                    color: Color::rgba8(176, 99, 246, 199),
                    col: 4,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(176, 99, 246, 199),
                    col: 5,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(176, 99, 246, 199),
                    col: 5,
                    y_pos_top: -60 as f32,
                },
                Block {
                    color: Color::rgba8(176, 99, 246, 199),
                    col: 5,
                    y_pos_top: -90 as f32,
                },
            ],
            rotation: 0,
        }
    }
}

#[derive(Clone)]
struct RightSkew {
    blocks: Vec<Block>,
    rotation: u32,
}

impl Piece for RightSkew {
    fn blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
    fn blocks_mut(&mut self) -> &mut Vec<Block> { &mut self.blocks }

    fn do_rotate(&mut self) {
        match self.rotation {
            0 => {
                self.blocks_mut()[0].col += 1;
                self.blocks_mut()[0].y_pos_top += -30 as f32;

                self.blocks_mut()[2].col += 1;
                self.blocks_mut()[2].y_pos_top += 30 as f32;

                self.blocks_mut()[3].col += 0;
                self.blocks_mut()[3].y_pos_top += 60 as f32;
            },
            90 => {
                self.blocks_mut()[0].col += -1;
                self.blocks_mut()[0].y_pos_top += 30 as f32;

                self.blocks_mut()[2].col += -1;
                self.blocks_mut()[2].y_pos_top += -30 as f32;

                self.blocks_mut()[3].col += 0;
                self.blocks_mut()[3].y_pos_top += -60 as f32;
            },
            _ => panic!("received unexpected rotation value: {}", self.rotation),
        }
        self.enforce_boundaries_after_rotation();
        self.rotation = (self.rotation + 90).rem_euclid(180);
    }
}

impl RightSkew {
    fn new() -> Self {
        Self {
            blocks: vec![
                Block {
                    color: Color::rgba8(244, 127, 241, 166),
                    col: 3,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(244, 127, 241, 166),
                    col: 4,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(244, 127, 241, 166),
                    col: 4,
                    y_pos_top: -60 as f32,
                },
                Block {
                    color: Color::rgba8(244, 127, 241, 166),
                    col: 5,
                    y_pos_top: -60 as f32,
                },
            ],
            rotation: 0,
        }
    }
}

#[derive(Clone)]
struct LeftSkew {
    blocks: Vec<Block>,
    rotation: u32,
}

impl Piece for LeftSkew {
    fn blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
    fn blocks_mut(&mut self) -> &mut Vec<Block> { &mut self.blocks }

    fn do_rotate(&mut self) {
        match self.rotation {
            0 => {
                self.blocks_mut()[1].col += -1;
                self.blocks_mut()[1].y_pos_top += 30 as f32;

                self.blocks_mut()[2].col += 2;
                self.blocks_mut()[2].y_pos_top += 0 as f32;

                self.blocks_mut()[3].col += 1;
                self.blocks_mut()[3].y_pos_top += 30 as f32;
            },
            90 => {
                self.blocks_mut()[1].col += 1;
                self.blocks_mut()[1].y_pos_top += -30 as f32;

                self.blocks_mut()[2].col += -2;
                self.blocks_mut()[2].y_pos_top += 0 as f32;

                self.blocks_mut()[3].col += -1;
                self.blocks_mut()[3].y_pos_top += -30 as f32;
            },
            _ => panic!("received unexpected rotation value: {}", self.rotation),
        }
        self.enforce_boundaries_after_rotation();
        self.rotation = (self.rotation + 90).rem_euclid(180);
    }
}

impl LeftSkew {
    fn new() -> Self {
        Self {
            blocks: vec![
                Block {
                    color: Color::rgba8(245, 96, 127, 225),
                    col: 4,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(245, 96, 127, 225),
                    col: 5,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(245, 96, 127, 225),
                    col: 3,
                    y_pos_top: -60 as f32,
                },
                Block {
                    color: Color::rgba8(245, 96, 127, 225),
                    col: 4,
                    y_pos_top: -60 as f32,
                },
            ],
            rotation: 0,
        }
    }
}








// impl Entity {
//     fn width(&self) -> f32 {
//         self.texture.width() as f32
//     }
//
//     fn height(&self) -> f32 {
//         self.texture.height() as f32
//     }
//
//     fn bounds(&self) -> Rectangle {
//         Rectangle::new(
//             self.position.x,
//             self.position.y,
//             self.width(),
//             self.height(),
//         )
//     }
//
//     fn center(&self) -> Point2 {
//         Vec2::new(
//             self.position.x + (self.width() / 2.0),
//             self.position.y - (self.height() / 2.0),
//         )
//     }
//
//     fn draw(&mut self, ctx: &mut Context) {
//         let screen_width = window::get_width(ctx) as f32;
//         let screen_height = window::get_height(ctx) as f32;
//         let screen_coords = world_to_screen_coords(self.position, screen_width, screen_height);
//         self.texture.draw(ctx, screen_coords);
//     }
// }
