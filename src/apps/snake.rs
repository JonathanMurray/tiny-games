use crate::apps::AppInit;
use crate::{translated, Cell, Graphics, GraphicsBuf, PanelItem, SidePanel};
use crate::{App, Color, Direction, Point};
use rand::seq::SliceRandom;

pub struct Snake {
    game_size: (u8, u8),
    alive: bool,
    snake: Vec<Point>,
    direction: Direction,
    food: Point,
    graphics: Graphics,
    score: u32,
}

const FOOD_SYMBOL: char = 'O';
const SNAKE_COLOR: Color = (255, 255, 100);
const FOOD_COLOR: Color = (255, 100, 100);

impl Snake {
    pub fn new() -> (Self, AppInit) {
        let game_size: (u8, u8) = (30, 20);
        let snake_pos: Point = (1, 5);
        let snake = vec![snake_pos];

        let mut buf = GraphicsBuf::new(game_size);
        let direction = Direction::Right;
        buf.set(
            snake_pos,
            Cell(Self::direction_symbol(direction) as u8, SNAKE_COLOR),
        );

        let help_text = "Use WASD keys to control the snake!".to_string();

        let score = 0;
        let graphics = Graphics::new(
            "Snake".to_string(),
            Some(SidePanel {
                items: vec![
                    PanelItem {
                        text: format!("Score: {}", score),
                    },
                    PanelItem { text: help_text },
                ],
            }),
            buf,
        );

        let mut this = Self {
            game_size,
            alive: true,
            snake,
            direction,
            food: (3, 5),
            graphics,
            score,
        };

        let food = this.pick_new_food_location();
        this.food = food;
        this.graphics
            .buf
            .set(food, Cell(FOOD_SYMBOL as u8, FOOD_COLOR));
        (this, AppInit { frame_rate: 10 })
    }

    fn set_direction(&mut self, direction: Direction) {
        if self.snake.len() >= 2 {
            let neck = self.snake[self.snake.len() - 2];
            if translated(*self.snake.last().unwrap(), direction) == neck {
                return;
            }
        }
        self.direction = direction;
    }

    fn is_within_game_bounds(&self, point: Point) -> bool {
        point.0 >= 0
            && point.1 >= 0
            && point.0 < self.game_size.0 as i16
            && point.1 < self.game_size.1 as i16
    }

    fn pick_new_food_location(&self) -> Point {
        let mut food_candidates: Vec<Point> = vec![];
        for x in 0..self.game_size.0 {
            for y in 0..self.game_size.1 {
                let x = x as i16;
                let y = y as i16;
                if !self.snake.contains(&(x, y)) {
                    food_candidates.push((x, y))
                }
            }
        }

        *food_candidates
            .choose(&mut rand::thread_rng())
            .expect("Vacant food location")
    }

    fn direction_symbol(direction: Direction) -> char {
        match direction {
            Direction::Up => '^',
            Direction::Left => '<',
            Direction::Down => 'V',
            Direction::Right => '>',
        }
    }
}

impl App for Snake {
    fn run_frame(&mut self) {
        if !self.alive {
            return;
        }

        let head = *self.snake.last().unwrap();
        self.graphics.buf.set(head, Cell(b'O', SNAKE_COLOR));
        let new_head = translated(head, self.direction);
        if self.is_within_game_bounds(new_head) {
            if new_head == self.food {
                self.food = self.pick_new_food_location();
                self.graphics
                    .buf
                    .set(self.food, Cell(FOOD_SYMBOL as u8, FOOD_COLOR));
                self.score += 1;
                self.graphics.side_panel.as_mut().unwrap().items[0].text =
                    format!("Score: {}", self.score);
            } else {
                self.graphics
                    .buf
                    .set(self.snake[0], Cell(b' ', (255, 255, 255)));
                self.snake.remove(0);
            }

            if self.snake.contains(&new_head) {
                self.alive = false;
            } else {
                self.snake.push(new_head);
                let symbol = Self::direction_symbol(self.direction);
                self.graphics
                    .buf
                    .set(new_head, Cell(symbol as u8, SNAKE_COLOR));
            }
        } else {
            self.alive = false;
        }

        if !self.alive {
            self.graphics.side_panel.as_mut().unwrap().items[0].text =
                format!("Game over.\nScore: {:?}", self.score);
        }
    }

    fn handle_pressed_key(&mut self, key: char) {
        let direction = match key {
            'w' => Some(Direction::Up),
            'a' => Some(Direction::Left),
            's' => Some(Direction::Down),
            'd' => Some(Direction::Right),
            _ => None,
        };
        if let Some(direction) = direction {
            self.set_direction(direction);
        }
    }

    fn graphics(&self) -> &Graphics {
        &self.graphics
    }
}
