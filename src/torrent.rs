use crate::bencode::Bencode;
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
pub struct Torrent {
    pub announce: String,
    pub created_by: String,
    pub creation_date: DateTime<Utc>,
}

impl Torrent {
    pub fn from_bencode(bencode: &Bencode) -> Option<Self> {
        let root = bencode.as_dict()?;

        let announce = get_string(root, b"announce")?;
        let created_by = get_string(root, b"created by")?;

        let creation_date = get_integer(root, b"creation date")
            .and_then(|epoch| DateTime::from_timestamp(epoch, 0))?;

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

#[cfg(test)]
mod tests {
    use super::Torrent;
    use crate::bencode::Bencode;
    use chrono::DateTime;
    use std::collections::BTreeMap;

    #[test]
    fn it_converts_bencode_to_torrent() {
        let mut bencode_dictionary: BTreeMap<Vec<u8>, Bencode> = BTreeMap::new();
        bencode_dictionary.insert(
            b"announce".to_vec(),
            Bencode::String(b"http://domain/announce".to_vec()),
        );
        bencode_dictionary.insert(
            b"created by".to_vec(),
            Bencode::String(b"created by me".to_vec()),
        );
        bencode_dictionary.insert(b"creation date".to_vec(), Bencode::Integer(1735403744163));
        let bencode_dictionary = Bencode::Dictionary(bencode_dictionary);

        let result = Torrent::from_bencode(&bencode_dictionary).unwrap();
        let expected = Torrent {
            announce: "http://domain/announce".to_string(),
            created_by: "created by me".to_string(),
            creation_date: DateTime::from_timestamp(1735403744163, 0).unwrap(),
        };

        assert_eq!(result, expected);
    }
}
