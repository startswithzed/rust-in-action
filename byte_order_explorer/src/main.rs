use std::io::Cursor; // to enable a Vec<T> to mock being a file
use byteorder::LittleEndian; 
use byteorder::{ReadBytesExt, WriteBytesExt}; // traits that provide read and write functions

///
/// Simulate writing to file.
fn write_numbers_to_file() -> (u32, i8, f64) {
    let mut w = vec![]; // writer

    let one: u32 = 1;
    let two: i8 = 2;
    let three: f64 = 3.0;

    w.write_u32::<LittleEndian>(one).unwrap(); // write values to disk
    println!("{:?}", &w);

    w.write_i8(two).unwrap(); // single byte types don't take endianness param
    println!("{:?}", &w);

    w.write_f64::<LittleEndian>(three).unwrap();
    println!("{:?}", &w);

    (one, two, three)
}


///
/// Simulate reading bytes from file.
/// 
fn read_numbers_from_file() -> (u32, i8, f64) {
    let mut r = Cursor::new(vec![1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 8, 64]);
    let one_ = r.read_u32::<LittleEndian>().unwrap();
    let two_ = r.read_i8().unwrap();
    let three_ = r.read_f64::<LittleEndian>().unwrap();

    (one_, two_, three_)
}

fn main() {
    let (one, two, three) = write_numbers_to_file();
    let (one_, two_, three_) = read_numbers_from_file();

    assert_eq!(one, one_);
    assert_eq!(two, two_);
    assert_eq!(three, three_);
}
