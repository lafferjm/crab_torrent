mod bencode;
mod torrent;

use anyhow::{anyhow, Result};
use bencode::decode;
use std::env;
use std::fs;
use torrent::{ToBencode, Torrent};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(anyhow!("Usage: ./bencode <torrent_name>"));
    }

    let torrent_name = &args[1];
    let file_contents = fs::read(torrent_name).expect("Couldn't read torrent file");
    let (decoded_file, _) = decode(&file_contents)?;

    let torrent_file = Torrent::from_bencode(&decoded_file).expect("torrent conversion failed");

    println!(
        "{}",
        String::from_utf8_lossy(torrent_file.info.to_bencode()?.as_slice())
    );
    Ok(())
}
