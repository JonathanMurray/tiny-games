use crate::apps::RunConfig;
use crate::{App, Cell, Color, Graphics, GraphicsBuf, PanelItem, SidePanel};
use rand::prelude::SliceRandom;
use rand::Rng;
use std::cmp::{max, min};

const SOLID: Cell = Cell::Colored((120, 70, 70));

pub struct Particles {
    graphics: Graphics,
    frame: u32,
    particles: Vec<Particle>,
    spawn_rate: f64,
    spawn_velocity: [i16; 2],
}

#[derive(Debug, Copy, Clone)]
struct Particle {
    color: Color,
    position: [i16; 2],
    velocity: [i16; 2],
}

impl Particles {
    pub fn new() -> (Self, RunConfig) {
        let dimensions = (30, 30);
        let mut buf = GraphicsBuf::new(dimensions);

        let particles = vec![];

        let solid_cells = [
            (0, 3),
            (1, 3),
            (2, 3),
            (3, 3),
            (4, 3),
            (5, 3),
            (6, 3),
            (7, 3),
            (8, 3),
            (9, 3),
            (10, 3),
            (12, 13),
            (13, 13),
            (14, 13),
            (15, 13),
            (16, 13),
            (17, 13),
            (18, 13),
            (18, 12),
            (18, 11),
            (6, 29),
            (6, 28),
            (6, 27),
            (6, 26),
            (5, 26),
            (5, 25),
            (7, 29),
            (7, 28),
            (7, 27),
            (7, 26),
            (8, 26),
            (8, 25),
        ];

        for cell in solid_cells {
            buf.set(cell, SOLID);
        }

        let graphics = Graphics::new(
            "Particles".to_string(),
            Some(SidePanel {
                items: vec![PanelItem::TextItem {
                    text: "".to_string(),
                },
                PanelItem::TextItem {
                    text: "Control spawn rate with 'W' and 'S'\nControl spawn velocity with 'A' and 'D'".to_string()
                }],
            }),
            buf,
        );
        let run_config = RunConfig { frame_rate: 5 };
        (
            Self {
                graphics,
                frame: 0,
                particles,
                spawn_rate: 0.1,
                spawn_velocity: [1, 0],
            },
            run_config,
        )
    }

    fn is_free(buf: &GraphicsBuf, position: (i16, i16)) -> bool {
        buf.get(position)
            .map(|cell| cell == Cell::Blank)
            .unwrap_or(false)
    }

    fn update_info_text(&mut self) {
        match &mut self.graphics.side_panel.as_mut().unwrap().items[0] {
            PanelItem::TextItem { text } => {
                *text = format!(
                    "Particles: {}\nSpawn rate: {:.2}\nSpawn velocity: {:?}",
                    self.particles.len(),
                    self.spawn_rate,
                    self.spawn_velocity
                );
            }
            _ => panic!("How?"),
        }
    }
}

impl App for Particles {
    fn run_frame(&mut self) {
        self.frame += 1;
        let buf = &mut self.graphics.buf;

        // VELOCITY / MOVEMENT
        // --------
        for particle in &mut self.particles {
            let [x, y] = particle.position;

            let (mut x0, mut y0) = (x, y);
            let x_dst = x + particle.velocity[0];
            let y_dst = y + particle.velocity[1];

            let mut collision = false;

            while !collision && (x0, y0) != (x_dst, y_dst) {
                let dx = x_dst - x0;
                let dy = y_dst - y0;
                let next_x = x0 + i16::signum(dx);
                let next_y = y0 + i16::signum(dy);
                let can_move_hor = dx != 0 && Self::is_free(buf, (next_x, y0));
                let can_move_vert = dy != 0 && Self::is_free(buf, (x0, next_y));
                if i16::abs(dx) > i16::abs(dy) && can_move_hor {
                    x0 = next_x;
                } else if i16::abs(dy) > i16::abs(dx) && can_move_vert {
                    y0 = next_y;
                } else if (dx, dy) != (0, 0) && Self::is_free(buf, (next_x, next_y)) {
                    (x0, y0) = (next_x, next_y);
                } else if can_move_hor {
                    x0 = next_x;
                } else if can_move_vert {
                    y0 = next_y;
                } else {
                    collision = true;

                    if x0 != x_dst {
                        //bounce horizontally
                        particle.velocity[0] = (particle.velocity[0] as f32 * -0.6) as i16;
                    } else if y0 != y_dst {
                        particle.velocity[1] = 0;
                    } else {
                        panic!("How?")
                    }
                }
            }

            if (x0, y0) != (x, y) {
                buf.set((x, y), Cell::Blank);
                buf.set((x0, y0), Cell::Colored(particle.color));
                particle.position = [x0, y0];
            }
        }

        // FORCES
        // ------
        for particle in &mut self.particles {
            let [x, y] = particle.position;

            if Self::is_free(buf, (x, y + 1)) {
                // Gravity (straight down)
                particle.velocity[1] += 1;
                if rand::thread_rng().gen_bool(0.1) {
                    // to make things look less static
                    particle.velocity[1] += 1;
                }
            } else if i16::abs(particle.velocity[0]) > 0 && rand::thread_rng().gen_bool(0.5) {
                // Friction
                particle.velocity[0] =
                    (particle.velocity[0].abs() - 1) * particle.velocity[0].signum();
            }

            if particle.velocity == [0, 0] {
                // Gravity (diagonally)
                let mut falling = false;
                for &dx in [[-1, 1], [1, -1]].choose(&mut rand::thread_rng()).unwrap() {
                    if Self::is_free(buf, (x + dx, y + 1)) {
                        particle.velocity = [dx, 1];
                        falling = true;
                        break;
                    }
                }

                if !falling {
                    // Side-way movement (from forces or by chance)
                    if Self::is_free(buf, (x - 1, y)) && !Self::is_free(buf, (x + 1, y)) {
                        particle.velocity[0] = -1;
                    } else if Self::is_free(buf, (x + 1, y)) && !Self::is_free(buf, (x - 1, y)) {
                        particle.velocity[0] = 1;
                    } else if buf
                        .get((x, y + 1))
                        .map(|cell| cell != Cell::Blank && cell != SOLID)
                        .unwrap_or(false)
                    {
                        // If above liquid, sometimes move side-way by chance
                        if rand::thread_rng().gen_bool(0.2) {
                            let dx = *[-1, 1].choose(&mut rand::thread_rng()).unwrap();
                            if Self::is_free(buf, (x + dx, y)) {
                                particle.velocity[0] = dx;
                            }
                        }
                    }
                }
            }
        }

        // SPAWN NEW
        // ---------
        if rand::thread_rng().gen_bool(self.spawn_rate) {
            let position = *[[0, 1], [0, 0], [0, 2]]
                .choose(&mut rand::thread_rng())
                .unwrap();
            let color = *[(100, 160, 220), (120, 120, 250), (150, 150, 250)]
                .choose(&mut rand::thread_rng())
                .unwrap();
            let particle = Particle {
                color,
                position,
                velocity: self.spawn_velocity,
            };
            buf.set((position[0], position[1]), Cell::Colored(particle.color));
            self.particles.push(particle);
        }

        self.update_info_text()
    }

    fn handle_pressed_key(&mut self, key: char) {
        match key {
            'w' => {
                self.spawn_rate += 0.05;
                if self.spawn_rate > 1.0 {
                    self.spawn_rate = 1.0;
                }
            }
            's' => {
                self.spawn_rate -= 0.05;
                if self.spawn_rate < 0.0 {
                    self.spawn_rate = 0.0;
                }
            }
            'a' => self.spawn_velocity[0] = max(self.spawn_velocity[0] - 1, 1),
            'd' => self.spawn_velocity[0] = min(self.spawn_velocity[0] + 1, 10),
            _ => {}
        }
    }

    fn graphics(&self) -> &Graphics {
        &self.graphics
    }
}
