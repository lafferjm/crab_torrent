use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_bencode::de;
use serde_bytes::ByteBuf;
use sha1::{Digest, Sha1};

#[derive(Debug, Deserialize)]
pub struct Torrent {
    pub announce: String,
    #[serde(rename = "created by")]
    pub created_by: String,
    #[serde(rename = "creation date")]
    pub creation_date: i64,
    pub info: TorrentInfo,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TorrentInfo {
    pub name: String,
    #[serde(rename = "piece length")]
    pub piece_length: i64,
    pub files: Vec<TorrentFile>,
    pub pieces: ByteBuf,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TorrentFile {
    pub length: i64,
    pub path: Vec<String>,
}

impl Torrent {
    pub fn new(torrent_contents: Vec<u8>) -> Result<Self> {
        let torrent: Torrent = de::from_bytes(&torrent_contents)?;
        Ok(torrent)
    }

    pub fn info_hash(&self) -> [u8; 20] {
        let info_bytes = serde_bencode::to_bytes(&self.info).expect("info serialization failed");

        let result = Sha1::digest(&info_bytes);

        result.into()
    }
}
