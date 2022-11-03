pub mod conway;

pub mod noise;
pub mod particles;
pub mod snake;
pub mod tetris;

use crate::Graphics;

pub trait App {
    fn run_frame(&mut self);
    fn handle_pressed_key(&mut self, _key: char) {}
    fn handle_released_key(&mut self, _key: char) {}
    fn graphics(&self) -> &Graphics;
}

pub struct RunConfig {
    pub frame_rate: u32,
}
