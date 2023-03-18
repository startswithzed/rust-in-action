use std::io::prelude::*; // prelude imports heavily used traits such as Read and Write in I/O operations.

const BYTES_PER_LINE: usize = 16;

// multiline string input
// b stands for this should be treated as bytes
// r: raw string
// #: multiline 
const INPUT: &'static [u8] = br#" //   
fn main() {
    println!("Hello, world!");
}"#;

fn main() -> std::io::Result<()> {
    let mut buffer: Vec<u8> =  vec!(); 
    INPUT.read_to_end(&mut buffer)?; // read input and insert it into buffer

    let mut position_in_input = 0;
    
    for line in buffer.chunks(BYTES_PER_LINE) {
        print!("[0x{:08x}] ", position_in_input); // writes the current position with up to 8 left-padded zeros
        for byte in line {
            print!("{:02x} ", byte);
        }
        println!();
        position_in_input += BYTES_PER_LINE;
    }

    Ok(())
}
