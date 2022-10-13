use anyhow::{anyhow, Result};
use std::io::{stdin, stdout, BufRead, Write};
use std::path::{Path, PathBuf};
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

pub fn output_directory() -> Result<PathBuf> {
    if let Ok(var) = std::env::var("NASUTIL_DIR") {
        Ok(var.into())
    } else {
        Ok(nas_root()
            .ok_or_else(|| anyhow!("Couldn't get nas root"))?
            .join("syncthing"))
    }
}

fn nas_root() -> Option<PathBuf> {
    let options = vec!["/media/nas", "//DAVISON-NAS/918-share", "Y://"];
    for option in options {
        let p = Path::new(option);
        if p.exists() && p.is_dir() {
            return Some(p.into());
        }
    }
    None
}

pub fn download_file() -> PathBuf {
    match std::env::var("NASUTIL_FILE") {
        Ok(filename) => Path::new(&filename).to_path_buf(),
        _ => Path::new(&shellexpand::tilde("~/.nasutil-to-download.txt").to_string()[..])
            .to_path_buf(),
    }
}

pub fn download_file_backup() -> PathBuf {
    match std::env::var("NASUTIL_BACKUP_FILE") {
        Ok(filename) => Path::new(&filename).to_path_buf(),
        _ => {
            let dl_fn = download_file().to_string_lossy().to_string() + ".bak";
            Path::new(&dl_fn).into()
        }
    }
}

pub fn read_from_stdin(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    let _ = stdout().flush();
    let mut response = String::new();
    stdin().read_line(&mut response)?;
    Ok(response.trim().to_string())
}
