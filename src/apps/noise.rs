use crate::apps::RunConfig;
use crate::{App, Cell, Graphics, GraphicsBuf};
use rand::Rng;

pub struct Noise {
    graphics: Graphics,
    empty_indices: Vec<usize>,
}

impl Noise {
    pub fn new() -> (Self, RunConfig) {
        let dimensions = (10, 5);
        let buf = GraphicsBuf::new(dimensions);
        let mut empty_indices = vec![];
        for i in 0..buf.buf.len() {
            empty_indices.push(i as usize);
        }
        let graphics = Graphics::new("Noise".to_string(), None, buf);
        let run_config = RunConfig { frame_rate: 15 };
        (
            Self {
                graphics,
                empty_indices,
            },
            run_config,
        )
    }
}

impl App for Noise {
    fn run_frame(&mut self) {
        if !self.empty_indices.is_empty() {
            let i = rand::thread_rng().gen_range(0..self.empty_indices.len());
            let buf_index = self.empty_indices[i];
            self.graphics.buf.set_by_index(buf_index, Cell::filled());
            self.empty_indices.swap_remove(i);

            if self.empty_indices.is_empty() {
                self.graphics.title = "The end.".to_string();
            }
        }
    }

    fn graphics(&self) -> &Graphics {
        &self.graphics
    }
}
