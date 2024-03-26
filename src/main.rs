use clap::Parser;
use crossterm::event::{read, Event, KeyCode, KeyEvent};
use std::io::{stdout, Read, Write};
use crossterm::{cursor, terminal, ExecutableCommand, QueueableCommand};
use std::fs::File;
use std::io::BufReader;

/// Simple CLI text editor
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the file 
    #[arg(required = true)]
    file: String,
}

#[tokio::main]
async fn main() {
    stdout().execute(crossterm::cursor::DisableBlinking).unwrap();
    stdout().execute(crossterm::terminal::EnterAlternateScreen).unwrap();
    crossterm::terminal::enable_raw_mode().unwrap();
    let args = Args::parse();
    let term = Terminal::new(crossterm::terminal::size().unwrap());
    term.clear();
    term.set_name("pico");

    let file = match File::open(&args.file) {
        Ok(f) => f,
        Err(_) => {
            File::create(&args.file).unwrap()
        }
    };

    term.set_name(&format!("pico - {}", args.file));

    let mut buf_reader = BufReader::new(file);
    let mut buf = String::new();
    buf_reader.read_to_string(&mut buf).unwrap_or(1);

    term.move_cursor(0, 0);
    println!("pico || {}", args.file);

    term.move_cursor(0, 1);
    for _ in 0..term.size.x {
        print!("-");
    }

    term.move_cursor(0, term.size.y - 2);
    for _ in 0..term.size.x {
        print!("-");
    }

    term.move_cursor(0, term.size.y - 1);
    print!("exit: ctrl + c");

    term.flush();

    loop {
        match read().unwrap() {
            Event::Key(KeyEvent { code, modifiers, .. }) => {
                match code {
                    KeyCode::Up => { stdout().execute(crossterm::cursor::MoveUp(1)).unwrap(); }
                    KeyCode::Down => { stdout().execute(crossterm::cursor::MoveDown(1)).unwrap(); }
                    KeyCode::Left => { stdout().execute(crossterm::cursor::MoveLeft(1)).unwrap(); }
                    KeyCode::Right => { stdout().execute(crossterm::cursor::MoveRight(1)).unwrap(); }
                    KeyCode::Char(c) => { 
                        match c {
                            'c' => {
                                match modifiers {
                                    crossterm::event::KeyModifiers::CONTROL => { break; }
                                    _ => {}
                                }
                            },
                            _ => ()
                        }
                    }
                    _ => {}
                }
            },
            _ => {}
        }
    } 
   
    crossterm::terminal::disable_raw_mode().unwrap();
    stdout().execute(crossterm::terminal::LeaveAlternateScreen).unwrap();
}

#[derive(Debug)]
struct Size {
    x: u16,
    y: u16
}

#[derive(Debug)]
struct Terminal {
    size: Size,
}

impl Terminal {
    fn new(size: (u16, u16)) -> Terminal {
        Terminal {
            size: Size {
                x: size.0,
                y: size.1
            }
        }
    }

    fn clear(&self) {
        stdout().queue(terminal::Clear(terminal::ClearType::All)).expect("Could not clear terminal");
    }

    fn set_name(&self, name: &str) {
        stdout().queue(crossterm::terminal::SetTitle(name)).unwrap();
    }

    fn move_cursor(&self, posx: u16, posy: u16) {
        stdout().queue(cursor::MoveTo(posx, posy)).unwrap();
    }

    fn flush(&self) {
        stdout().flush().unwrap();
    }
}

