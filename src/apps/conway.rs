use crate::apps::{AppEvent, Info, TextBar};
use crate::{App, Point, ReadRenderBuf, RenderBuf};

#[derive(Debug)]
pub struct Conway {
    dimensions: (u8, u8),
    bufs: [RenderBuf<bool>; 2],
    buf_index: usize,
}

impl Conway {
    pub fn new(dimensions: (u8, u8), cells_offset: (i16, i16), live_cells: &[(i16, i16)]) -> Self {
        let mut buf0 = RenderBuf::new(dimensions);
        for &cell in live_cells {
            let cell = (cell.0 + cells_offset.0, cell.1 + cells_offset.1);
            buf0.set(cell, true);
        }
        let buf1 = RenderBuf::new(dimensions);

        Self {
            dimensions,
            bufs: [buf0, buf1],
            buf_index: 0,
        }
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
            if let Some(value) = self.bufs[self.buf_index].get((nx, ny)) {
                if value {
                    count += 1;
                }
            }
        }

        count
    }
}

impl App for Conway {
    fn info(&self) -> Info {
        Info {
            title: "Conway".to_string(),
            frame_rate: 15,
            text_bar: TextBar::Enabled {
                help_text: Some("Conway's game of life".to_string()),
            },
        }
    }

    fn run_frame(&mut self) -> Option<AppEvent> {
        let next_buf_index = (self.buf_index + 1) % 2;

        for y in 0..self.dimensions.1 {
            for x in 0..self.dimensions.0 {
                let x = x as i16;
                let y = y as i16;
                let is_live = self.bufs[self.buf_index].get((x, y)).unwrap();
                let live_neighbors = self.count_live_neighbors((x, y));
                if is_live {
                    // "Any live cell with two or three live neighbours survives."
                    // "All other live cells die in the next generation"
                    self.bufs[next_buf_index].set((x, y), [2, 3].contains(&live_neighbors));
                } else {
                    // "Any dead cell with three live neighbours becomes a live cell."
                    // "All other dead cells stay dead."
                    self.bufs[next_buf_index].set((x, y), live_neighbors == 3);
                }
            }
        }

        self.buf_index = next_buf_index;

        None
    }

    fn render_buf(&self) -> &dyn ReadRenderBuf {
        &self.bufs[self.buf_index]
    }
}
