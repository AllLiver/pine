use clap::Parser;
use std::io::{stdout, Read};
use crossterm::{terminal, ExecutableCommand};
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

fn main() {
    stdout().execute(crossterm::terminal::SetTitle("pico")).unwrap();

    let args = Args::parse();
    let term = Terminal::new(crossterm::terminal::size().unwrap());
    term.clear();

    let file = File::open(args.file).expect("Could not open file");
    let mut buf_reader = BufReader::new(file);
    let mut buf = String::new();
    buf_reader.read_to_string(&mut buf).expect("Could not read file");

    println!("{}, {:?}", buf, term.size);
}

#[derive(Debug)]
struct Terminal {
    size: (u16, u16),
}

impl Terminal {
    fn new(size: (u16, u16)) -> Terminal {
        Terminal {
            size: size
        }
    }

    fn clear(&self) {
        stdout().execute(terminal::Clear(terminal::ClearType::All)).expect("Could not clear terminal");
    }
}

