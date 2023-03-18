use std::fs::File;
use std::io::prelude::*;
use std::env;

const BYTES_PER_LINE: usize = 16;

fn main() {
    let arg1 = env::args().nth(1);

    let fname = arg1.expect("usage: fview FILENAME");

    let mut f = File::open(&fname).expect("unable to open file.");
    let mut pos = 0;
    let mut buffer = [0; BYTES_PER_LINE];


    // run this loop until f.read_exact returns an error
    // f.read_exact reads data from source and passes it to buffer 
    while let Ok(_) = f.read_exact(&mut buffer) { 
        print!("[0x{:08x}] ", pos); // writes the current position with up to 8 left-padded zeros
        for byte in &buffer {
            // print!("{:02x} ", byte);
            match *byte {
                0x00 => print!(".  "),
                0xff => print!("## "),
                _ => print!("{:02x} ", byte),
            }
        }
        println!();
        pos += BYTES_PER_LINE;
    }
}
