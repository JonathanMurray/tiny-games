use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color, DrawMode, DrawParam, Mesh, Quad, Rect, Text};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::{Context, ContextBuilder, GameResult};

use crate::apps::{AppEvent, Info, TextBar};
use crate::{App, Cell};

const GRAPHICS_MARGIN: f32 = 10.0;
const CELL_SIZE: f32 = 30.0;
const TEXT_AREA_WIDTH: f32 = 300.0;

pub fn run_main_loop(app: Box<dyn App>) -> ! {
    let (w, h) = app.render_buf().dimensions();

    let Info {
        title,
        frame_rate,
        text_bar,
    } = app.info();

    let window_w = GRAPHICS_MARGIN * 2.0
        + CELL_SIZE * w as f32
        + if matches!(text_bar, TextBar::Enabled { .. }) {
            TEXT_AREA_WIDTH
        } else {
            0.0
        };
    let window_h = GRAPHICS_MARGIN * 2.0 + CELL_SIZE * h as f32;

    let (ctx, event_loop) = ContextBuilder::new("ggez_ui", "some_author")
        .window_setup(WindowSetup::default().title(&title))
        .window_mode(WindowMode::default().dimensions(window_w, window_h))
        .build()
        .unwrap();

    let help_text = match text_bar {
        TextBar::Enabled { help_text } => help_text,
        TextBar::Disabled => None,
    };

    let event_handler = AppEventHandler {
        app,
        scaling: CELL_SIZE,
        frame_rate,
        help_text,
    };

    event::run(ctx, event_loop, event_handler);
}

struct AppEventHandler {
    app: Box<dyn App>,
    scaling: f32,
    frame_rate: u32,
    help_text: Option<String>,
}

impl EventHandler for AppEventHandler {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while ctx.time.check_update_time(self.frame_rate) {
            if let Some(event) = self.app.run_frame() {
                match event {
                    AppEvent::SetTitle(title) => ctx.gfx.set_window_title(&title),
                }
            }
        }

        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: KeyInput,
        _repeated: bool,
    ) -> GameResult {
        if let KeyInput {
            keycode: Some(key), ..
        } = input
        {
            if key == KeyCode::Q {
                ctx.request_quit();
                return Ok(());
            }

            if let Some(ch) = keycode_to_char(key) {
                if let Some(event) = self.app.handle_pressed_key(ch) {
                    match event {
                        AppEvent::SetTitle(title) => ctx.gfx.set_window_title(&title),
                    }
                }
            }
        }

        Ok(())
    }

    fn key_up_event(&mut self, _ctx: &mut Context, input: KeyInput) -> GameResult {
        if let KeyInput {
            keycode: Some(key), ..
        } = input
        {
            if let Some(ch) = keycode_to_char(key) {
                self.app.handle_released_key(ch);
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        let buf = self.app.render_buf();

        let graphics_width = self.scaling * buf.dimensions().0 as f32;
        let graphics_height = self.scaling * buf.dimensions().1 as f32;

        let bg = Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(0.0, 0.0, graphics_width, graphics_height),
            Color::from_rgb(50, 50, 50),
        )?;

        canvas.draw(
            &bg,
            DrawParam::default().dest([GRAPHICS_MARGIN, GRAPHICS_MARGIN]),
        );

        for y in 0..buf.dimensions().1 {
            for x in 0..buf.dimensions().0 {
                let Cell(ch, (r, g, b)) = buf.get_cell((x as i16, y as i16));
                if ch != b' ' {
                    canvas.draw(
                        &Quad,
                        DrawParam::default()
                            .color(Color::from_rgb(r, g, b))
                            .scale([self.scaling, self.scaling])
                            .dest([
                                GRAPHICS_MARGIN + self.scaling * x as f32,
                                GRAPHICS_MARGIN + self.scaling * y as f32,
                            ]),
                    );
                }
            }
        }

        if let Some(text) = &self.help_text {
            let mut text = Text::new(text);
            text.set_scale(30.0)
                .set_bounds([TEXT_AREA_WIDTH, graphics_height]);
            canvas.draw(
                &text,
                DrawParam::default()
                    .dest([GRAPHICS_MARGIN * 2.0 + graphics_width, GRAPHICS_MARGIN]),
            );
        }

        canvas.finish(ctx)
    }
}

fn keycode_to_char(key_code: KeyCode) -> Option<char> {
    use KeyCode::*;
    match key_code {
        W => Some('w'),
        A => Some('a'),
        S => Some('s'),
        D => Some('d'),
        Q => Some('q'),
        unhandled => {
            eprintln!("Unhandled key: {:?}", unhandled);
            None
        }
    }
}
