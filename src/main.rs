use std::ops::ControlFlow;
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

type Point2 = Vec2<f32>;

enum Rotation {
    R0,
    R90,
    R180,
    R270,
}

struct GameState {
    block_texture: Texture,
    scaler: ScreenScaler,
    lines: [Line; 15],
    active_piece: Box<dyn Piece>,
    velocity: f32,
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
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

        self.active_piece.blocks().iter().for_each(|block| {
            self.block_texture.draw(
                ctx,
                DrawParams::new()
                    .position(Vec2::new((block.col * 30) as f32, block.y_pos_top))
                    .color(block.color)
                    .scale(Vec2::new(30 as f32 / 16 as f32, 30 as f32 / 16 as f32))
            )
        });
        // println!("{}", self.block_texture.width());

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
            Event::KeyPressed{ key: key @ (Key::Right | Key::Left)}  => {
                self.move_piece(key);
            },
            _ => ()
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

            active_piece: Box::new(Square {
                blocks: vec![
                    Block {
                        color: Color::rgba8(245, 40, 145, 204),
                        col: 4,
                        y_pos_top: 0 as f32,
                    },
                    Block {
                        color: Color::rgba8(245, 40, 145, 204),
                        col: 5,
                        y_pos_top: 0 as f32,
                    },
                    Block {
                        color: Color::rgba8(245, 40, 145, 204),
                        col: 4,
                        y_pos_top: 30 as f32,
                    },
                    Block {
                        color: Color::rgba8(245, 40, 145, 204),
                        col: 5,
                        y_pos_top: 30 as f32,
                    },
                ],
                rotation: Rotation::R0,
            }),
            lines: generate_lines(),
            velocity: 1 as f32,
        })
    }


    fn next_piece(&mut self) {
        self.active_piece.blocks_mut().iter_mut().for_each(|block| {

            let line_num = block.y_pos_top as i32 / 30;

            block.y_pos_top = (self.lines[line_num as usize].row * 30) as f32;

            self.lines[line_num as usize].blocks[block.col as usize] = Some(*block);
        });

        self.active_piece = Box::new(Square {
            blocks: vec![
                Block {
                    color: Color::rgba8(245, 40, 145, 204),
                    col: 4,
                    y_pos_top: 0 as f32,
                },
                Block {
                    color: Color::rgba8(245, 40, 145, 204),
                    col: 5,
                    y_pos_top: 0 as f32,
                },
                Block {
                    color: Color::rgba8(245, 40, 145, 204),
                    col: 4,
                    y_pos_top: 30 as f32,
                },
                Block {
                    color: Color::rgba8(245, 40, 145, 204),
                    col: 5,
                    y_pos_top: 30 as f32,
                },
            ],
            rotation: Rotation::R0,
        });
    }

    fn move_piece(&mut self, key: Key) {
        let mut atBoundary = false;
        self.active_piece.blocks().iter().try_for_each(|block| {
            if block.col == 0 && key == Key::Left  || block.col == 9 && key == Key::Right {
                atBoundary = true;
                return ControlFlow::Break(())
            }
            ControlFlow::Continue(())
        });

        if atBoundary {
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
}

fn generate_lines() -> [Line; 15] {
    let mut lines = [ Line{ row: 0, blocks: [None; 10] }; 15];

    for i in 0..lines.len() {
        lines[i].row = i as u32;
    }

    lines
}

struct Square {
    blocks: Vec<Block>,
    rotation: Rotation,
}

impl Piece for Square {
    fn blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
    fn blocks_mut(&mut self) -> &mut Vec<Block> { &mut self.blocks }
}

trait Piece {
    fn blocks(&self) -> &Vec<Block>;
    fn blocks_mut(&mut self) -> &mut Vec<Block>;
}

#[derive(Clone, Copy)]
struct Block {
    color: Color,
    col: u32,
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


fn main() -> tetra::Result {
    ContextBuilder::new("Tetris", WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
        .quit_on_escape(true)
        .resizable(true)
        .build()?
        .run(GameState::new)
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
