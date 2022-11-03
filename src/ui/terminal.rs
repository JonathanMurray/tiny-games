use backtrace::Backtrace;
use crossterm::terminal::{ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use std::io::{Stdout, Write};
use tui::backend::CrosstermBackend;
use tui::widgets::{Block, BorderType, Borders, Paragraph, Widget};
use tui::Terminal;

use crate::{App, Cell, Graphics, GraphicsBuf, PanelItem};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::cmp::{max, min};
use std::time::{Duration, Instant};
use tui::buffer::Buffer;
use tui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::Text;

pub fn run_main_loop(mut app: Box<dyn App>, frame_rate: u32, cell_width: u16) {
    let mut ui = TerminalUi::new(cell_width);
    ui.render(app.graphics());

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
                        app.handle_pressed_key(char);
                        // Terminals generally don't emit release events, so
                        // we have to rely on the builtin "repeated key press" event
                        // instead of signalling the accurate duration of a key press
                        // to the app.
                        app.handle_released_key(char);
                    }
                }
            }
        }

        app.run_frame();
        previous_update = Instant::now();

        ui.render(app.graphics());
    }
}

struct TerminalUi {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    cell_width: u16,
}

impl TerminalUi {
    fn new(cell_width: u16) -> Self {
        setup_panic_handler();
        let terminal = claim_terminal(std::io::stdout());

        Self {
            terminal,
            cell_width,
        }
    }

    fn render(&mut self, graphics: &Graphics) {
        self.terminal
            .draw(|frame| {
                let graphics_container = Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded);

                let graphics_width = graphics.buf.dimensions().0 as u16 * self.cell_width + 2;
                let horizontal_sub_rects = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(graphics_width), Constraint::Min(0)].as_ref())
                    .split(frame.size());

                let mut graphics_container_rect = horizontal_sub_rects[0];

                let header_height = 2;

                graphics_container_rect.width = min(graphics_container_rect.width, graphics_width);
                graphics_container_rect.height = min(
                    graphics_container_rect.height,
                    graphics.buf.dimensions().1 as u16 + 2 + header_height,
                );

                let container_sub_rects = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(header_height), Constraint::Min(0)].as_ref())
                    .split(graphics_container_rect.inner(&Margin {
                        vertical: 1,
                        horizontal: 1,
                    }));

                let header = Paragraph::new(&graphics.title[..])
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

                let content = BufWidget {
                    app_buf: &graphics.buf,
                    cell_width: self.cell_width,
                };
                let content_rect = container_sub_rects[1];

                if let Some(panel) = graphics.side_panel() {
                    let side_panel_rect = horizontal_sub_rects[1];

                    let mut constraints: Vec<Constraint> = vec![];
                    let mut widgets: Vec<(PanelItemWidget, (u16, u16))> = vec![];
                    let mut max_width = 0;

                    for item in &panel.items {
                        match item {
                            PanelItem::TextItem { text } => {
                                let text: Text = text[..].into();
                                let text_container = Block::default()
                                    .borders(Borders::TOP)
                                    .border_type(BorderType::Double);
                                constraints.push(Constraint::Length(text.height() as u16 + 1));
                                let size = (text.width() as u16, text.height() as u16 + 1);
                                max_width = max(max_width, size.0);
                                let paragraph = Paragraph::new(text).block(text_container);
                                widgets.push((PanelItemWidget::Paragraph(paragraph), size));
                            }
                            PanelItem::GraphicsItem { buf } => {
                                constraints.push(Constraint::Length(buf.dimensions().1 as u16));
                                let buf_widget = BufWidget {
                                    app_buf: buf,
                                    cell_width: self.cell_width,
                                };
                                widgets.push((
                                    PanelItemWidget::Buf(buf_widget),
                                    (buf.dimensions().0 as u16, buf.dimensions().1 as u16),
                                ))
                            }
                        }
                    }

                    let panel_item_rects = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(constraints)
                        .split(side_panel_rect);

                    for (i, (widget, size)) in widgets.into_iter().enumerate() {
                        let mut rect = panel_item_rects[i];
                        rect.width = min(rect.width, max_width);
                        rect.height = min(rect.height, size.1);

                        match widget {
                            PanelItemWidget::Buf(widget) => {
                                frame.render_widget(widget, rect);
                            }
                            PanelItemWidget::Paragraph(widget) => {
                                frame.render_widget(widget, rect);
                            }
                        }
                    }
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
}

// This needs to be an enum because Frame::render_widget doesn't
// accept trait objects.
enum PanelItemWidget<'a> {
    Buf(BufWidget<'a>),
    Paragraph(Paragraph<'a>),
}

struct BufWidget<'a> {
    app_buf: &'a GraphicsBuf,
    cell_width: u16,
}

impl Widget for BufWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for y in 0..self.app_buf.dimensions().1 as u16 {
            for app_x in 0..self.app_buf.dimensions().0 as u16 {
                let x = app_x * self.cell_width;
                if x < area.width && y < area.height {
                    let app_cell = self.app_buf.get((app_x as i16, y as i16)).unwrap();
                    match app_cell {
                        Cell::Blank => {}
                        Cell::Colored((r, g, b)) => {
                            for sub_cell_x in x..x + self.cell_width {
                                buf.get_mut(area.x + sub_cell_x, area.y + y)
                                    .set_bg(Color::Rgb(r, g, b));
                            }
                        }
                    }
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
