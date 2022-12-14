use crate::{App, Cell, GraphicsBuf};
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor};
use std::io::Write;

pub fn run_main_loop(mut app: Box<dyn App>) {
    let mut stdout = std::io::stdout();
    let stdin = std::io::stdin();

    println!("Title: {:?}", app.graphics().title);

    let mut input = String::new();

    loop {
        dump(&app.graphics().buf);
        print!("> ");
        stdout.flush().unwrap();
        input.clear();

        stdin.read_line(&mut input).unwrap();
        if let Some(ch) = input.chars().next() {
            if ch == 'q' {
                println!("Good bye.");
                break;
            }
            app.handle_pressed_key(ch);
        }

        app.run_frame();
    }
}

fn dump(buf: &GraphicsBuf) {
    let mut stdout = std::io::stdout();
    print!("+");
    print!("{}", "-".repeat(buf.dimensions().0 as usize));
    println!("+");
    for y in 0..buf.dimensions().1 {
        print!("|");
        for x in 0..buf.dimensions().0 {
            let cell = buf.get((x as i16, y as i16)).unwrap();
            match cell {
                Cell::Blank => print!(" "),
                Cell::Colored(color) => {
                    crossterm::execute!(
                        stdout,
                        SetBackgroundColor(Color::from(color)),
                        Print(" ".to_string()),
                        ResetColor
                    )
                    .unwrap();
                }
            }
        }
        println!("|");
    }
    print!("+");
    print!("{}", "-".repeat(buf.dimensions().0 as usize));
    println!("+");
}
