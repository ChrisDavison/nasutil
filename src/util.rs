use std::io::{BufRead, BufReader};
use std::str;

#[derive(Debug)]
pub struct CrLfLines<B> {
    buffer: B,
}

#[derive(Debug)]
pub enum MyError {
    Io(std::io::Error),
    Utf8(std::str::Utf8Error),
}

impl<B> CrLfLines<B> {
    pub fn new(buffer: B) -> Self {
        Self { buffer }
    }
}

impl<B: BufRead> Iterator for CrLfLines<B> {
    type Item = Result<String, MyError>;

    fn next(&mut self) -> Option<Self::Item> {
        let (line, total) = {
            let buffer = match self.buffer.fill_buf() {
                Ok(buffer) => buffer,
                Err(e) => return Some(Err(MyError::Io(e))),
            };
            if buffer.is_empty() {
                return None;
            }
            let consumed = buffer
                .iter()
                .take_while(|c| **c != b'\n' && **c != b'\r')
                .count();
            let total = consumed
                + if consumed < buffer.len() {
                    // we found a delimiter
                    if consumed + 1 < buffer.len() // we look if we found two delimiter
                    && buffer[consumed] == b'\r'
                    && buffer[consumed + 1] == b'\n'
                    {
                        2
                    } else {
                        1
                    }
                } else {
                    0
                };
            let line = match str::from_utf8(&buffer[..consumed]) {
                Ok(line) => line.to_string(),
                Err(e) => return Some(Err(MyError::Utf8(e))),
            };
            (line, total)
        };
        self.buffer.consume(total);

        Some(Ok(line))
    }
}
