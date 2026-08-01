use std::io::BufRead;

pub fn summarize<T: BufRead>(_reader: T, _status: u16) {
    println!("filter")
}