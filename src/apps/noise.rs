use crate::apps::{AppEvent, Info, TextBar};
use crate::{App, ReadRenderBuf, RenderBuf};
use rand::Rng;

#[derive(Debug)]
pub struct Noise {
    buf: RenderBuf<bool>,
    empty_indices: Vec<usize>,
}

impl Noise {
    pub fn new(dimensions: (u8, u8)) -> Self {
        let buf = RenderBuf::new(dimensions);
        let mut empty_indices = vec![];
        for i in 0..buf.buf.len() {
            empty_indices.push(i as usize);
        }
        Self { buf, empty_indices }
    }
}

impl App for Noise {
    fn info(&self) -> Info {
        Info {
            title: "Noise".to_string(),
            frame_rate: 15,
            text_bar: TextBar::Disabled,
        }
    }

    fn run_frame(&mut self) -> Option<AppEvent> {
        if !self.empty_indices.is_empty() {
            let i = rand::thread_rng().gen_range(0..self.empty_indices.len());
            let buf_index = self.empty_indices[i];
            self.buf.set_by_index(buf_index, true);
            self.empty_indices.swap_remove(i);
        }
        None
    }

    fn render_buf(&self) -> &dyn ReadRenderBuf {
        &self.buf
    }
}
