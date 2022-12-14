use crate::apps::RunConfig;
use crate::{
    translated, App, Cell, Color, Direction, Graphics, GraphicsBuf, PanelItem, Point, SidePanel,
};
use rand::seq::SliceRandom;

pub struct Tetris {
    graphics: Graphics,
    falling: Option<Tetromino>,
    upcoming: Shape,
    holding_down: bool,
    frame: u32,
    fall_delay: u32,
    score: u32,
}

impl Tetris {
    pub fn new() -> (Self, RunConfig) {
        let mut buf = GraphicsBuf::new((10, 20));
        let falling = Tetromino::at_top(generate_next());
        for block in falling.blocks() {
            buf.set(block, Cell::Colored(falling.color()));
        }
        let help_text = "\
Controls:
--------
A: move left
D: move right
W: rotate
S: fall faster
"
        .to_string();
        let score = 0;
        let upcoming = generate_next();
        let mut upcoming_buf = GraphicsBuf::new((4, 2));
        Self::render_upcoming_buf(upcoming, &mut upcoming_buf);

        let graphics = Graphics::new(
            "Tetris".to_string(),
            Some(SidePanel {
                items: vec![
                    PanelItem::TextItem {
                        text: format!("Score: {}", score),
                    },
                    PanelItem::TextItem {
                        text: "Next:".to_string(),
                    },
                    PanelItem::GraphicsItem { buf: upcoming_buf },
                    PanelItem::TextItem { text: help_text },
                ],
            }),
            buf,
        );
        let frame_rate = 30;

        (
            Self {
                graphics,
                falling: Some(falling),
                upcoming,
                holding_down: false,
                frame: 0,
                fall_delay: 15,
                score,
            },
            RunConfig { frame_rate },
        )
    }

    fn render_upcoming_buf(shape: Shape, buf: &mut GraphicsBuf) {
        {
            for i in 0..buf.dimensions().0 * buf.dimensions().1 {
                buf.set_by_index(i as usize, Cell::Blank);
            }

            let tetromino = Tetromino::in_upcoming_hint(shape);
            for point in tetromino.blocks() {
                buf.set(point, Cell::Colored(tetromino.color()));
            }
        }
    }
}

impl App for Tetris {
    fn run_frame(&mut self) {
        if self.falling.is_none() {
            // Game ended in a previous frame
            return;
        }

        self.frame += 1;

        if !self.holding_down && self.frame % self.fall_delay != 0 {
            // Simulate slower fall speed by ignoring some frames
            return;
        }

        if !self.try_move(Direction::Down) {
            // The falling tetromino just landed

            self.remove_any_complete_rows();

            self.falling = None;
            let next = Tetromino::at_top(self.upcoming);
            self.upcoming = generate_next();
            match &mut self.graphics.side_panel.as_mut().unwrap().items[2] {
                PanelItem::GraphicsItem { buf } => {
                    Self::render_upcoming_buf(self.upcoming, buf);
                }
                unexpected => panic!("Unexpected panel item: {:?}", unexpected),
            }

            let game_over = self.would_collide(next);

            for block in next.blocks() {
                self.graphics.buf.set(block, Cell::Colored(next.color()));
            }

            if game_over {
                self.graphics.side_panel.as_mut().unwrap().items[0] = PanelItem::TextItem {
                    text: format!("Game over.\nScore: {:?}", self.score),
                };

                return;
            }

            self.falling = Some(next);
        }
    }

    fn handle_pressed_key(&mut self, key: char) {
        if self.falling.is_none() {
            // Game over
            return;
        }
        // TODO handle hold down movement
        match key {
            'a' => {
                self.try_move(Direction::Left);
            }
            'd' => {
                self.try_move(Direction::Right);
            }
            'w' => {
                self.rotate_if_possible();
            }
            's' => {
                let was_already = self.holding_down;
                self.holding_down = true;
                if !was_already {
                    self.run_frame();
                }
            }
            _ => {}
        };
    }

    fn handle_released_key(&mut self, key: char) {
        if key == 's' {
            self.holding_down = false;
        }
    }

    fn graphics(&self) -> &Graphics {
        &self.graphics
    }
}

impl Tetris {
    fn try_move(&mut self, direction: Direction) -> bool {
        let moved = self.falling.unwrap().translate(direction);

        if self.would_collide(moved) {
            false
        } else {
            for block in self.falling.unwrap().blocks() {
                self.graphics.buf.set(block, Cell::Blank);
            }
            for moved_block in moved.blocks() {
                self.graphics
                    .buf
                    .set(moved_block, Cell::Colored(moved.color()));
            }
            self.falling = Some(moved);
            true
        }
    }

    fn rotate_if_possible(&mut self) {
        let rotated = self.falling.unwrap().rotated();

        if !self.would_collide(rotated) {
            for block in self.falling.unwrap().blocks() {
                self.graphics.buf.set(block, Cell::Blank);
            }
            for rotated_block in rotated.blocks() {
                self.graphics
                    .buf
                    .set(rotated_block, Cell::Colored(rotated.color()));
            }
            self.falling = Some(rotated);
        }
    }

    fn remove_any_complete_rows(&mut self) {
        let mut y = self.graphics.buf.dimensions().1 as i16 - 1;
        while y >= 0 {
            let mut is_complete_row = true;
            for x in 0..self.graphics.buf.dimensions.0 {
                if self.graphics.buf.get((x as i16, y as i16)).unwrap() == Cell::Blank {
                    is_complete_row = false;
                    break;
                }
            }
            if is_complete_row {
                self.score += 1;
                self.graphics.side_panel.as_mut().unwrap().items[0] = PanelItem::TextItem {
                    text: format!("Score: {:?}", self.score),
                };
                if self.score % 2 == 0 {
                    self.fall_delay = std::cmp::max(1, self.fall_delay - 1);
                }
                for shift_y in (0..y + 1).rev() {
                    let shift_y = shift_y as i16;
                    for x in 0..self.graphics.buf.dimensions.0 {
                        let x = x as i16;
                        let value_above = self
                            .graphics
                            .buf
                            .get((x, shift_y - 1))
                            .unwrap_or(Cell::Blank);
                        self.graphics.buf.set((x, shift_y), value_above);
                    }
                }
            } else {
                y -= 1;
            }
        }
    }

    fn would_collide(&self, hypothetical: Tetromino) -> bool {
        hypothetical.blocks().iter().any(|block| {
            // we ignore "self collision"
            let collision_with_falling = self
                .falling
                .map(|tetromino| tetromino.blocks().contains(block))
                .unwrap_or(false);
            let collision = self
                .graphics
                .buf
                .get(*block)
                .map(|cell| cell != Cell::Blank)
                .unwrap_or(true);
            collision && !collision_with_falling
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct Tetromino {
    origin: Point,
    orientation: Orientation,
    shape: Shape,
}

#[derive(Debug, Clone, Copy)]
enum Shape {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl Tetromino {
    fn at_top(shape: Shape) -> Self {
        let orientation = Orientation::First;
        let origin = match shape {
            Shape::I => (3, -2),
            Shape::O => (4, 0),
            Shape::T => (4, -1),
            Shape::S => (4, -1),
            Shape::Z => (4, -1),
            Shape::J => (4, -1),
            Shape::L => (4, -1),
        };
        Self {
            origin,
            orientation,
            shape,
        }
    }

    fn in_upcoming_hint(shape: Shape) -> Self {
        let orientation = Orientation::First;
        let origin = match shape {
            Shape::I => (0, -2),
            Shape::O => (0, 0),
            Shape::T => (0, -1),
            Shape::S => (0, -1),
            Shape::Z => (0, -1),
            Shape::J => (0, -1),
            Shape::L => (0, -1),
        };
        Self {
            origin,
            orientation,
            shape,
        }
    }

    fn blocks(&self) -> [Point; 4] {
        match self.shape {
            Shape::I => match self.orientation {
                Orientation::First | Orientation::Third => {
                    self.resolve([(0, 2), (1, 2), (2, 2), (3, 2)])
                }
                Orientation::Second | Orientation::Fourth => {
                    self.resolve([(2, 0), (2, 1), (2, 2), (2, 3)])
                }
            },
            Shape::O => self.resolve([(0, 0), (1, 0), (0, 1), (1, 1)]),
            Shape::T => match self.orientation {
                Orientation::First => self.resolve([(0, 1), (1, 1), (2, 1), (1, 2)]),
                Orientation::Second => self.resolve([(0, 1), (1, 0), (1, 1), (1, 2)]),
                Orientation::Third => self.resolve([(0, 1), (1, 1), (2, 1), (1, 0)]),
                Orientation::Fourth => self.resolve([(2, 1), (1, 0), (1, 1), (1, 2)]),
            },
            Shape::S => match self.orientation {
                Orientation::First | Orientation::Third => {
                    self.resolve([(0, 2), (1, 2), (1, 1), (2, 1)])
                }
                Orientation::Second | Orientation::Fourth => {
                    self.resolve([(1, 0), (1, 1), (2, 1), (2, 2)])
                }
            },
            Shape::Z => match self.orientation {
                Orientation::First | Orientation::Third => {
                    self.resolve([(0, 1), (1, 1), (1, 2), (2, 2)])
                }
                Orientation::Second | Orientation::Fourth => {
                    self.resolve([(1, 2), (1, 1), (2, 1), (2, 0)])
                }
            },
            Shape::J => match self.orientation {
                Orientation::First => self.resolve([(0, 1), (1, 1), (2, 1), (2, 2)]),
                Orientation::Second => self.resolve([(0, 2), (1, 2), (1, 1), (1, 0)]),
                Orientation::Third => self.resolve([(0, 0), (0, 1), (1, 1), (2, 1)]),
                Orientation::Fourth => self.resolve([(1, 2), (1, 1), (1, 0), (2, 0)]),
            },
            Shape::L => match self.orientation {
                Orientation::First => self.resolve([(0, 2), (0, 1), (1, 1), (2, 1)]),
                Orientation::Second => self.resolve([(0, 0), (1, 0), (1, 1), (1, 2)]),
                Orientation::Third => self.resolve([(0, 1), (1, 1), (2, 1), (2, 0)]),
                Orientation::Fourth => self.resolve([(1, 0), (1, 1), (1, 2), (2, 2)]),
            },
        }
    }

    fn color(&self) -> Color {
        match self.shape {
            Shape::I => (235, 50, 50),
            Shape::O => (50, 235, 50),
            Shape::T => (80, 80, 235),
            Shape::S => (170, 170, 50),
            Shape::Z => (50, 170, 170),
            Shape::J => (170, 50, 170),
            Shape::L => (200, 100, 100),
        }
    }

    fn resolve(&self, points: [Point; 4]) -> [Point; 4] {
        points.map(|r| (self.origin.0 + r.0, self.origin.1 + r.1))
    }

    fn translate(&self, direction: Direction) -> Self {
        Self {
            origin: translated(self.origin, direction),
            orientation: self.orientation,
            shape: self.shape,
        }
    }

    fn rotated(&mut self) -> Self {
        Self {
            origin: self.origin,
            orientation: self.orientation.rotated(),
            shape: self.shape,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Orientation {
    First,
    Second,
    Third,
    Fourth,
}

impl Orientation {
    fn rotated(&self) -> Self {
        match self {
            Orientation::First => Orientation::Second,
            Orientation::Second => Orientation::Third,
            Orientation::Third => Orientation::Fourth,
            Orientation::Fourth => Orientation::First,
        }
    }
}

fn generate_next() -> Shape {
    let shapes = [
        Shape::I,
        Shape::O,
        Shape::T,
        Shape::S,
        Shape::Z,
        Shape::J,
        Shape::L,
    ];

    *shapes.choose(&mut rand::thread_rng()).unwrap()
}
