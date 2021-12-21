// use rand::prelude::*;
use tetra::graphics::scaling::{ScalingMode, ScreenScaler};
use tetra::graphics::{self, Color, DrawParams, Rectangle, Texture};
// use tetra::input::{self, Key};
use tetra::math::Vec2;
use tetra::window;
use tetra::{Context, ContextBuilder, Event, State};
// use image::GenericImageView;

const WINDOW_WIDTH: i32 = 300;
const WINDOW_HEIGHT: i32 = 550;

type Point2 = Vec2<f32>;

enum Rotation {
    R0,
    R90,
    R180,
    R270,
}

fn main() -> tetra::Result {
    ContextBuilder::new("Tetris", WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
        .quit_on_escape(true)
        .resizable(true)
        .build()?
        .run(GameState::new)
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.active_piece.blocks().iter_mut().for_each(|block| {
            block.position.y += self.velocity
        });

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::set_canvas(ctx, self.scaler.canvas());
        graphics::clear(ctx, Color::rgba8(102, 101, 112, 222));

        self.active_piece.blocks().iter().for_each(|block| {
            self.block_texture.draw(
                ctx,
                DrawParams::new()
                    .position(block.position)
                    .color(block.color)
                    .scale(Vec2::new(30 as f32 / 16 as f32, 30 as f32 / 16 as f32))
            )
        });
        // println!("{}", self.block_texture.width());

        self.lines.iter().for_each(|line| {
            line.blocks.iter().for_each(|block| {
                self.block_texture.draw(
                    ctx,
                    DrawParams::new()
                        .position(block.position)
                        .color(block.color),
                )
            })
        });
        graphics::reset_canvas(ctx);
        graphics::clear(ctx, Color::BLACK);
        self.scaler.draw(ctx);

        Ok(())
    }

    fn event(&mut self, _: &mut Context, event: Event) -> tetra::Result {
        if let Event::Resized { width, height } = event {
            self.scaler.set_outer_size(width, height);
        }

        Ok(())
    }
}

struct GameState {
    block_texture: Texture,
    scaler: ScreenScaler,
    lines: Vec<Line>,
    active_piece: Box<dyn Piece>,
    velocity: f32,
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
                        position: Point2::new(120f32, 0f32),
                    },
                    Block {
                        color: Color::rgba8(245, 40, 145, 204),
                        position: Point2::new(150f32, 0f32),
                    },
                    Block {
                        color: Color::rgba8(245, 40, 145, 204),
                        position: Point2::new(120f32, 30f32),
                    },
                    Block {
                        color: Color::rgba8(245, 40, 145, 204),
                        position: Point2::new(150f32, 30f32),
                    },
                ],
                rotation: Rotation::R0,
            }),
            lines: vec![],
            velocity: 1 as f32,
        })
    }
}

struct Square {
    blocks: Vec<Block>,
    rotation: Rotation,
}

impl Piece for Square {
    fn blocks(&mut self) -> &mut Vec<Block> {
        &mut self.blocks
    }
}

trait Piece {
    fn blocks(&mut self) -> &mut Vec<Block>;
}

struct Block {
    color: Color,
    position: Point2,
}

struct Line {
    blocks: [Block; 10],
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
