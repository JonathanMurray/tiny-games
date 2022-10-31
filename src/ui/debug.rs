use crate::{App, Cell, ReadRenderBuf};
use std::io::Write;

pub fn run_main_loop(mut app: Box<dyn App>) {
    let mut stdout = std::io::stdout();
    let stdin = std::io::stdin();

    println!("Info: {:?}", app.info());

    let mut input = String::new();

    loop {
        dump(app.render_buf());
        print!("> ");
        stdout.flush().unwrap();
        input.clear();

        stdin.read_line(&mut input).unwrap();
        if let Some(ch) = input.chars().next() {
            if ch == 'q' {
                println!("Good bye.");
                break;
            }
            if let Some(event) = app.handle_pressed_key(ch) {
                println!("{:?}", event);
            }
        }

        if let Some(event) = app.run_frame() {
            println!("{:?}", event);
        }
    }
}

fn dump(buf: &dyn ReadRenderBuf) {
    print!("+");
    print!("{}", "-".repeat(buf.dimensions().0 as usize));
    println!("+");
    for y in 0..buf.dimensions().1 {
        print!("|");
        for x in 0..buf.dimensions().0 {
            let Cell(char, _color) = buf.get_cell((x as i16, y as i16));
            print!("{}", char as char);
        }
        println!("|");
    }
    print!("+");
    print!("{}", "-".repeat(buf.dimensions().0 as usize));
    println!("+");
}
