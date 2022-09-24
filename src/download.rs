#![allow(unused_macros, dead_code, unused_variables, unused_imports)]
use crate::util::*;
use anyhow::{anyhow, Result};
use regex::Regex;
use std::collections::HashSet;
use std::fs::read_to_string;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
enum Url {
    Valid(String),
    Invalid(String),
}

impl std::fmt::Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Url::Valid(url) => write!(f, "{}", url),
            Url::Invalid(url) => write!(f, "{} FAILED", url),
        }
    }
}

#[derive(Debug)]
pub struct Downloads {
    url_states: HashSet<Url>,
    out_dir: PathBuf,
    in_file: PathBuf,
}

impl Downloads {
    pub fn load_from_file(filepath: &PathBuf) -> Self {
        let url_states: HashSet<Url> = read_to_string(filepath)
            .expect("Failed to read file")
            .lines()
            .map(|f| Url::Valid(f.to_string()))
            .collect();
        Downloads {
            url_states,
            out_dir: crate::util::output_directory().expect("Failed to find output directory"),
            in_file: filepath.to_path_buf(),
        }
    }

    pub fn list_succeeded(&self) -> Result<()> {
        for url in self
            .url_states
            .iter()
            .filter(|url| matches!(url, Url::Valid(_)))
        {
            println!("{}", url);
        }
        Ok(())
    }

    pub fn list_failed(&self) -> Result<()> {
        for url in self
            .url_states
            .iter()
            .filter(|url| matches!(url, Url::Invalid(_)))
        {
            println!("{}", url);
        }
        Ok(())
    }

    pub fn summary(&self) -> Result<()> {
        let (mut n_failed, mut n_to_download) = (0, 0);
        for url in self.url_states.iter() {
            match url {
                Url::Valid(_) => n_to_download += 1,
                Url::Invalid(_) => n_failed += 1,
            }
        }
        println!("{n_to_download} urls to download. {n_failed} previously failed.\n");
        Ok(())
    }

    pub fn add(&mut self, url: Option<impl ToString>) -> Result<()> {
        let rx: Regex = Regex::new(r"\[.*\]\((.+)(&.*)\)")?;
        if let Some(url) = url {
            let tidy_url = match rx.captures(&url.to_string()) {
                Some(caps) => caps[1].split('&').next().unwrap().to_string(),
                None => url.to_string(),
            };
            self.url_states
                .insert(Url::Valid(tidy_url.split('&').next().unwrap().to_string()));
        }
        Ok(())
    }

    pub fn download(&mut self) -> Result<()> {
        for url in self
            .url_states
            .iter()
            .filter(|url| matches!(url, Url::Valid(_)))
        {
            let as_str = url.to_string();
            if as_str.contains("youtube") || as_str.contains("youtu.be") {
                if let Err(e) = download_from_youtube(&as_str, &self.out_dir) {
                    eprintln!("Failed to download `{as_str}`: {e}");
                }
            }
        }
        self.empty()
    }

    pub fn empty(&mut self) -> Result<()> {
        self.url_states.clear();
        Ok(())
    }

    fn write_list_of_urls(urls: &HashSet<Url>) -> Result<()> {
        let mut out = String::new();
        urls.iter().for_each(|url| {
            out.push_str(&*format!("{}\n", url));
        });
        std::fs::write(&*download_file_backup(), out)
            .map_err(|_| anyhow!("Failed to write urls to file"))?;
        std::fs::rename(&*download_file_backup(), &*download_file())
            .map_err(|e| anyhow!("Failed to atomically overwrite backup to file. {e}"))
    }

    pub fn save(&mut self) -> Result<()> {
        Downloads::write_list_of_urls(&self.url_states)
    }
}

fn download_from_youtube(url: &str, out_dir: &PathBuf) -> Result<()> {
    let cmd_reader = duct::cmd!(
        "yt-dlp",
        "-f",
        "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best",
        "--no-playlist",
        "--progress",
        "--merge-output-format",
        "mp4",
        url,
        "-o",
        "%(uploader)s---%(title)s.%(ext)s",
        "--restrict-filenames",
    )
    .dir(out_dir)
    .reader()?;

    let rx_eta: Regex = Regex::new(".*([0-9]+.[0-9]+%).*ETA (.*)")?;

    let buf = BufReader::new(cmd_reader);
    let mut title = String::new();
    for line in CrLfLines::new(buf).flatten() {
        let blanks = " ".repeat(80);
        if line.contains("Destination") {
            title = line
                .trim_start_matches("[download] Destination: ")
                .split('.')
                .next()
                .unwrap_or("NO TITLE?")
                .trim()
                .to_string();
        }
        if line.contains("ETA") {
            let m = rx_eta.captures(&line).unwrap();
            let pct = m.get(1).unwrap().as_str();
            let eta = m.get(2).unwrap().as_str();
            let short_title = &title[..40.min(title.len())];
            print!("\r{blanks}\r{short_title}...: {pct} (ETA {eta})        ");
            std::io::stdout().flush().expect("Couldn't flush output");
        }
    }
    println!();
    Ok(())
}

mod test {
    use super::*;

    fn dummy_downloads() -> Downloads {
        Downloads {
            url_states: [
                Url::Valid("https://www.youtube.com/watch?v=tbnLqRW9Ef0".to_string()),
                Url::Invalid("news.ycombinator.com".to_string()),
                Url::Invalid("www.reddit.com".to_string()),
            ]
            .iter()
            .cloned()
            .collect::<HashSet<Url>>(),
            out_dir: PathBuf::from("."),
            in_file: download_file(),
        }
    }

    #[test]
    fn download_test() {
        let mut dls = dummy_downloads();
        dls.download().expect("Failed to dummy download");
        let filename_out = std::path::PathBuf::from("Chaladz---1_sec_VIDEO.mp4");
        assert!(filename_out.exists());
        std::fs::remove_file(filename_out).expect("Couldn't remove downloaded test file");
    }

    #[test]
    fn add_test() {
        let mut downloads_for_test = dummy_downloads();
        downloads_for_test.add(Some("www.google.com")).unwrap();
        assert!(downloads_for_test
            .url_states
            .contains(&Url::Valid("www.google.com".into())));
    }

    #[test]
    fn empty_test() {
        let mut downloads_for_test = dummy_downloads();
        downloads_for_test.empty().unwrap();
        assert_eq!(downloads_for_test.url_states, HashSet::new());
    }
}
