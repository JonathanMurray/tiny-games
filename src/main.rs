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
use ui::debug;
use ui::terminal;
use ui::window;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let app_name = args.get(1).cloned().unwrap_or_else(|| "snake".to_string());
    let ui_type = args
        .get(2)
        .cloned()
        .unwrap_or_else(|| "terminal".to_string());

    let frame_rate;

    let app: Box<dyn App> = match &app_name[..] {
        "conway" => {
            let (app, app_init) = Conway::new(
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
            );
            frame_rate = app_init.frame_rate;
            Box::new(app)
        }

        "noise" => {
            let (app, app_init) = Noise::new((10, 5));
            frame_rate = app_init.frame_rate;
            Box::new(app)
        }
        "snake" => {
            let (app, app_init) = Snake::new();
            frame_rate = app_init.frame_rate;
            Box::new(app)
        }
        "tetris" => {
            let (app, app_init) = Tetris::new();
            frame_rate = app_init.frame_rate;
            Box::new(app)
        }
        unknown => panic!("Unknown app: {}", unknown),
    };

    match &ui_type[..] {
        "window" => window::run_main_loop(app, frame_rate),
        "debug" => debug::run_main_loop(app),
        "terminal" => terminal::run_main_loop(app, frame_rate),
        unknown => panic!("Unknown ui: {}", unknown),
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

pub fn translated(point: Point, direction: Direction) -> Point {
    let (dx, dy) = match direction {
        Direction::Up => (0, -1),
        Direction::Left => (-1, 0),
        Direction::Down => (0, 1),
        Direction::Right => (1, 0),
    };
    (point.0 + dx, point.1 + dy)
}

pub struct Graphics {
    pub title: String,
    side_panel: Option<SidePanel>,
    pub buf: GraphicsBuf,
}

impl Graphics {
    pub fn new(title: String, side_panel: Option<SidePanel>, graphics: GraphicsBuf) -> Self {
        Self {
            title,
            side_panel,
            buf: graphics,
        }
    }

    pub fn side_panel(&self) -> Option<&SidePanel> {
        self.side_panel.as_ref()
    }
}

#[derive(Debug)]
pub struct SidePanel {
    pub help_text: Option<String>,
}

#[derive(Debug)]
pub struct GraphicsBuf {
    buf: Vec<Cell>,
    dimensions: (u8, u8),
}

impl GraphicsBuf {
    pub fn new(dimensions: (u8, u8)) -> Self {
        Self {
            buf: vec![Default::default(); dimensions.0 as usize * dimensions.1 as usize],
            dimensions,
        }
    }

    pub fn dimensions(&self) -> (u8, u8) {
        self.dimensions
    }

    pub fn set(&mut self, point: Point, value: Cell) {
        let i = self.buf_index(point).unwrap();
        self.buf[i] = value;
    }

    pub fn set_by_index(&mut self, index: usize, value: Cell) {
        self.buf[index] = value;
    }

    pub fn get(&self, point: (i16, i16)) -> Option<Cell> {
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Cell(u8, Color);

impl Default for Cell {
    fn default() -> Self {
        Self(b' ', (255, 255, 255))
    }
}

impl Cell {
    pub fn on() -> Self {
        Self(b'#', (255, 255, 255))
    }

    pub fn off() -> Self {
        Self(b' ', (0, 0, 0))
    }
}