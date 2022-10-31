use backtrace::Backtrace;
use crossterm::terminal::{ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use std::io::{Stdout, Write};
use tui::backend::CrosstermBackend;
use tui::widgets::{Block, BorderType, Borders, Paragraph, Widget};
use tui::Terminal;

use crate::apps::{AppEvent, Info, TextBar};
use crate::{App, Cell, ReadRenderBuf};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::cmp::min;
use std::time::{Duration, Instant};
use tui::buffer::Buffer;
use tui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::Text;

pub fn run_main_loop(mut app: Box<dyn App>) {
    let Info {
        title,
        frame_rate,
        text_bar,
    } = app.info();

    let help_text = match text_bar {
        TextBar::Enabled { help_text } => help_text,
        TextBar::Disabled => None,
    };

    let mut ui = TerminalUi::new(title, help_text);
    ui.render(app.render_buf());

    let mut previous_update = Instant::now();

    let frame_duration = Duration::from_millis((1000 / frame_rate) as u64);
    loop {
        while let Some(until_next) =
            frame_duration.checked_sub(Instant::now().duration_since(previous_update))
        {
            if let Some(event) = ui.next_event(until_next) {
                match event {
                    InputEvent::Quit => {
                        return;
                    }
                    InputEvent::KeyPressed(char) => {
                        if let Some(event) = app.handle_pressed_key(char) {
                            match event {
                                AppEvent::SetTitle(title) => ui.set_title(title),
                            }
                        }
                        // Terminals generally don't emit release events, so
                        // we have to rely on the builtin "repeated key press" event
                        // instead of signalling the accurate duration of a key press
                        // to the app.
                        app.handle_released_key(char);
                    }
                }
            }
        }

        if let Some(event) = app.run_frame() {
            match event {
                AppEvent::SetTitle(title) => ui.set_title(title),
            }
        }
        previous_update = Instant::now();

        // debug_ui::render(app.render_buf());
        ui.render(app.render_buf());
    }
}

struct TerminalUi {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    title: String,
    text: Option<String>,
}

impl TerminalUi {
    fn new(title: String, text: Option<String>) -> Self {
        setup_panic_handler();
        let terminal = claim_terminal(std::io::stdout());

        Self {
            terminal,
            title,
            text,
        }
    }

    fn render(&mut self, buf: &dyn ReadRenderBuf) {
        self.terminal
            .draw(|frame| {
                let graphics_container = Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded);

                let horizontal_sub_rects = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Length(buf.dimensions().0 as u16 + 2),
                            Constraint::Min(0),
                        ]
                        .as_ref(),
                    )
                    .split(frame.size());

                let mut graphics_container_rect = horizontal_sub_rects[0];

                let header_height = 2;

                graphics_container_rect.width =
                    min(graphics_container_rect.width, buf.dimensions().0 as u16 + 2);
                graphics_container_rect.height = min(
                    graphics_container_rect.height,
                    buf.dimensions().1 as u16 + 2 + header_height,
                );

                let container_sub_rects = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(header_height), Constraint::Min(0)].as_ref())
                    .split(graphics_container_rect.inner(&Margin {
                        vertical: 1,
                        horizontal: 1,
                    }));

                let header = Paragraph::new(&self.title[..])
                    .style(
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    )
                    .alignment(Alignment::Center);
                let header_container = Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default())
                    .border_type(BorderType::Double);
                let header_container_rect = container_sub_rects[0];

                let mut header_rect = header_container_rect;
                header_rect.height = min(header_rect.height, 1);

                let content = BufWidget(buf);
                let content_rect = container_sub_rects[1];

                if let Some(text) = self.text.as_deref() {
                    let mut rect = horizontal_sub_rects[1];

                    let text_container = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded);
                    let text: Text = text.into();

                    rect.height = min(rect.height, text.height() as u16 + 2);
                    rect.width = min(rect.width, text.width() as u16 + 2);

                    let text = Paragraph::new(text).block(text_container);
                    frame.render_widget(text, rect);
                }

                frame.render_widget(graphics_container, graphics_container_rect);
                frame.render_widget(header_container, header_container_rect);
                frame.render_widget(header, header_container_rect);
                frame.render_widget(content, content_rect);
            })
            .unwrap();
    }

    fn next_event(&self, timeout: Duration) -> Option<InputEvent> {
        if crossterm::event::poll(timeout).unwrap() {
            let event = crossterm::event::read().unwrap();
            if let Event::Key(key_event) = event {
                match key_event {
                    KeyEvent {
                        code: KeyCode::Char('q'),
                        ..
                    } => {
                        return Some(InputEvent::Quit);
                    }
                    KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        kind: KeyEventKind::Press,
                        ..
                    } => {
                        return Some(InputEvent::Quit);
                    }
                    KeyEvent {
                        code: KeyCode::Char(ch),
                        kind,
                        ..
                    } => {
                        if kind == KeyEventKind::Press {
                            return Some(InputEvent::KeyPressed(ch));
                        }
                    }
                    _ => {}
                }
            }
        }
        None
    }

    fn set_title(&mut self, title: String) {
        self.title = title;
    }
}

struct BufWidget<'a>(&'a dyn ReadRenderBuf);

impl Widget for BufWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for y in 0..self.0.dimensions().1 {
            for x in 0..self.0.dimensions().0 {
                if (x as u16) < area.width && (y as u16) < area.height {
                    let Cell(char, (r, g, b)) = self.0.get_cell((x as i16, y as i16));
                    let char = [char];
                    let symbol = std::str::from_utf8(&char).unwrap();
                    buf.get_mut(area.x + x as u16, area.y + y as u16)
                        .set_symbol(symbol)
                        .set_fg(tui::style::Color::Rgb(r, g, b));
                }
            }
        }
    }
}

impl Drop for TerminalUi {
    fn drop(&mut self) {
        restore_terminal(&mut self.terminal);
    }
}

enum InputEvent {
    Quit,
    KeyPressed(char),
}

fn claim_terminal(stdout: Stdout) -> Terminal<CrosstermBackend<Stdout>> {
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout)).unwrap();
    crossterm::terminal::enable_raw_mode().unwrap();
    crossterm::execute!(terminal.backend_mut(), EnterAlternateScreen).unwrap();
    terminal.hide_cursor().unwrap();
    terminal
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) {
    crossterm::terminal::disable_raw_mode().unwrap();
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen,).unwrap();
    terminal.show_cursor().unwrap();
}

fn setup_panic_handler() {
    std::panic::set_hook(Box::new(move |panic_info| {
        let mut stdout = std::io::stdout();
        stdout.flush().unwrap();
        crossterm::execute!(stdout, crossterm::terminal::Clear(ClearType::All)).unwrap();
        crossterm::execute!(stdout, LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();

        println!("Panic backtrace: >{:?}<", Backtrace::new());
        println!("Panic: >{:?}<", panic_info);
    }));
}
