use std::cmp;
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

        let mut next = false;
        self.lines.iter().for_each(|line| {
            line.blocks.iter().filter_map(|opt_line_block| {
                match opt_line_block {
                    Some(line_block) => Some(line_block),
                    None => None,
                }
            }).try_for_each(|line_block| {
                self.active_piece.blocks().iter().try_for_each(|active_block| {
                    if active_block.y_pos_bottom() > (line.row * 30) as f32
                        && line_block.col == active_block.col {
                        next = true;
                        return ControlFlow::Break(())
                    }
                    ControlFlow::Continue(())
                })
            });
        });

        if !next {
            self.active_piece.blocks().iter().try_for_each(|active_block| {
                if active_block.y_pos_bottom() > 450 as f32 {
                    next = true;
                    return ControlFlow::Break(())
                }
                ControlFlow::Continue(())
            });
        }

        if next {
            self.next_piece();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::set_canvas(ctx, self.scaler.canvas());
        graphics::clear(ctx, Color::rgba8(102, 101, 112, 222));

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
            line.blocks.iter().filter_map(|opt_block| {
                match opt_block {
                    Some(block) => Some(block),
                    None => None,
                }
            }).for_each(|block| {
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
                self.move_piece(key);
            }
            Event::KeyPressed{ key: Key::Space } => self.active_piece.rotate(),
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

    fn move_piece(&mut self, key: Key) {
        let mut at_boundary = false;
        self.active_piece.blocks().iter().try_for_each(|block| {
            if block.col == 0 && key == Key::Left  || block.col == 9 && key == Key::Right {
                at_boundary = true;
                return ControlFlow::Break(())
            }
            ControlFlow::Continue(())
        });

        if at_boundary {
            return
        }

        self.active_piece.blocks_mut().iter_mut().for_each(|block| {
            match key {
                Key::Left => { block.col -= 1 },
                Key::Right => { block.col += 1 },
                _ => panic!("unexpected key type encountered: {:?} ", key),
            }
        });
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
            for block in lines[row].blocks.iter_mut().filter_map(|opt_block| {
                match opt_block {
                    Some(block) => Some(block),
                    None => None,
                }
            }) {
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

trait Piece {
    fn blocks(&self) -> &Vec<Block>;
    fn blocks_mut(&mut self) -> &mut Vec<Block>;
    fn rotate(&mut self);

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
}

struct Square {
    blocks: Vec<Block>,
}

impl Piece for Square {
    fn blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
    fn blocks_mut(&mut self) -> &mut Vec<Block> { &mut self.blocks }
    fn rotate(&mut self) {}
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

struct Straight {
    blocks: Vec<Block>,
    rotation: u32,
}

impl Piece for Straight {
    fn blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
    fn blocks_mut(&mut self) -> &mut Vec<Block> { &mut self.blocks }
    fn rotate(&mut self) {
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
        self.enforce_boundaries_after_rotation();
        self.rotation = (self.rotation + 90).rem_euclid(360);
    }
}

impl Straight {
    fn new() -> Self {
        Self {
            blocks: vec![
                Block {
                    color: Color::rgba8(43, 215, 54, 204),
                    col: 4,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(245, 40, 145, 204),
                    col: 4,
                    y_pos_top: -60 as f32,
                },
                Block {
                    color: Color::rgba8(8, 210, 234, 204),
                    col: 4,
                    y_pos_top: -90 as f32,
                },
                Block {
                    color: Color::rgba8(222, 18, 55, 204),
                    col: 4,
                    y_pos_top: -120 as f32,
                },
            ],
            rotation: 0,
        }
    }
}

struct T {
    blocks: Vec<Block>,
    rotation: u32,
}

impl Piece for T {
    fn blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
    fn blocks_mut(&mut self) -> &mut Vec<Block> { &mut self.blocks }
    fn rotate(&mut self) {
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
                    color: Color::rgba8(43, 215, 54, 204),
                    col: 3,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(245, 40, 145, 204),
                    col: 4,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(8, 210, 234, 204),
                    col: 5,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(222, 18, 55, 204),
                    col: 4,
                    y_pos_top: -60 as f32,
                },
            ],
            rotation: 0,
        }
    }
}

struct RightL {
    blocks: Vec<Block>,
    rotation: u32,
}

impl Piece for RightL {
    fn blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
    fn blocks_mut(&mut self) -> &mut Vec<Block> { &mut self.blocks }
    fn rotate(&mut self) {
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
                    color: Color::rgba8(43, 215, 54, 204),
                    col: 4,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(245, 40, 145, 204),
                    col: 5,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(8, 210, 234, 204),
                    col: 4,
                    y_pos_top: -60 as f32,
                },
                Block {
                    color: Color::rgba8(222, 18, 55, 204),
                    col: 4,
                    y_pos_top: -90 as f32,
                },
            ],
            rotation: 0,
        }
    }
}

struct LeftL {
    blocks: Vec<Block>,
    rotation: u32,
}

impl Piece for LeftL {
    fn blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
    fn blocks_mut(&mut self) -> &mut Vec<Block> { &mut self.blocks }
    fn rotate(&mut self) {
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
                    color: Color::rgba8(43, 215, 54, 204),
                    col: 4,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(245, 40, 145, 204),
                    col: 5,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(8, 210, 234, 204),
                    col: 5,
                    y_pos_top: -60 as f32,
                },
                Block {
                    color: Color::rgba8(222, 18, 55, 204),
                    col: 5,
                    y_pos_top: -90 as f32,
                },
            ],
            rotation: 0,
        }
    }
}

struct RightSkew {
    blocks: Vec<Block>,
    rotation: u32,
}

impl Piece for RightSkew {
    fn blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
    fn blocks_mut(&mut self) -> &mut Vec<Block> { &mut self.blocks }
    fn rotate(&mut self) {
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
                    color: Color::rgba8(43, 215, 54, 204),
                    col: 3,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(245, 40, 145, 204),
                    col: 4,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(8, 210, 234, 204),
                    col: 4,
                    y_pos_top: -60 as f32,
                },
                Block {
                    color: Color::rgba8(222, 18, 55, 204),
                    col: 5,
                    y_pos_top: -60 as f32,
                },
            ],
            rotation: 0,
        }
    }
}

struct LeftSkew {
    blocks: Vec<Block>,
    rotation: u32,
}

impl Piece for LeftSkew {
    fn blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
    fn blocks_mut(&mut self) -> &mut Vec<Block> { &mut self.blocks }
    fn rotate(&mut self) {
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
                    color: Color::rgba8(43, 215, 54, 204),
                    col: 4,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(245, 40, 145, 204),
                    col: 5,
                    y_pos_top: -30 as f32,
                },
                Block {
                    color: Color::rgba8(8, 210, 234, 204),
                    col: 3,
                    y_pos_top: -60 as f32,
                },
                Block {
                    color: Color::rgba8(222, 18, 55, 204),
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
