mod download;
mod util;

use crate::download::*;
use anyhow::*;
use util::download_file;

const VERSION: &str = "0.4.0";
const USAGE: &str = "usage: nasutil CMD

command
    a|add <link>      add a url to the list

    l|list            urls waiting to be downloaded
    d|download        use youtube-dl to download each url
    e|empty           list to be downloaded
    ";

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();

    if args.is_empty() {
        let _ = usage();
        std::process::exit(0);
    }

    let dl_file = download_file();
    let cmd = &args[0];
    if let Err(e) = match cmd.as_ref() {
        "l" | "list" => list_downloads(&dl_file),
        "d" | "download" => download_all(&dl_file),
        "v" | "version" => version(),
        "e" | "empty" => empty_download_file(&dl_file),
        "a" | "add" => add_url(args.get(1).cloned(), &dl_file),
        _ => usage(),
    } {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn usage() -> Result<()> {
    println!("{USAGE}");
    Ok(())
}

fn version() -> Result<()> {
    println!("nasutil {VERSION}");
    Ok(())
}
