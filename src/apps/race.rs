use crate::apps::RunConfig;
use crate::{App, Cell, Graphics, GraphicsBuf, PanelItem, Point, SidePanel};
use std::cmp::{max, min};
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct Race {
    graphics: Graphics,
    pos_on_screen: Point,
    crashed: bool,
    velocity: [i16; 2],
    world: World,
    cursor: Cursor,
    timer: u32,
    elapsed_time: u32,
}

struct World {
    dimensions: (u16, u16),
    car: Point,
    obstacles: Vec<Point>,
    grass: Vec<Point>,
}

const CAR: Cell = Cell::Colored((250, 250, 250));
const CRASH: Cell = Cell::Colored((250, 50, 50));
const GRASS: Cell = Cell::Colored((100, 150, 100));
const OBSTACLE: Cell = Cell::Colored((100, 100, 150));

impl Race {
    pub fn new() -> (Self, RunConfig) {
        let world = Self::load_map();

        let buf = GraphicsBuf::new((30, 30));

        let pos_on_screen = (14, 14);

        let max_minimap_size = 8u8;
        let minimap_size = if world.dimensions.0 > world.dimensions.1 {
            (
                max_minimap_size,
                (max_minimap_size as u16 * world.dimensions.1 / world.dimensions.0) as u8,
            )
        } else {
            (
                (max_minimap_size as u16 * world.dimensions.0 / world.dimensions.1) as u8,
                max_minimap_size,
            )
        };

        let minimap_buf = GraphicsBuf::new(minimap_size);

        let elapsed_time = 0;

        let graphics = Graphics::new(
            "Race".to_string(),
            Some(SidePanel {
                items: vec![
                    PanelItem::TextItem {
                        text: Self::time_text(elapsed_time),
                    },
                    PanelItem::TextItem {
                        text: "Minimap:".to_string(),
                    },
                    PanelItem::GraphicsItem {
                        buf: minimap_buf,
                    },
                    PanelItem::TextItem {
                        text: "Use WASD to control the car.\nThe blinking dot indicates where you are heading.".to_string(),
                    },
                ],
            }),
            buf,
        );
        let run_config = RunConfig { frame_rate: 30 };

        let mut this = Self {
            graphics,
            pos_on_screen,
            world,
            crashed: false,
            velocity: [0, 0],
            cursor: Cursor::new(pos_on_screen),
            timer: 0,
            elapsed_time,
        };
        this.update_graphics();
        (this, run_config)
    }

    fn time_text(elapsed_time: u32) -> String {
        format!("Time: {}", elapsed_time)
    }

    fn load_map() -> World {
        let mut car_pos_in_world = None;
        let mut obstacles = vec![];
        let mut grass = vec![];

        let file = File::open("src/apps/race_map.txt").unwrap();

        let reader = BufReader::new(file);
        let lines = reader.lines();
        let mut y = 0;
        let mut max_x = 0;
        for line in lines {
            for (x, ch) in line.unwrap().chars().enumerate() {
                let x = x as i16;
                if ch == 'x' {
                    obstacles.push((x, y));
                } else if ch == 'o' {
                    let previous = car_pos_in_world.replace((x, y));
                    assert_eq!(previous, None);
                } else if ch == '.' {
                    grass.push((x, y))
                }
                max_x = max(max_x, x);
            }
            y += 1;
        }
        let car_pos_in_world = car_pos_in_world.expect("Must specify car position");

        World {
            car: car_pos_in_world,
            obstacles,
            grass,
            dimensions: (max_x as u16, (y - 1) as u16),
        }
    }

    fn update_graphics(&mut self) {
        for i in
            0..self.graphics.buf.dimensions().0 as usize * self.graphics.buf.dimensions().1 as usize
        {
            self.graphics.buf.set_by_index(i, Cell::Blank);
        }

        for &world_pos in &self.world.obstacles {
            let on_screen = (
                world_pos.0 - self.world.car.0 + self.pos_on_screen.0,
                world_pos.1 - self.world.car.1 + self.pos_on_screen.1,
            );
            if on_screen.0 >= 0
                && on_screen.0 < self.graphics.buf.dimensions().0 as i16
                && on_screen.1 >= 0
                && on_screen.1 < self.graphics.buf.dimensions().1 as i16
            {
                self.graphics.buf.set(on_screen, OBSTACLE);
            }
        }

        for &world_pos in &self.world.grass {
            let on_screen = (
                world_pos.0 - self.world.car.0 + self.pos_on_screen.0,
                world_pos.1 - self.world.car.1 + self.pos_on_screen.1,
            );
            if on_screen.0 >= 0
                && on_screen.0 < self.graphics.buf.dimensions().0 as i16
                && on_screen.1 >= 0
                && on_screen.1 < self.graphics.buf.dimensions().1 as i16
            {
                self.graphics.buf.set(on_screen, GRASS);
            }
        }

        self.graphics
            .buf
            .set(self.pos_on_screen, if self.crashed { CRASH } else { CAR });

        self.draw_minimap();

        if !self.crashed {
            self.cursor.draw(&mut self.graphics.buf);
        }
    }

    fn draw_minimap(&mut self) {
        let buf = self
            .graphics
            .side_panel
            .as_mut()
            .unwrap()
            .unwrap_graphics_item_mut(2);
        let buf_w = buf.dimensions().0 as u16;
        let buf_h = buf.dimensions().1 as u16;

        for y in 0..buf_h {
            for x in 0..buf_w {
                let (x_car, y_car) = self.world.car;

                if x_car >= (self.world.dimensions.0 * x / buf_w) as i16
                    && x_car <= (self.world.dimensions.0 * (x + 1) / buf_w) as i16
                    && y_car >= (self.world.dimensions.1 * y / buf_h) as i16
                    && y_car <= (self.world.dimensions.1 * (y + 1) / buf_h) as i16
                {
                    buf.set((x as i16, y as i16), Cell::Colored((255, 255, 255)));
                } else {
                    buf.set((x as i16, y as i16), Cell::Colored((150, 150, 150)));
                }
            }
        }
    }
}

impl App for Race {
    fn run_frame(&mut self) {
        self.timer = (self.timer + 1) % 8;

        if self.timer == 0 && !self.crashed {
            self.elapsed_time += 1;
            *self
                .graphics
                .side_panel
                .as_mut()
                .unwrap()
                .unwrap_text_item_mut(0) = Self::time_text(self.elapsed_time);
            self.velocity[0] += self.cursor.direction[0];
            self.velocity[1] += self.cursor.direction[1];

            self.cursor.pos_on_screen = (
                self.pos_on_screen.0 + self.velocity[0],
                self.pos_on_screen.1 + self.velocity[1],
            );
            self.cursor.direction = [0, 0];

            let mut x0 = self.world.car.0;
            let mut y0 = self.world.car.1;
            let x_dst = self.world.car.0 + self.velocity[0];
            let y_dst = self.world.car.1 + self.velocity[1];
            while [x0, y0] != [x_dst, y_dst] {
                if (x_dst - x0).abs() > (y_dst - y0).abs() {
                    x0 += (x_dst - x0).signum();
                } else {
                    y0 += (y_dst - y0).signum();
                }
                if self.world.obstacles.contains(&(x0, y0)) {
                    self.crashed = true;
                    let text = self
                        .graphics
                        .side_panel
                        .as_mut()
                        .unwrap()
                        .unwrap_text_item_mut(3);
                    *text = "Game Over:\nYou crashed!".to_string();
                    break;
                }
            }

            self.world.car = (x0, y0);
        }

        self.cursor.update();

        self.update_graphics();
    }

    fn handle_pressed_key(&mut self, key: char) {
        self.cursor.handle_pressed_key(key);
    }

    fn graphics(&self) -> &Graphics {
        &self.graphics
    }
}

struct Cursor {
    pos_on_screen: Point,
    timer: u32,
    direction: [i16; 2],
}

impl Cursor {
    fn new(pos_on_screen: (i16, i16)) -> Self {
        Self {
            pos_on_screen,
            timer: 0,
            direction: [0, 0],
        }
    }

    fn update(&mut self) {
        self.timer = (self.timer + 1) % 10;
    }

    fn handle_pressed_key(&mut self, key: char) {
        match key {
            'w' => self.direction[1] = max(-1, self.direction[1] - 1),
            'a' => self.direction[0] = max(-1, self.direction[0] - 1),
            's' => self.direction[1] = min(self.direction[1] + 1, 1),
            'd' => self.direction[0] = min(self.direction[0] + 1, 1),
            _ => {}
        }
    }

    fn draw(&self, buf: &mut GraphicsBuf) {
        if self.timer < 6 {
            buf.set(
                (
                    self.pos_on_screen.0 + self.direction[0],
                    self.pos_on_screen.1 + self.direction[1],
                ),
                Cell::Colored((200, 250, 200)),
            );
        }
    }
}
