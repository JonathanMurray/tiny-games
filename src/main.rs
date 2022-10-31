extern crate backtrace;
extern crate crossterm;
extern crate rand;
extern crate tui;

mod apps;
mod ui;

use apps::conway::Conway;
use apps::noise::Noise;
use apps::snake::Snake;
use apps::tetris::Tetris;
use apps::App;
use ui::{debug, terminal, window};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let app_name = args.get(1).cloned().unwrap_or_else(|| "snake".to_string());
    let ui_type = args
        .get(2)
        .cloned()
        .unwrap_or_else(|| "terminal".to_string());

    let app: Box<dyn App> = match &app_name[..] {
        "conway" => Box::new(Conway::new(
            (20, 20),
            (10, 0),
            &[
                (2, 3),
                (3, 3),
                (4, 3),
                (5, 3),
                (3, 4),
                (4, 4),
                (5, 4),
                (6, 4),
                (8, 1),
                (9, 1),
                (8, 2),
                (9, 2),
            ],
        )),
        "noise" => Box::new(Noise::new((10, 5))),
        "snake" => Box::new(Snake::new()),
        "tetris" => Box::new(Tetris::new()),
        unknown => panic!("Unknown app: {}", unknown),
    };

    match &ui_type[..] {
        "window" => window::run_main_loop(app),
        "debug" => debug::run_main_loop(app),
        "terminal" => terminal::run_main_loop(app),
        unknown => panic!("Unknown ui: {}", unknown),
    }
}

impl From<bool> for Cell {
    fn from(value: bool) -> Self {
        if value {
            Cell(b'#', (255, 255, 255))
        } else {
            Cell(b' ', (255, 255, 255))
        }
    }
}

pub type Point = (i16, i16);

#[derive(Debug, Copy, Clone)]
pub enum Direction {
    Up,
    Left,
    Down,
    Right,
}

pub type Color = (u8, u8, u8);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Cell(u8, Color);

impl Default for Cell {
    fn default() -> Self {
        Self(b' ', (255, 255, 255))
    }
}

pub trait ReadRenderBuf {
    fn get_cell(&self, point: Point) -> Cell;

    fn dimensions(&self) -> (u8, u8);
}

#[derive(Debug)]
pub struct RenderBuf<T> {
    buf: Vec<T>,
    dimensions: (u8, u8),
}

impl<T: Default + Copy> RenderBuf<T> {
    pub fn new(dimensions: (u8, u8)) -> Self {
        Self {
            buf: vec![Default::default(); dimensions.0 as usize * dimensions.1 as usize],
            dimensions,
        }
    }

    pub fn dimensions(&self) -> (u8, u8) {
        self.dimensions
    }

    pub fn set(&mut self, point: Point, value: T) {
        let i = self.buf_index(point).unwrap();
        self.buf[i] = value;
    }

    pub fn set_by_index(&mut self, index: usize, value: T) {
        self.buf[index] = value;
    }

    pub fn get(&self, point: (i16, i16)) -> Option<T> {
        self.buf_index(point).map(|i| self.buf[i])
    }

    fn buf_index(&self, pos: (i16, i16)) -> Option<usize> {
        if pos.0 >= 0
            && pos.1 >= 0
            && pos.0 < self.dimensions.0 as i16
            && pos.1 < self.dimensions.1 as i16
        {
            Some((pos.1 * self.dimensions.0 as i16 + pos.0) as usize)
        } else {
            None
        }
    }
}

impl<T: Into<Cell> + Copy + Default> ReadRenderBuf for RenderBuf<T> {
    fn get_cell(&self, point: (i16, i16)) -> Cell {
        self.get(point).unwrap().into()
    }

    fn dimensions(&self) -> (u8, u8) {
        self.dimensions
    }
}

pub fn translated(point: Point, direction: Direction) -> Point {
    let (dx, dy) = match direction {
        Direction::Up => (0, -1),
        Direction::Left => (-1, 0),
        Direction::Down => (0, 1),
        Direction::Right => (1, 0),
    };
    (point.0 + dx, point.1 + dy)
}
