use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Canvas, Color, DrawMode, DrawParam, Mesh, Quad, Rect, Text};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::{Context, ContextBuilder, GameResult};

use crate::{App, PanelItem};
use crate::{Cell, GraphicsBuf};

const GRAPHICS_MARGIN: f32 = 10.0;
const CELL_SIZE: f32 = 30.0;
const TEXT_AREA_WIDTH: f32 = 300.0;

pub fn run_main_loop(app: Box<dyn App>, frame_rate: u32) -> ! {
    let side_panel = app.graphics().side_panel();
    let title = app.graphics().title.to_string();
    let (w, h) = app.graphics().buf.dimensions();

    let window_w = GRAPHICS_MARGIN * 2.0
        + CELL_SIZE * w as f32
        + if side_panel.is_some() {
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

    let event_handler = AppEventHandler {
        app,
        scaling: CELL_SIZE,
        frame_rate,
        title,
    };

    event::run(ctx, event_loop, event_handler);
}

struct AppEventHandler {
    app: Box<dyn App>,
    scaling: f32,
    frame_rate: u32,
    title: String,
}

impl EventHandler for AppEventHandler {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while ctx.time.check_update_time(self.frame_rate) {
            self.app.run_frame();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        let buf = &self.app.graphics().buf;

        if self.app.graphics().title[..] != self.title[..] {
            self.title = self.app.graphics().title.clone();
            ctx.gfx.set_window_title(&self.title);
        }

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

        self.render_buf(&mut canvas, buf, [GRAPHICS_MARGIN, GRAPHICS_MARGIN]);

        if let Some(panel) = self.app.graphics().side_panel() {
            let mut y = GRAPHICS_MARGIN;
            let margin = 20.0;
            for item in &panel.items {
                let destination = [GRAPHICS_MARGIN * 2.0 + graphics_width, y];
                match item {
                    PanelItem::TextItem { text } => {
                        let mut text = Text::new(text);
                        text.set_scale(30.0)
                            .set_bounds([TEXT_AREA_WIDTH, graphics_height]);
                        let text_size = text.measure(ctx).unwrap();
                        canvas.draw(&text, DrawParam::default().dest(destination));
                        y += text_size.y + margin;
                    }
                    PanelItem::GraphicsItem { buf } => {
                        self.render_buf(&mut canvas, buf, destination);
                        y += buf.dimensions.1 as f32 * self.scaling + margin;
                    }
                }
            }
        }

        canvas.finish(ctx)
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
                self.app.handle_pressed_key(ch);
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
}

impl AppEventHandler {
    fn render_buf(&self, canvas: &mut Canvas, buf: &GraphicsBuf, destination: [f32; 2]) {
        for y in 0..buf.dimensions().1 {
            for x in 0..buf.dimensions().0 {
                let Cell(ch, (r, g, b)) = buf.get((x as i16, y as i16)).unwrap();
                if ch != b' ' {
                    canvas.draw(
                        &Quad,
                        DrawParam::default()
                            .color(Color::from_rgb(r, g, b))
                            .scale([self.scaling, self.scaling])
                            .dest([
                                destination[0] + self.scaling * x as f32,
                                destination[1] + self.scaling * y as f32,
                            ]),
                    );
                }
            }
        }
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
