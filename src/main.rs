use clap::Parser;
use std::io::{stdout, Read, Write};
use crossterm::{cursor, terminal, QueueableCommand};
use std::fs::File;
use std::io::BufReader;
use anyhow::{Context, Result};

/// Simple CLI text editor
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the file 
    #[arg(required = true)]
    file: String,
}

fn main() -> Result<()> {
    let term = Terminal::new(crossterm::terminal::size().unwrap());
    let args = Args::parse();
    let file = match File::open(&args.file) { // Attempt to open file
        Ok(f) => f, // If file opens, return it
        Err(_) => { // If it does not, create it and return that
            File::create(&args.file).context("Could not create file")?
        }
    };

    let mut buf_reader = BufReader::new(file); // Create a new BufReader for the file
    let mut buf = String::new();
    buf_reader.read_to_string(&mut buf).context("Could not read file buffer")?; // Read to a string

    let mut buf: Vec<Vec<char>> = buf.split("\n").map(|x| x.chars().collect()).collect();

    Ok(()) // Return Ok if everything executes fine
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

