use crate::bencode::Bencode;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Torrent {
    pub announce: String,
    pub created_by: String,
    pub creation_date: i64,
}

impl Torrent {
    pub fn from_bencode(bencode: &Bencode) -> Option<Self> {
        let root = bencode.as_dict()?;

        let announce = get_string(root, b"announce")?;
        let created_by = get_string(root, b"created by")?;

        let creation_date = get_integer(root, b"creation date")?;

        Some(Torrent {
            announce,
            created_by,
            creation_date,
        })
    }
}

fn get_integer(dictionary: &BTreeMap<Vec<u8>, Bencode>, key: &[u8]) -> Option<i64> {
    dictionary.get(key).and_then(|value| value.as_integer())
}

fn get_string(dictionary: &BTreeMap<Vec<u8>, Bencode>, key: &[u8]) -> Option<String> {
    dictionary
        .get(key)
        .and_then(|value| value.as_string())
        .map(|value| value.to_string())
}
