use anyhow::{Context, Result};
use clap::Parser;
use crossterm::event::{Event, KeyCode};
use crossterm::{cursor, event, terminal, ExecutableCommand, QueueableCommand};
use std::char;
use std::fs::{write, File};
use std::io::{stdout, BufReader, Read, Write};
use std::path::Path;

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

        buf = buf_str.split("\n").map(|x| x.chars().collect()).collect(); // Chop up file contents into chars
    }

    // Switch terminal modes
    crossterm::terminal::enable_raw_mode().context("Could not enable raw mode")?;
    stdout()
        .execute(crossterm::terminal::EnterAlternateScreen)
        .context("Could not enter alternate screen")?;
    term.clear();
    term.set_name(&format!("pine || {}", args.file));

    // endregion:   -- Startup

    term.move_cursor(0, 0);
    print!("pine: {}", args.file);

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

    term.move_cursor(term.size.x - 3, term.size.y - 1);
    print!("1:1");

    // Print file buffer
    term.move_cursor(0, 2);
    for i in term.viewing_range.ymin..term.viewing_range.ymax {
        if i < buf.len() {
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
            Event::Key(e) => {
                // Inputs for all key events
                match e.modifiers {
                    // Routes for the input modifiers (ctrl in ctrl + c)
                    event::KeyModifiers::CONTROL => {
                        // Routes for ctrl modifiers
                        match e.code {
                            event::KeyCode::Char(c) => {
                                // Routes for ctrl + char inputs
                                match c {
                                    'c' => {
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    } // Routes for ctrl modifiers
                    _ => {}
                } // Routes for input modifiers

                match e.code {
                    // Routes for single keys
                    KeyCode::Char(c) => {
                        buf[(term.pos.y - 2 + term.viewing_range.ymin as u16) as usize]
                            .insert((term.pos.x + term.viewing_range.xmin as u16) as usize, c);
                        term.move_relative(1, 0);
                        term.buf_x_pos = term.pos.x as usize + term.viewing_range.xmin;
                        term.redraw_buf(&buf);
                    }
                    KeyCode::Tab => {
                        for _ in 0..4 {
                            buf[(term.pos.y - 2 + term.viewing_range.ymin as u16) as usize].insert((term.pos.x + term.viewing_range.xmin as u16) as usize, ' ');
                            term.move_relative(1, 0);
                            term.redraw_buf(&buf);
                        }
                    }
                    KeyCode::Backspace => {
                        if term.pos.x == 0 && term.viewing_range.xmin == 0 {
                            if term.pos.y != 2 {
                                let add_to = buf[(term.pos.y - 2 + term.viewing_range.ymin as u16) as usize].clone();
                                let move_to_x: usize = buf[(term.pos.y - 3 + term.viewing_range.ymin as u16) as usize].len();
                                buf[(term.pos.y - 3 + term.viewing_range.ymin as u16) as usize].extend(add_to);
                                buf.remove((term.pos.y - 2) as usize);
                                term.move_relative((move_to_x - term.pos.x as usize) as i16, -1);
                                term.buf_x_pos = term.pos.x as usize + term.viewing_range.xmin;
                                term.redraw_buf(&buf);
                            }
                        } else if term.pos.x != 0 {
                            buf[(term.pos.y - 2 + term.viewing_range.ymin as u16) as usize]
                                .remove((term.pos.x - 1 + term.viewing_range.xmin as u16) as usize);
                            term.move_relative(-1, 0);
                            term.buf_x_pos = term.pos.x as usize + term.viewing_range.xmin;
                            term.redraw_buf(&buf);
                        }
                    }
                    KeyCode::Enter => {
                        buf.insert((term.pos.y - 1 + term.viewing_range.ymin as u16) as usize, Vec::new());
                        buf[(term.pos.y - 1 + term.viewing_range.ymin as u16) as usize] =
                            buf[(term.pos.y - 2 + term.viewing_range.ymin as u16) as usize].split_off((term.pos.x as usize + term.viewing_range.xmin) as usize);
                        term.move_relative( 0 - term.pos.x as i16, 1);
                        term.viewing_range.xmin = 0;
                        term.viewing_range.xmax = term.size.x as usize;
                        term.buf_x_pos = 0;
                        term.redraw_buf(&buf);
                    }
                    KeyCode::Up => {
                        //if term.pos.y > 1 {
                            term.move_relative((term.buf_x_pos - term.pos.x as usize) as i16, -1);

                            if (term.pos.x as usize + term.viewing_range.xmin) as isize
                                > buf[(term.pos.y - 2 + term.viewing_range.ymin as u16) as usize].len() as isize - 1
                            {
                                term.move_relative(
                                    ((term.pos.x as usize + term.viewing_range.xmin)
                                        - buf[(term.pos.y - 2 + term.viewing_range.ymin as u16) as usize].len())
                                        as i16
                                        * -1,
                                    0,
                                );
                                //println!("{}", (term.pos.x as usize - buf[(term.pos.y - 2) as usize].len()) as i16);
                            }
                            term.redraw_buf(&buf);
                        //}
                    }
                    KeyCode::Down => {
                        if term.pos.y as usize + term.viewing_range.ymin < (buf.len() + 1) as usize {
                            term.move_relative((term.buf_x_pos - term.pos.x as usize) as i16, 1);

                            if (term.pos.x as usize + term.viewing_range.xmin) as isize
                                > buf[(term.pos.y - 2 + term.viewing_range.ymin as u16) as usize].len() as isize - 1
                            {
                                term.move_relative(
                                    ((term.pos.x as usize + term.viewing_range.xmin)
                                        - buf[(term.pos.y - 2 + term.viewing_range.ymin as u16) as usize].len())
                                        as i16
                                        * -1,
                                    0,
                                );
                                //println!("{}", (term.pos.x as usize - buf[(term.pos.y - 2) as usize].len()) as i16);
                            }
                            term.redraw_buf(&buf);
                        }
                    }
                    KeyCode::Left => {
                        term.move_relative(-1, 0);
                        term.buf_x_pos = term.pos.x as usize + term.viewing_range.xmin;
                        term.redraw_buf(&buf);
                    }
                    KeyCode::Right => {
                        if term.pos.x as usize + 1 + term.viewing_range.xmin
                            < buf[(term.pos.y - 2 + term.viewing_range.ymin as u16) as usize].len() + 1
                        {
                            term.move_relative(1, 0);
                            term.buf_x_pos = term.pos.x as usize + term.viewing_range.xmin;
                            term.redraw_buf(&buf);
                        }
                    }
                    _ => {}
                } // Routes for single keys
            } // Inputs for all key events
            Event::Resize(w, h) => {
                term.size.x = w;
                term.size.y = h;
                term.redraw_buf(&buf);
            }
            _ => {}
        }
    }

    // region:  -- Shutdown

    let buf_write = buf
        .into_iter()
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
    y: u16,
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
    buf_x_pos: usize,
    viewing_range: ViewingRange,
}

impl Terminal {
    fn new(size: (u16, u16), name: String) -> Terminal {
        Terminal {
            size: Size {
                x: size.0,
                y: size.1,
            },
            pos: Pos { x: 0, y: 0 },
            name: name,
            buf_x_pos: 0,
            viewing_range: ViewingRange {
                xmin: 0,
                xmax: size.0 as usize,

                ymin: 0,
                ymax: (size.1 - 2) as usize,
            },
        }
    }

    fn clear(&self) {
        stdout()
            .queue(terminal::Clear(terminal::ClearType::All))
            .unwrap();
    }

    fn redraw_buf(&mut self, buf: &Vec<Vec<char>>) {
        if self.pos.x > self.size.x {
            self.viewing_range.xmin += (self.pos.x - self.size.x) as usize;
            self.viewing_range.xmax += (self.pos.x - self.size.x) as usize;
        }

        let start_x = self.pos.x;
        let start_y = self.pos.y;

        self.clear();

        self.move_cursor(0, 0);
        print!("pine: {}", self.name);

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

        //let cursor_indicator_str = format!("{}:{}", start_x as usize + self.viewing_range.xmin + 1, start_y as usize + self.viewing_range.ymin - 1);
        let cursor_indicator_str = format!("{}:{}", self.viewing_range.ymin, self.viewing_range.ymax);
        self.move_cursor((self.size.x as usize - cursor_indicator_str.len()) as u16, self.size.y - 1);
        print!("{cursor_indicator_str}");

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
                self.move_cursor(0, 3 + (i - self.viewing_range.ymin) as u16);
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

    fn move_relative(&mut self, mut relx: i16, rely: i16) {
        let mut posx = self.pos.x as i16;
        let mut posy = self.pos.y as i16;

        if posx + relx < self.size.x as i16 && posx + relx >= 0 {
            posx += relx;
        } else if posx + relx >= self.size.x as i16 {
            relx -= (self.size.x - 1) as i16 - posx;
            posx = (self.size.x - 1) as i16;

            self.viewing_range.xmin += i16::abs(relx) as usize;
            self.viewing_range.xmax += i16::abs(relx) as usize;
        } else if posx + relx < 0 && self.viewing_range.xmin != 0 {
            relx += posx;
            posx = 0;

            self.viewing_range.xmin -= i16::abs(relx) as usize;
            self.viewing_range.xmax -= i16::abs(relx) as usize;
        }

        if posy + rely < (self.size.y - 2) as i16 && posy + rely >= 2 {
            posy += rely;
        } else if posy + rely >= (self.size.y as i16) - 2 {
            self.viewing_range.ymin += 1;
            self.viewing_range.ymax += 1;
        } else if posy + rely < 2 && self.viewing_range.ymin != 0 {
            self.viewing_range.ymin -= 1;
            self.viewing_range.ymax -= 1;
        }

        self.move_cursor(posx as u16, posy as u16);
    }

    fn flush(&self) {
        stdout().flush().unwrap();
    }
}
