use crate::bencode::Bencode;
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum TorrentError {
    #[error("invalid dictionary=")]
    InvalidDictionary,
    #[error("invalid list")]
    InvalidList,
    #[error("invalid string")]
    InvalidString,
    #[error("invalid torrent file")]
    InvalidTorrentFile,
    #[error("missing field `{0}`")]
    MissingField(String),
}

#[derive(Debug, PartialEq)]
pub struct TorrentFile {
    pub length: i64,
    pub path: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct TorrentInfo {
    pub name: String,
    pub piece_length: i64,
    pub files: Vec<TorrentFile>,
}

#[derive(Debug, PartialEq)]
pub struct Torrent {
    pub announce: String,
    pub created_by: String,
    pub creation_date: DateTime<Utc>,
    pub info: TorrentInfo,
}

impl Torrent {
    pub fn from_bencode(bencode: &Bencode) -> Result<Self, TorrentError> {
        let root = bencode.as_dict().ok_or(TorrentError::InvalidTorrentFile)?;

        let announce = get_string(root, b"announce")
            .ok_or(TorrentError::MissingField("announce".to_string()))?;
        let created_by = get_string(root, b"created by")
            .ok_or(TorrentError::MissingField("created by".to_string()))?;

        let creation_date = get_integer(root, b"creation date")
            .and_then(|epoch| DateTime::from_timestamp(epoch, 0))
            .ok_or(TorrentError::MissingField("creation date".to_string()))?;

        let info =
            get_dictionary(root, b"info").ok_or(TorrentError::MissingField("info".to_string()))?;
        let info = get_info(info)?;

        Ok(Torrent {
            announce,
            created_by,
            creation_date,
            info,
        })
    }
}

fn get_files(file_list: &Vec<Bencode>) -> Result<Vec<TorrentFile>, TorrentError> {
    file_list
        .into_iter()
        .map(|file| {
            let file_dictionary = file.as_dict().ok_or(TorrentError::InvalidDictionary)?;

            let length = get_integer(file_dictionary, b"length")
                .ok_or(TorrentError::MissingField("length".to_string()))?;

            let path = get_list(file_dictionary, b"path")
                .ok_or(TorrentError::MissingField("path".to_string()))?
                .iter()
                .map(|value| {
                    value
                        .as_string()
                        .map(String::from)
                        .ok_or(TorrentError::InvalidString)
                })
                .collect::<Result<Vec<String>, TorrentError>>()?;

            Ok(TorrentFile { length, path })
        })
        .collect()
}

fn get_info(info_dictionary: &BTreeMap<Vec<u8>, Bencode>) -> Result<TorrentInfo, TorrentError> {
    let name = get_string(info_dictionary, b"name")
        .ok_or(TorrentError::MissingField("name".to_string()))?;

    let piece_length = get_integer(info_dictionary, b"piece length")
        .ok_or(TorrentError::MissingField("piece length".to_string()))?;

    let files = get_list(info_dictionary, b"files")
        .ok_or(TorrentError::MissingField("files".to_string()))
        .and_then(|files| get_files(files))?;

    Ok(TorrentInfo {
        name,
        piece_length,
        files,
    })
}

fn get_dictionary<'a>(
    dictionary: &'a BTreeMap<Vec<u8>, Bencode>,
    key: &'a [u8],
) -> Option<&'a BTreeMap<Vec<u8>, Bencode>> {
    dictionary.get(key).and_then(|value| value.as_dict())
}

fn get_integer(dictionary: &BTreeMap<Vec<u8>, Bencode>, key: &[u8]) -> Option<i64> {
    dictionary.get(key).and_then(|value| value.as_integer())
}

fn get_list<'a>(
    dictionary: &'a BTreeMap<Vec<u8>, Bencode>,
    key: &'a [u8],
) -> Option<&'a Vec<Bencode>> {
    dictionary.get(key).and_then(|value| value.as_list())
}

fn get_string(dictionary: &BTreeMap<Vec<u8>, Bencode>, key: &[u8]) -> Option<String> {
    dictionary
        .get(key)
        .and_then(|value| value.as_string())
        .map(|value| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::{Torrent, TorrentError, TorrentFile, TorrentInfo};
    use crate::bencode::Bencode;
    use chrono::DateTime;
    use std::collections::BTreeMap;

    #[test]
    fn it_converts_bencode_to_torrent() {
        let mut file_info_dict: BTreeMap<Vec<u8>, Bencode> = BTreeMap::new();
        file_info_dict.insert(b"length".to_vec(), Bencode::Integer(1234));

        let mut info_dictionary: BTreeMap<Vec<u8>, Bencode> = BTreeMap::new();
        info_dictionary.insert(b"name".to_vec(), Bencode::String(b"torrent name".to_vec()));
        info_dictionary.insert(b"piece length".to_vec(), Bencode::Integer(123));
        info_dictionary.insert(
            b"files".to_vec(),
            Bencode::List(vec![Bencode::Dictionary(file_info_dict)]),
        );

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
        bencode_dictionary.insert(b"info".to_vec(), Bencode::Dictionary(info_dictionary));

        let bencode_dictionary = Bencode::Dictionary(bencode_dictionary);

        let result = Torrent::from_bencode(&bencode_dictionary).unwrap();
        let expected = Torrent {
            announce: "http://domain/announce".to_string(),
            created_by: "created by me".to_string(),
            creation_date: DateTime::from_timestamp(1735403744163, 0).unwrap(),
            info: TorrentInfo {
                name: "torrent name".to_string(),
                piece_length: 123,
                files: vec![TorrentFile { length: 1234 }],
            },
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn it_fails_if_root_is_not_dictionary() {
        let bencode = Bencode::Integer(123);

        let result = Torrent::from_bencode(&bencode);
        let expected = Err(TorrentError::InvalidTorrentFile);

        assert_eq!(result, expected);
    }

    #[test]
    fn it_fails_if_announce_missing() {
        let mut bencode_dictionary: BTreeMap<Vec<u8>, Bencode> = BTreeMap::new();
        bencode_dictionary.insert(
            b"created by".to_vec(),
            Bencode::String(b"created by me".to_vec()),
        );
        bencode_dictionary.insert(b"creation date".to_vec(), Bencode::Integer(1735403744163));
        let bencode_dictionary = Bencode::Dictionary(bencode_dictionary);

        let result = Torrent::from_bencode(&bencode_dictionary);
        let expected = Err(TorrentError::MissingField("announce".to_string()));

        assert_eq!(result, expected);
    }

    #[test]
    fn it_fails_if_created_by_missing() {
        let mut bencode_dictionary: BTreeMap<Vec<u8>, Bencode> = BTreeMap::new();
        bencode_dictionary.insert(
            b"announce".to_vec(),
            Bencode::String(b"http://domain/announce".to_vec()),
        );
        bencode_dictionary.insert(b"creation date".to_vec(), Bencode::Integer(1735403744163));
        let bencode_dictionary = Bencode::Dictionary(bencode_dictionary);

        let result = Torrent::from_bencode(&bencode_dictionary);
        let expected = Err(TorrentError::MissingField("created by".to_string()));

        assert_eq!(result, expected);
    }

    #[test]
    fn it_fails_if_creation_date_missing() {
        let mut bencode_dictionary: BTreeMap<Vec<u8>, Bencode> = BTreeMap::new();
        bencode_dictionary.insert(
            b"announce".to_vec(),
            Bencode::String(b"http://domain/announce".to_vec()),
        );
        bencode_dictionary.insert(
            b"created by".to_vec(),
            Bencode::String(b"created by me".to_vec()),
        );
        let bencode_dictionary = Bencode::Dictionary(bencode_dictionary);

        let result = Torrent::from_bencode(&bencode_dictionary);
        let expected = Err(TorrentError::MissingField("creation date".to_string()));

        assert_eq!(result, expected);
    }

    #[test]
    fn it_fails_if_info_missing() {
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

        let result = Torrent::from_bencode(&bencode_dictionary);
        let expected = Err(TorrentError::MissingField("info".to_string()));

        assert_eq!(result, expected);
    }

    #[test]
    fn it_fails_if_info_missing_name() {
        let mut info_dictionary: BTreeMap<Vec<u8>, Bencode> = BTreeMap::new();
        info_dictionary.insert(b"piece length".to_vec(), Bencode::Integer(123));

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
        bencode_dictionary.insert(b"info".to_vec(), Bencode::Dictionary(info_dictionary));

        let bencode_dictionary = Bencode::Dictionary(bencode_dictionary);

        let result = Torrent::from_bencode(&bencode_dictionary);
        let expected = Err(TorrentError::MissingField("name".to_string()));

        assert_eq!(result, expected);
    }

    #[test]
    fn it_fails_if_info_missing_piece_length() {
        let mut info_dictionary: BTreeMap<Vec<u8>, Bencode> = BTreeMap::new();
        info_dictionary.insert(b"name".to_vec(), Bencode::String(b"torrent name".to_vec()));

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
        bencode_dictionary.insert(b"info".to_vec(), Bencode::Dictionary(info_dictionary));

        let bencode_dictionary = Bencode::Dictionary(bencode_dictionary);

        let result = Torrent::from_bencode(&bencode_dictionary);
        let expected = Err(TorrentError::MissingField("piece length".to_string()));

        assert_eq!(result, expected);
    }
}
