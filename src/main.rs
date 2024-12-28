mod bencode;
mod torrent;

use bencode::decode;
use std::env;
use std::fs;
use torrent::Torrent;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err("Usage: ./bencode <torrent_name>".to_string());
    }

    let torrent_name = &args[1];
    let file_contents = fs::read(torrent_name).expect("Couldn't read torrent file");
    let (decoded_file, _) = decode(&file_contents).unwrap();

    let decoded_file: Torrent = Torrent::from_bencode(&decoded_file)
        .ok_or_else(|| "Could not decode torrent file".to_string())?;

    println!("{:?}", decoded_file);

    Ok(())
}
