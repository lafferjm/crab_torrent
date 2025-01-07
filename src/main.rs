mod bencode;
mod torrent;

use anyhow::{anyhow, Result};
use bencode::decode;
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use std::env;
use std::fs;
use torrent::Torrent;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(anyhow!("Usage: ./bencode <torrent_name>"));
    }

    let torrent_name = &args[1];
    let file_contents = fs::read(torrent_name).expect("Couldn't read torrent file");
    let (decoded_file, _) = decode(&file_contents)?;

    let torrent_file = Torrent::from_bencode(&decoded_file).expect("torrent conversion failed");

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&torrent_file.announce)
        .query(&[
            (
                "info_hash",
                percent_encode(&torrent_file.info_hash()?, NON_ALPHANUMERIC).to_string(),
            ),
            (
                "peer_id",
                percent_encode(b"-TESTING_TORRENT_CLIENT_123ABC01062025", NON_ALPHANUMERIC)
                    .to_string(),
            ),
            ("downloaded", "0".to_string()),
            ("uploaded", "0".to_string()),
            ("left", torrent_file.info.piece_length.to_string()),
            ("event", "started".to_string()),
            ("port", "6881".to_string()),
        ])
        .send()?;

    println!("{}", response.text()?);

    // println!("body = {body:?}");

    Ok(())
}
