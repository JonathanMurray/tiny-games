extern crate backtrace;
extern crate crossterm;
extern crate rand;
extern crate tui;

mod apps;
mod ui;

use crate::apps::RunConfig;
use apps::conway::Conway;
use apps::noise::Noise;
use apps::particles::Particles;
use apps::race::Race;
use apps::snake::Snake;
use apps::tetris::Tetris;
use apps::App;
use ui::debug;
use ui::terminal;
use ui::window;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let app_name = args.get(1).cloned().unwrap_or_else(|| "tetris".to_string());
    let ui_type = args.get(2).cloned().unwrap_or_else(|| "window".to_string());

    let (app, run_config): (Box<dyn App>, RunConfig) = match &app_name[..] {
        "conway" => {
            let (app, run_config) = Conway::new();
            (Box::new(app), run_config)
        }
        "noise" => {
            let (app, run_config) = Noise::new();
            (Box::new(app), run_config)
        }
        "snake" => {
            let (app, run_config) = Snake::new();
            (Box::new(app), run_config)
        }
        "tetris" => {
            let (app, run_config) = Tetris::new();
            (Box::new(app), run_config)
        }
        "particles" => {
            let (app, run_config) = Particles::new();
            (Box::new(app), run_config)
        }
        "race" => {
            let (app, run_config) = Race::new();
            (Box::new(app), run_config)
        }
        unknown => panic!("Unknown app: {}", unknown),
    };

    let frame_rate = run_config.frame_rate;

    match &ui_type[..] {
        "window" => window::run_main_loop(app, frame_rate),
        "terminal" => {
            let cell_width = 3;
            terminal::run_main_loop(app, frame_rate, cell_width)
        }
        "debug" => debug::run_main_loop(app),
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
    pub items: Vec<PanelItem>,
}

impl SidePanel {
    pub fn unwrap_graphics_item_mut(&mut self, index: usize) -> &mut GraphicsBuf {
        match &mut self.items[index] {
            PanelItem::TextItem { .. } => panic!("Expected graphics"),
            PanelItem::GraphicsItem { buf } => buf,
        }
    }

    pub fn unwrap_text_item_mut(&mut self, index: usize) -> &mut String {
        match &mut self.items[index] {
            PanelItem::TextItem { text } => text,
            PanelItem::GraphicsItem { .. } => panic!("Expected text"),
        }
    }
}

#[derive(Debug)]
pub enum PanelItem {
    TextItem { text: String },
    GraphicsItem { buf: GraphicsBuf },
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

    pub fn get_by_index(&self, index: usize) -> Cell {
        self.buf[index]
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
pub enum Cell {
    Blank,
    Colored(Color),
}

impl Default for Cell {
    fn default() -> Self {
        Self::Blank
    }
}

impl Cell {
    pub fn filled() -> Self {
        Self::Colored((255, 255, 255))
    }
}
