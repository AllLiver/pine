# pine
[![Rust](https://github.com/AllLiver/pico/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/AllLiver/pico/actions/workflows/rust.yml)  
A simple CLI text editor inspired by GNU Nano

# Features
 - Idiomatic error handling using anyhow  
 - Intuitive CLI interface using clap  
 - Lightweight  
 - Easy keybinds  
 - Cross-platform terminal manupulation using crossterm  
 - Fast file reading/writing with Rust's powerful standard library  

# Installation
Verify you have rust, gcc or other basic c build tools, and git installed  
Then run this command:
```
git clone https://github.com/allliver/pine && cd pine && cargo install --path .
```
This command will provide documentation on all of the program's arguments
```
pine -h
```

# Class Diagram

![Pine UML](https://github.com/AllLiver/pine/blob/ec8910223ad1ee7547b49a698ceb223b77606e5a/img/pico-uml.svg "Pine UML")
