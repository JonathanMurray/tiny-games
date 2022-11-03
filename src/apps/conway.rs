use crate::apps::AppInit;
use crate::{App, Cell, Graphics, GraphicsBuf, PanelItem, Point, SidePanel};

pub struct Conway {
    dimensions: (u8, u8),
    graphics: Graphics,
    tmp_buf: GraphicsBuf,
}

impl Conway {
    pub fn new(
        dimensions: (u8, u8),
        cells_offset: (i16, i16),
        live_cells: &[(i16, i16)],
    ) -> (Self, AppInit) {
        let mut buf0 = GraphicsBuf::new(dimensions);
        for &cell in live_cells {
            let cell = (cell.0 + cells_offset.0, cell.1 + cells_offset.1);
            buf0.set(cell, Cell::filled());
        }
        let tmp_buf = GraphicsBuf::new(dimensions);

        let side_panel = Some(SidePanel {
            items: vec![PanelItem::TextItem {
                text: "Conway's game of life".to_string(),
            }],
        });

        let graphics = Graphics::new("Conway".to_string(), side_panel, buf0);
        let app_init = AppInit { frame_rate: 10 };

        (
            Self {
                dimensions,
                graphics,
                tmp_buf,
            },
            app_init,
        )
    }

    fn count_live_neighbors(&self, position: Point) -> u32 {
        let (x, y) = position;
        let mut count = 0;
        for neighbor in [
            // north west
            (x - 1, y - 1),
            // north
            (x, y - 1),
            // north east
            (x + 1, y - 1),
            // west
            (x - 1, y),
            // east
            (x + 1, y),
            // south west
            (x - 1, y + 1),
            // south
            (x, y + 1),
            // south east
            (x + 1, y + 1),
        ] {
            let (nx, ny) = neighbor;
            if let Some(value) = self.graphics.buf.get((nx, ny)) {
                if value == Cell::filled() {
                    count += 1;
                }
            }
        }

        count
    }
}

impl App for Conway {
    fn run_frame(&mut self) {
        for y in 0..self.dimensions.1 {
            for x in 0..self.dimensions.0 {
                let x = x as i16;
                let y = y as i16;
                let is_live = self.graphics.buf.get((x, y)).unwrap() != Cell::Blank;
                let live_neighbors = self.count_live_neighbors((x, y));
                if is_live {
                    // "Any live cell with two or three live neighbours survives."
                    // "All other live cells die in the next generation"
                    let stays_alive = [2, 3].contains(&live_neighbors);
                    self.tmp_buf.set(
                        (x, y),
                        if stays_alive {
                            Cell::filled()
                        } else {
                            Cell::Blank
                        },
                    );
                } else {
                    // "Any dead cell with three live neighbours becomes a live cell."
                    // "All other dead cells stay dead."
                    let becomes_alive = live_neighbors == 3;
                    self.tmp_buf.set(
                        (x, y),
                        if becomes_alive {
                            Cell::filled()
                        } else {
                            Cell::Blank
                        },
                    );
                }
            }
        }

        std::mem::swap(&mut self.graphics.buf, &mut self.tmp_buf);
    }

    fn graphics(&self) -> &Graphics {
        &self.graphics
    }
}
