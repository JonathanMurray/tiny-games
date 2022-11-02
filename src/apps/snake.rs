use crate::apps::{AppEvent, Info, TextBar};
use crate::translated;
use crate::{App, Cell, Color, Direction, Point, ReadRenderBuf, RenderBuf};
use rand::seq::SliceRandom;

#[derive(Debug)]
pub struct Snake {
    game_size: (u8, u8),
    alive: bool,
    snake: Vec<Point>,
    direction: Direction,
    food: Point,
    buf: RenderBuf<Cell>,
}

const FOOD_SYMBOL: char = 'O';
const SNAKE_COLOR: Color = (255, 255, 100);
const FOOD_COLOR: Color = (255, 100, 100);

impl Snake {
    pub fn new() -> Self {
        let game_size: (u8, u8) = (30, 20);
        let snake_pos: Point = (1, 5);
        let snake = vec![snake_pos];

        let mut buf = RenderBuf::new(game_size);
        let direction = Direction::Right;
        buf.set(
            snake_pos,
            Cell(Self::direction_symbol(direction) as u8, SNAKE_COLOR),
        );

        let mut this = Self {
            game_size,
            alive: true,
            snake,
            direction,
            food: (3, 5),
            buf,
        };

        let food = this.pick_new_food_location();
        this.food = food;
        this.buf.set(food, Cell(FOOD_SYMBOL as u8, FOOD_COLOR));
        this
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
    fn info(&self) -> Info {
        let help_text = "\
Use WASD keys to control the snake!
";
        Info {
            title: "Snake".to_string(),
            frame_rate: 10,
            text_bar: TextBar::Enabled {
                help_text: Some(help_text.to_string()),
            },
        }
    }

    fn run_frame(&mut self) -> Option<AppEvent> {
        if !self.alive {
            return None;
        }

        let head = *self.snake.last().unwrap();
        self.buf.set(head, Cell(b'O', SNAKE_COLOR));
        let new_head = translated(head, self.direction);
        if self.is_within_game_bounds(new_head) {
            if new_head == self.food {
                self.food = self.pick_new_food_location();
                self.buf.set(self.food, Cell(FOOD_SYMBOL as u8, FOOD_COLOR));
            } else {
                self.buf.set(self.snake[0], Cell(b' ', (255, 255, 255)));
                self.snake.remove(0);
            }

            if self.snake.contains(&new_head) {
                self.alive = false;
            } else {
                self.snake.push(new_head);
                let symbol = Self::direction_symbol(self.direction);
                self.buf.set(new_head, Cell(symbol as u8, SNAKE_COLOR));
            }
        } else {
            self.alive = false;
        }

        (!self.alive).then_some(AppEvent::SetTitle(format!(
            "Score: {:?}",
            self.snake.len() - 1
        )))
    }

    fn handle_pressed_key(&mut self, key: char) -> Option<AppEvent> {
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
        None
    }

    fn render_buf(&self) -> &dyn ReadRenderBuf {
        &self.buf
    }
}
