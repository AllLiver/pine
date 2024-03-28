use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{cursor, event, terminal, ExecutableCommand, QueueableCommand};
use crossterm::event::{Event, KeyCode};
use std::char;
use std::fs::File;
use std::io::{stdout, BufReader, Read, Write};

/// Simple CLI text editor
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the file
    #[arg(required = true)]
    file: String,
}

fn main() -> Result<()> {
    // region:  -- Startup

    let mut term = Terminal::new(crossterm::terminal::size().unwrap());
    let args = Args::parse();
    let file = match File::open(&args.file) {
        // Attempt to open file
        Ok(f) => f, // If file opens, return it
        Err(_) => {
            // If it does not, create it and return that
            File::create(&args.file).context("Could not create file")?
        }
    };

    let mut buf_reader = BufReader::new(file); // Create a new BufReader for the file
    let mut buf = String::new();
    buf_reader
        .read_to_string(&mut buf)
        .context("Could not read file buffer")?; // Read to a string

    let mut buf: Vec<Vec<char>> = buf
        .split("\n")
        .map(|x| x.trim().chars().collect())
        .collect(); // Chop up file contents into chars

    // Switch terminal modes
    crossterm::terminal::enable_raw_mode().context("Could not enable raw mode")?;
    stdout()
        .execute(crossterm::terminal::EnterAlternateScreen)
        .context("Could not enter alternate screen")?;
    term.clear();
    term.set_name(&format!("pico || {}", args.file));

    // endregion:   -- Startup

    term.move_cursor(0, 0);
    print!("pico: {}", args.file);

    // Print starting screen
    term.move_cursor(0, 1);
    for _ in 0..term.size.x {
        print!("-");
    }

    // Print info text
    term.move_cursor(0, term.size.y - 2);
    for _ in 0..term.size.x {
        print!("-");
    }

    term.move_cursor(0, term.size.y - 1);
    print!("exit: ctrl + c || ");

    // Print file buffer
    term.move_cursor(0, 2);
    for i in 0..buf.len() {
        for x in buf[i].clone() {
            print!("{}", x);
        }
        term.move_cursor(0, 3 + i as u16);
    }

    // Flush all terminal prints
    term.flush();

    // App loop
    loop {
        match event::read().context("Could not read user input")? {
            Event::Key(e) => { // Inputs for all key events
                match e.modifiers { // Routes for the input modifiers (ctrl in ctrl + c)
                    event::KeyModifiers::CONTROL => { // Routes for ctrl modifiers
                        match e.code { 
                            event::KeyCode::Char(c) => { // Routes for ctrl + char inputs
                                match c {
                                    'c' => { break; },
                                    _ => {}
                                }
                            },
                            _ => {}
                        } 
                    } // Routes for ctrl modifiers
                    _ => {}
                } // Routes for input modifiers
                
                match e.code { // Routes for single keys
                    KeyCode::Up => {
                        if term.pos.y != 2 {
                            term.move_relative(0, -1); 
                        }
                    },
                    KeyCode::Down => {
                        if term.pos.y != term.size.y - 3 {
                            term.move_relative(0, 1);
                        }
                    },
                    KeyCode::Left => { term.move_relative(-1, 0); },
                    KeyCode::Right => { term.move_relative(1, 0) }
                    _ => {}
                } // Routes for single keys
            }, // Inputs for all key events
            _ => {}
        }
    }

    // region:  -- Shutdown

    // Switch back terminal modes
    crossterm::terminal::disable_raw_mode().context("Could not disable raw mode")?;
    stdout()
        .execute(crossterm::terminal::LeaveAlternateScreen)
        .context("Could not leave alternate screen")?;

    Ok(()) // Return Ok if everything executes fine

    // endregion:   -- Shutdown
}

#[derive(Debug)]
struct Size {
    x: u16,
    y: u16,
}

#[derive(Debug)]
struct Pos {
    x: u16,
    y: u16
}

#[derive(Debug)]
struct Terminal {
    size: Size,
    pos: Pos
}

impl Terminal {
    fn new(size: (u16, u16)) -> Terminal {
        Terminal {
            size: Size {
                x: size.0,
                y: size.1,
            },
            pos: Pos {
                x: 0,
                y: 0
            }
        }
    }

    fn clear(&self) {
        stdout()
            .queue(terminal::Clear(terminal::ClearType::All))
            .unwrap();
    }

    fn set_name(&self, name: &str) {
        stdout().queue(crossterm::terminal::SetTitle(name)).unwrap();
    }

    fn move_cursor(&mut self, posx: u16, posy: u16) {
        self.pos.x = posx;
        self.pos.y = posy;
        stdout().execute(cursor::MoveTo(posx, posy)).unwrap();
    }

    fn move_relative(&mut self, relx: i16, rely: i16) {
        let mut posx = self.pos.x as i16;
        let mut posy = self.pos.y as i16;
        if posx + relx >= 0 && posx + relx <= self.size.x as i16 {
            posx += relx;
        }

        if posy + rely >= 0 && posy + rely <= self.size.y as i16 {
            posy += rely;
        }

        self.move_cursor(posx as u16, posy as u16);
    }

    fn flush(&self) {
        stdout().flush().unwrap();
    }
}
