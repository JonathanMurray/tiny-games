pub mod conway;
pub mod noise;
pub mod snake;
pub mod tetris;

use crate::ReadRenderBuf;

pub trait App {
    fn info(&self) -> Info;
    fn run_frame(&mut self) -> Option<AppEvent>;
    fn handle_pressed_key(&mut self, _key: char) -> Option<AppEvent> {
        None
    }
    fn handle_released_key(&mut self, _key: char) {}
    fn render_buf(&self) -> &dyn ReadRenderBuf;
}

#[derive(Debug)]
pub struct Info {
    pub title: String,
    pub frame_rate: u32,
    pub text_bar: TextBar,
}

#[derive(Debug)]
pub enum TextBar {
    Enabled { help_text: Option<String> },
    Disabled,
}

#[derive(Debug)]

pub enum AppEvent {
    SetTitle(String),
}
