use std::io::BufRead;

pub fn summarize<T: BufRead>(_reader: T, _number: usize) {
    println!("top")
}