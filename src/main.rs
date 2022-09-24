mod download;
mod util;

use crate::download::Downloads;
use anyhow::*;
use util::download_file;

const VERSION: &str = "0.1.0";
const USAGE: &str = "usage: nasutil CMD

command
    a|add <link>      add a url to the list

    l|list            urls waiting to be downloaded
    d|download        use youtube-dl to download each url
    e|empty           list to be downloaded
    ";

fn main() {
    let mut d = Downloads::load_from_file(&download_file());

    let args: Vec<_> = std::env::args().skip(1).collect();

    if args.is_empty() {
        let _ = usage();
        std::process::exit(0);
    }

    let cmd = &args[0];
    if let Err(e) = match cmd.as_ref() {
        "l" | "list" => d.list_succeeded(),
        "d" | "download" => d.download(),
        "v" | "version" => version(),
        "e" | "empty" => d.empty(),
        "a" | "add" => d.add(args.get(1)),
        _ => usage(),
    } {
        eprintln!("{e}");
        std::process::exit(1);
    }
    if let Err(e) = d.save() {
        eprintln!("Failed to save after `{cmd}`: {e}");
        std::process::exit(2);
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
