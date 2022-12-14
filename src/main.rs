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

use clap::Parser;

/// Tiny games, played in a window or right in your terminal!
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// What game to run
    #[arg(value_enum, default_value = "tetris")]
    app: AppName,

    /// How to run the game
    #[arg(short, long, value_enum, default_value = "terminal")]
    runtime: Runtime,

    /// In the terminal, how many characters wide should each game cell be
    #[arg(short, long, default_value = "3")]
    cell_width: u16,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum AppName {
    Tetris,
    Snake,
    Conway,
    Noise,
    Race,
    Particles,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum Runtime {
    Window,
    Terminal,
    Debug,
}

fn main() {
    let args = Args::parse();

    let app_name = args.app;
    let runtime = args.runtime;

    let (app, run_config): (Box<dyn App>, RunConfig) = match app_name {
        AppName::Conway => {
            let (app, run_config) = Conway::new();
            (Box::new(app), run_config)
        }
        AppName::Noise => {
            let (app, run_config) = Noise::new();
            (Box::new(app), run_config)
        }
        AppName::Snake => {
            let (app, run_config) = Snake::new();
            (Box::new(app), run_config)
        }
        AppName::Tetris => {
            let (app, run_config) = Tetris::new();
            (Box::new(app), run_config)
        }
        AppName::Particles => {
            let (app, run_config) = Particles::new();
            (Box::new(app), run_config)
        }
        AppName::Race => {
            let (app, run_config) = Race::new();
            (Box::new(app), run_config)
        }
    };

    let frame_rate = run_config.frame_rate;

    match runtime {
        Runtime::Window => window::run_main_loop(app, frame_rate),
        Runtime::Terminal => {
            let cell_width = args.cell_width;
            terminal::run_main_loop(app, frame_rate, cell_width)
        }
        Runtime::Debug => debug::run_main_loop(app),
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
