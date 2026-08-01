use std::io::BufRead;

pub fn summarize<T: BufRead>(_reader: T) {
    println!("status")
}