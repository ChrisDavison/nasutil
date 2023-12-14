#!/usr/bin/env python3
from argparse import ArgumentParser
from pathlib import Path
import subprocess
import os
import re


def dl_file():
    fn = os.environ["NASUTIL_FILE"]
    return Path(fn).expanduser()


def outdir():
    return os.environ["NASUTIL_DIR"]


def get_current_urls():
    return set(dl_file().read_text().splitlines())


def next_url_from_file():
    return next(thing for thing in get_current_urls())


def add_url(url):
    current = get_current_urls()
    if not url:
        try:
            import pyperclip

            url = pyperclip.paste()
        except ImportError:
            url = input("URL: ")
    url = next(url.split("&"))
    current.add(url)
    dl_file.write_text("\n".join(current))


def remove_from_file(url):
    current = get_current_urls()
    current.remove(url)
    dl_file.write_text("\n".join(current))


def list():
    for line in get_current_urls():
        print(line)


def download():
    while current := next_url_from_file():
        download_url(current)
        remove_from_file(current)
        break


def download_url(url):
    command = [
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
    ]
    proc = subprocess.Popen(command, cwd=outdir(), stdout=subprocess.PIPE)
    rx_title = re.compile(r"[download] Destination: (.*)")
    for line in proc.stdout:
        line = line.decode().strip()
        if m := rx_title.search(line):
            print(m)
            print(">>>", m)
            break


def clear_downloads():
    fn = os.environ["NASUTIL_FILE"]
    Path(fn).unlink()
    Path(fn).touch()


if __name__ == "__main__":
    parser = ArgumentParser()
    cmds = parser.add_subparsers(dest="command")
    cmds.add_parser("list", help="list videos to download", aliases=["ls", "l"])
    add = cmds.add_parser("add", help="add a video", aliases=["a"])
    cmds.add_parser("download", aliases=["d", "dl"])
    cmds.add_parser("version", aliases=["v"])
    cmds.add_parser("empty", aliases=["e", "clear"])
    add.add_argument("video", type=str)

    args = parser.parse_args()

    print(args)

    match args.command:
        case "list" | "l" | "ls":
            list()
        case "add" | "a":
            add_url(args.video)
        case "download" | "d":
            download()
        case "empty" | "e":
            clear_downloads()
        case "version" | "v":
            print("nasutil 0.1.0")
        case x:
            print(f"Unrecognised command {x}")
