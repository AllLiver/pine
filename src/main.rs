use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{cursor, event, terminal, ExecutableCommand, QueueableCommand};
use crossterm::event::{Event, KeyCode};
use std::char;
use std::fs::{File, write};
use std::path::Path;
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

    let mut file_created = false;

    let args = Args::parse();
    let mut term = Terminal::new(crossterm::terminal::size().unwrap(), args.file.clone());
    let file = match File::open(Path::new(&args.file)) {
        // Attempt to open file
        Ok(f) => f, // If file opens, return it
        Err(_) => {
            // If it does not, create it and return that
            file_created = true;
            File::create(Path::new(&args.file)).context("Could not create file")?
        }
    };

    let mut buf: Vec<Vec<char>> = vec![Vec::new()];

    if !file_created {

        let mut buf_str = String::new();

        let mut buf_reader = BufReader::new(file); // Create a new BufReader for the file
        buf_reader
            .read_to_string(&mut buf_str)
            .context("Could not read file buffer")?; // Read to a string

        buf = buf_str
            .split("\n")
            .map(|x| x.chars().collect())
            .collect(); // Chop up file contents into chars

    }

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
    print!("save and exit: ctrl + c || ");

    // Print file buffer
    for i in term.viewing_range.ymin..term.viewing_range.ymax {
        if i < buf.len() as usize {
            for x in term.viewing_range.xmin..term.viewing_range.xmax {
                if x < buf[i].len() {
                    print!("{}", buf[i][x]);
                } else {
                    break;
                }
            }
            term.move_cursor(0, 3 + i as u16);
        } else {
            break;
        }
    }

    term.move_cursor(0, 2);

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
                    KeyCode::Char(c) => {
                        buf[(term.pos.y - 2) as usize].insert((term.pos.x) as usize, c);
                        term.move_relative(1, 0);
                        term.buf_x_pos = term.pos.x;
                        term.redraw_buf(&buf);
                    },
                    KeyCode::Tab => {
                        for _ in 0..4 {
                            buf[(term.pos.y - 2) as usize].insert((term.pos.x) as usize, ' ');
                            term.move_relative(1, 0);
                            term.redraw_buf(&buf);
                        }
                    },
                    KeyCode::Backspace => {
                        if term.pos.x == 0 {
                            if term.pos.y != 2 {
                                let add_to = buf[(term.pos.y - 2) as usize].clone();
                                let move_to_x: u16 = buf[(term.pos.y - 3) as usize].len() as u16;
                                buf[(term.pos.y - 3) as usize].extend(add_to);
                                buf.remove((term.pos.y - 2) as usize);
                                term.move_cursor(move_to_x, term.pos.y - 1);
                                term.buf_x_pos = term.pos.x;
                                term.redraw_buf(&buf);
                            }
                        } else if term.pos.x != 0 {
                            buf[(term.pos.y - 2) as usize].remove((term.pos.x - 1) as usize);
                            term.move_relative(-1, 0);
                            term.buf_x_pos = term.pos.x;
                            term.redraw_buf(&buf);
                        } 
                    }
                    KeyCode::Enter => {
                        buf.insert((term.pos.y - 1) as usize, Vec::new());
                        buf[(term.pos.y - 1) as usize] = buf[(term.pos.y - 2) as usize].split_off(term.pos.x as usize);
                        term.move_cursor(0, term.pos.y + 1);
                        term.buf_x_pos = term.pos.x;
                        term.redraw_buf(&buf);
                    },
                    KeyCode::Up => {
                        if term.pos.y != 2 {  
                            term.move_relative(0, -1);
                            let current_line_size = buf[(term.pos.y - 2) as usize].len();

                            if current_line_size < term.buf_x_pos as usize {
                                term.move_cursor(current_line_size as u16, term.pos.y);
                            } else {
                                term.move_cursor(term.buf_x_pos, term.pos.y);
                            }
                        }
                    },
                    KeyCode::Down => {
                        if term.pos.y != term.size.y - 3 && term.pos.y <= buf.len() as u16 {  
                            term.move_relative(0, 1);
                            let current_line_size = buf[(term.pos.y - 2) as usize].len();

                            if current_line_size < term.buf_x_pos as usize {
                                term.move_cursor(current_line_size as u16, term.pos.y);
                            } else {
                                term.move_cursor(term.buf_x_pos, term.pos.y);
                            }
                        }
                    },
                    KeyCode::Left => {
                        term.move_relative(-1, 0);
                        term.buf_x_pos = term.pos.x;
                    },
                    KeyCode::Right => {
                        if term.pos.x + 1 < (buf[(term.pos.y - 2) as usize].len() + 1) as u16 {
                            term.move_relative(1, 0); 
                            term.buf_x_pos = term.pos.x;
                        }
                    }
                    _ => {}
                } // Routes for single keys
            }, // Inputs for all key events
            Event::Resize(w, h) => {
                term.size.x = w;
                term.size.y = h;
                term.redraw_buf(&buf);
            }
            _ => {}
        }
    }

    // region:  -- Shutdown

    let buf_write = buf.into_iter()
        .map(|x| x.into_iter().collect::<String>())
        .collect::<Vec<String>>()
        .join("\n");

    write(Path::new(&args.file), buf_write.as_bytes()).context("Could not write buffer to file")?;

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
struct ViewingRange {
    xmin: usize,
    xmax: usize,

    ymin: usize,
    ymax: usize,
}

#[derive(Debug)]
struct Terminal {
    size: Size,
    pos: Pos,
    name: String,
    buf_x_pos: u16,
    viewing_range: ViewingRange,
}

impl Terminal {
    fn new(size: (u16, u16), name: String) -> Terminal {
        Terminal {
            size: Size {
                x: size.0,
                y: size.1,
            },
            pos: Pos {
                x: 0,
                y: 0
            },
            name: name,
            buf_x_pos: size.0,
            viewing_range: ViewingRange {
                xmin: 0,
                xmax: size.0 as usize,

                ymin: 0,
                ymax: (size.1 - 3) as usize, 
            },
        }
    }

    fn clear(&self) {
        stdout()
            .queue(terminal::Clear(terminal::ClearType::All))
            .unwrap();
    }

    fn redraw_buf(&mut self, buf: &Vec<Vec<char>>) {
        let start_x = self.pos.x;
        let start_y = self.pos.y;

        self.clear();

        self.move_cursor(0, 0);
        print!("pico: {}", self.name);

        // Print starting screen
        self.move_cursor(0, 1);
        for _ in 0..self.size.x {
            print!("-");
        }

        // Print info text
        self.move_cursor(0, self.size.y - 2);
        for _ in 0..self.size.x {
            print!("-");
        }

        self.move_cursor(0, self.size.y - 1);
        print!("save and exit: ctrl + c || ");

        self.move_cursor(0, 2);
        for i in self.viewing_range.ymin..self.viewing_range.ymax {
            if i < buf.len() as usize {
                for x in self.viewing_range.xmin..self.viewing_range.xmax {
                    if x < buf[i].len() {
                        print!("{}", buf[i][x]);
                    } else {
                        break;
                    }
                }
                self.move_cursor(0, 3 + i as u16);
            } else {
                break;
            }
        }

        self.move_cursor(start_x, start_y);

        self.flush();
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
        } else {
            if relx < 0 && self.viewing_range.xmin > 0 {
                self.viewing_range.xmin -= 1;
                self.viewing_range.xmax -= 1;
            } else if relx > 0 {
                self.viewing_range.xmin += 1;
                self.viewing_range.xmax += 1;
            }
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
