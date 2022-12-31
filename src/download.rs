use crate::util::*;
use anyhow::Result;
use regex::Regex;
use std::fs::{read_to_string, remove_file, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

pub fn list_downloads(filename: &Path) -> Result<()> {
    for line in read_to_string(filename)?.lines() {
        println!("{line}");
    }
    Ok(())
}

pub fn progress(eta: &str, pct: &str) -> String {
    let pct_int: i32 = pct.split('.').next().unwrap().parse().unwrap();
    let n_spaces = 10;
    let n_full = (pct_int as usize) / n_spaces;
    let med = if n_full < n_spaces { 1 } else { 0 };
    let light = if n_full < (n_spaces - 1) {
        n_spaces - med - n_full
    } else {
        0
    };
    let bar = "=".repeat(n_full) + &">".repeat(med) + &"-".repeat(light);
    format!("[{bar}] (ETA {eta})")
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

    let rx_eta: Regex = Regex::new(r#" +(\d*\d*\d\.[0-9]+%).*ETA (.*)"#)?;

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
            let prog = progress(eta, pct);
            print!("\r{blanks}\r{short_title}...: {prog}        ");
            std::io::stdout().flush().expect("Couldn't flush output");
        }
    }
    println!();
    Ok(())
}

fn next_url_from_file(filename: &Path) -> Result<Option<String>> {
    let f = File::open(filename)?;
    let buf = std::io::BufReader::new(f);
    if let Some(Ok(line)) = buf.lines().next() {
        Ok(Some(line))
    } else {
        Ok(None)
    }
}

pub fn download_all(filename: &PathBuf, outdir: &PathBuf) -> Result<()> {
    loop {
        match next_url_from_file(filename) {
            Ok(Some(next_url)) => {
                download_one(&next_url, outdir)?;
                remove_link_from_file(&next_url, filename)?;
            }
            Ok(None) => break,
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

pub fn download_one(url: &str, outdir: &PathBuf) -> Result<()> {
    let as_str = url.to_string();
    if as_str.contains("youtube") || as_str.contains("youtu.be") {
        if let Err(e) = download_from_youtube(&as_str, outdir) {
            eprintln!("Failed to download `{as_str}`: {e}");
        }
    }
    Ok(())
}

pub fn remove_link_from_file(url: &str, filename: &PathBuf) -> Result<()> {
    let lines = read_to_string(filename)?
        .lines()
        .filter(|&l| l != url)
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join("\n");
    let mut f = File::create(filename)?;
    write!(f, "{lines}")?;
    Ok(())
}

pub fn empty_download_file(filename: &Path) -> Result<()> {
    remove_file(filename)?;
    File::create(filename)?;
    Ok(())
}

pub fn add_url(url: Option<String>, filename: &Path) -> Result<()> {
    let mut f = File::options().append(true).open(filename)?;
    if let Some(url) = url {
        let url = url.split('&').next().unwrap().to_string();
        write!(f, "{url}\n")?;
    } else {
        let url = match url_from_clipboard() {
            Ok(Some(url)) => url,
            _ => read_from_stdin("URL: ")?
                .split('&')
                .next()
                .unwrap()
                .to_string(),
        };
        write!(f, "{url}\n")?;
    }
    Ok(())
}
