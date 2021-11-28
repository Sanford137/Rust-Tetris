// use rand::prelude::*;
use tetra::graphics::{self, Color, Rectangle, Texture};
// use tetra::input::{self, Key};
use tetra::math::Vec2;
use tetra::window;
use tetra::{Context, ContextBuilder, State};
// use image::GenericImageView;

const WINDOW_WIDTH: f32 = 300.0;
const WINDOW_HEIGHT: f32 = 550.0;

type Point2 = Vec2<f32>;

fn main() -> tetra::Result {
    ContextBuilder::new("Tetris", WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
        .quit_on_escape(true)
        .resizable(true)
        .build()?
        .run(GameState::new)
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgba8(102, 101, 112, 222));

        for block in self.blocks.iter_mut() {
            block.draw(ctx);
        }

        Ok(())
    }
}

struct GameState {
    blocks: Vec<Entity>,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let texture = Texture::new(ctx, "/Users/sanford/rust_tetris/resources/block.png")?;

        // let texture = Texture::from_rgba(ctx, 100, 30, &[185u8, 50u8, 120u8])?;

        Ok(GameState {
            blocks: vec![
                Entity{
                    texture,
                    position: Point2::new(-50f32, 100f32),
                    velocity: Vec2::zero(),
                    rotation: 0f32,
                }
            ]
        })
    }
}

struct Entity {
    texture: Texture,
    position: Point2,
    velocity: Vec2<f32>,
    rotation: f32,
}

impl Entity {
    fn width(&self) -> f32 {
        self.texture.width() as f32
    }

    fn height(&self) -> f32 {
        self.texture.height() as f32
    }

    fn bounds(&self) -> Rectangle {
        Rectangle::new(
            self.position.x,
            self.position.y ,
            self.width(),
            self.height(),
        )
    }

    fn center(&self) -> Point2 {
        Vec2::new(
            self.position.x + (self.width() / 2.0),
            self.position.y - (self.height() / 2.0),
        )
    }

    fn draw(&mut self, ctx: &mut Context) {
        let screen_width = window::get_width(ctx) as f32;
        let screen_height = window::get_height(ctx) as f32;
        let screen_coords = world_to_screen_coords(self.position, screen_width, screen_height);
        self.texture.draw(ctx, screen_coords);
    }
}

fn world_to_screen_coords(point: Point2, screen_width: f32, screen_height: f32) -> Point2 {
    let x = point.x + screen_width / 2.0;
    let y = screen_height - point.y;
    Point2::new(x, y)
}

