mod torrent;

use anyhow::{anyhow, Result};
use std::env;
use std::fs;
use torrent::Torrent;
use url::Url;
use urlencoding::encode_binary;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(anyhow!("Usage: ./bencode <torrent_name>"));
    }

    let torrent_name = &args[1];
    let file_contents = fs::read(torrent_name).expect("Couldn't read torrent file");
    let torrent = Torrent::new(file_contents)?;

    let client = reqwest::blocking::Client::new();
    let mut url = Url::parse(&torrent.announce)?;

    let info_hash = torrent.info_hash();
    let info_hash_string = encode_binary(&info_hash);
    let sum: i64 = torrent.info.files.iter().map(|b| b.length).sum();

    url.set_query(Some(&format!(
        "info_hash={}&peer_id={}&downloaded={}&uploaded={}&left={}&event={}&port={}",
        info_hash_string, "-PC0001-W6R0LID6jXMs", 0, 0, sum, "started", 6881,
    )));

    let response = client.get(url).send()?;

    println!("{}", response.text()?);

    Ok(())
}
