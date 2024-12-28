pub mod bencode {
    use std::collections::BTreeMap;
    use std::fmt;
    use thiserror::Error;

    #[derive(Debug, Error, PartialEq)]
    pub enum BencodeError {
        #[error("invalid input")]
        InvalidInput,
        #[error("invalid number")]
        InvalidNumber,
        #[error("invalid utf8 sequence")]
        InvalidSequence,
        #[error("no end marker found")]
        NoEndMarker,
        #[error("no string delimiter found")]
        NoStringDelimiter,
    }

    #[derive(Debug, PartialEq)]
    pub enum Bencode {
        Integer(i64),
        String(Vec<u8>),
        List(Vec<Bencode>),
        Dictionary(BTreeMap<Vec<u8>, Bencode>),
    }

    #[derive(Debug)]
    pub struct Torrent {
        pub announce: String,
        pub created_by: String,
        pub creation_date: i64,
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

    impl Bencode {
        fn as_integer(&self) -> Option<i64> {
            if let Bencode::Integer(i) = self {
                Some(*i)
            } else {
                None
            }
        }

        fn as_string(&self) -> Option<&str> {
            if let Bencode::String(s) = self {
                std::str::from_utf8(s).ok()
            } else {
                None
            }
        }

        fn as_dict(&self) -> Option<&BTreeMap<Vec<u8>, Bencode>> {
            if let Bencode::Dictionary(d) = self {
                Some(d)
            } else {
                None
            }
        }

        pub fn to_torrent(&self) -> Option<Torrent> {
            let root = self.as_dict()?;

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

    impl fmt::Display for Bencode {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Bencode::Integer(i) => write!(f, "{}", i),
                Bencode::String(s) => write!(f, "\"{}\"", String::from_utf8_lossy(s)),
                Bencode::List(list) => {
                    write!(f, "[")?;
                    for (i, item) in list.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", item)?;
                    }
                    write!(f, "]")
                }
                Bencode::Dictionary(dict) => {
                    write!(f, "{{")?;
                    for (i, (key, value)) in dict.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        let key_str = String::from_utf8_lossy(key);
                        write!(f, "\"{}\": {}", key_str, value)?;
                    }
                    write!(f, "}}")
                }
            }
        }
    }

    pub fn decode(input: &[u8]) -> Result<(Bencode, &[u8]), BencodeError> {
        match input.first() {
            Some(b'i') => decode_integer(input),
            Some(b'0'..=b'9') => decode_string(input),
            Some(b'l') => decode_list(input),
            Some(b'd') => decode_dictionary(input),
            _ => Err(BencodeError::InvalidInput),
        }
    }

    fn decode_integer(input: &[u8]) -> Result<(Bencode, &[u8]), BencodeError> {
        let end_position = input
            .iter()
            .position(|&x| x == b'e')
            .ok_or(BencodeError::NoEndMarker)?;

        let num = std::str::from_utf8(&input[1..end_position])
            .map_err(|_| BencodeError::InvalidSequence)?
            .parse::<i64>()
            .map_err(|_| BencodeError::InvalidNumber)?;

        Ok((Bencode::Integer(num), &input[end_position + 1..]))
    }

    fn decode_string(input: &[u8]) -> Result<(Bencode, &[u8]), BencodeError> {
        let end_position = input
            .iter()
            .position(|&x| x == b':')
            .ok_or_else(|| BencodeError::NoStringDelimiter)?;

        let length = std::str::from_utf8(&input[..end_position])
            .map_err(|_| BencodeError::InvalidSequence)?
            .parse::<usize>()
            .map_err(|_| BencodeError::InvalidNumber)?;

        let start = end_position + 1;
        let end = end_position + 1 + length;

        Ok((Bencode::String(input[start..end].to_vec()), &input[end..]))
    }

    fn decode_list(input: &[u8]) -> Result<(Bencode, &[u8]), BencodeError> {
        let mut list: Vec<Bencode> = Vec::new();
        let mut rest = &input[1..];

        while !rest.is_empty() && rest[0] != b'e' {
            let (value, rest_input) = decode(rest)?;
            list.push(value);
            rest = rest_input;
        }

        Ok((Bencode::List(list), &rest[1..]))
    }

    fn decode_dictionary(input: &[u8]) -> Result<(Bencode, &[u8]), BencodeError> {
        let mut dictionary: BTreeMap<Vec<u8>, Bencode> = BTreeMap::new();
        let mut remaining = &input[1..];

        while !remaining.is_empty() && remaining[0] != b'e' {
            let (key, rest) = decode_string(remaining)?;
            let (value, rest) = decode(rest)?;

            if let Bencode::String(key_value) = key {
                dictionary.insert(key_value, value);
            }

            remaining = rest;
        }

        Ok((Bencode::Dictionary(dictionary), &remaining[1..]))
    }
}

#[cfg(test)]
mod tests {
    use crate::bencode::bencode::{decode, Bencode, BencodeError};
    use std::collections::BTreeMap;

    #[test]
    fn it_returns_error_on_invalid_input() {
        let input = b"hello world";
        let result = decode(input);

        assert!(result.is_err());
    }
    #[test]
    fn it_decodes_integers() {
        let input = b"i42e";
        let result = decode(input);
        let expected = Ok((Bencode::Integer(42), &b""[..]));

        assert_eq!(result, expected);
    }

    #[test]
    fn it_decodes_strings() {
        let input = b"7:bencode";
        let result = decode(input);
        let expected = Ok((Bencode::String(b"bencode".to_vec()), &b""[..]));

        assert_eq!(result, expected);
    }

    #[test]
    fn it_decodes_lists() {
        let input = b"li42ee";
        let result = decode(input);
        let expected = Ok((Bencode::List(vec![Bencode::Integer(42)]), &b""[..]));

        assert_eq!(result, expected);
    }

    #[test]
    fn it_decodes_dictionary() {
        let input = b"d4:wiki7:bencode7:meaningi42ee";
        let result = decode(input);

        let mut expected_dict = BTreeMap::new();
        expected_dict.insert(b"wiki".to_vec(), Bencode::String(b"bencode".to_vec()));
        expected_dict.insert(b"meaning".to_vec(), Bencode::Integer(42));

        let expected = Ok((Bencode::Dictionary(expected_dict), &b""[..]));

        assert_eq!(result, expected);
    }

    #[test]
    fn it_decodes_negative_integer() {
        let input = b"i-42e";
        let result = decode(input);
        let expected = Ok((Bencode::Integer(-42), &b""[..]));

        assert_eq!(result, expected);
    }

    #[test]
    fn it_decodes_integer_and_returns_rest() {
        let input = b"i123ei456e";
        let result = decode(input);
        let expected = Ok((Bencode::Integer(123), &b"i456e"[..]));

        assert_eq!(result, expected);
    }

    #[test]
    fn it_handles_invalid_integer() {
        let input = b"i12a3e";
        let result = decode(input);
        assert!(result.is_err());
    }

    #[test]
    fn it_handles_no_end_marker_for_integer() {
        let input = b"i123";
        let result = decode(input);
        let expected = Err(BencodeError::NoEndMarker);

        assert_eq!(result, expected);
    }

    #[test]
    fn it_decodes_string() {
        let input = b"7:bencode";
        let result = decode(input);
        let expected = Ok((Bencode::String(b"bencode".to_vec()), &b""[..]));

        assert_eq!(result, expected);
    }

    #[test]
    fn it_decodes_string_with_two_digit_length() {
        let input = b"10:1234567890";
        let result = decode(input);
        let expected = Ok((Bencode::String(b"1234567890".to_vec()), &b""[..]));

        assert_eq!(result, expected);
    }

    #[test]
    fn it_decodes_string_and_returns_rest() {
        let input = b"7:bencodei42e";
        let result = decode(input);
        let expected = Ok((Bencode::String(b"bencode".to_vec()), &b"i42e"[..]));

        assert_eq!(result, expected);
    }

    #[test]
    fn it_handles_no_length_delimiter() {
        let input = b"7bencode";
        let result = decode(input);
        let expected = Err(BencodeError::NoStringDelimiter);

        assert_eq!(result, expected);
    }

    #[test]
    fn it_handles_invalid_length_value() {
        let input = b"7a:bencode";
        let result = decode(input);

        assert!(result.is_err());
    }

    #[test]
    fn it_decodes_list_with_one_element() {
        let input = b"li42ee";
        let result = decode(input);
        let expected = Ok((Bencode::List(vec![Bencode::Integer(42)]), &b""[..]));

        assert_eq!(result, expected);
    }

    #[test]
    fn it_decodes_list_with_two_elements() {
        let input = b"li42ei-20ee";
        let result = decode(input);
        let result_vector = vec![Bencode::Integer(42), Bencode::Integer(-20)];
        let expected = Ok((Bencode::List(result_vector), &b""[..]));

        assert_eq!(result, expected);
    }

    #[test]
    fn it_decodes_list_with_mixed_elements() {
        let input = b"li42e7:bencodee";
        let result = decode(input);
        let result_vector = vec![Bencode::Integer(42), Bencode::String(b"bencode".to_vec())];
        let expected = Ok((Bencode::List(result_vector), &b""[..]));

        assert_eq!(result, expected);
    }

    #[test]
    fn it_decodes_list_with_list() {
        let input = b"li42eli42eee";
        let result = decode(input);
        let nested_result = Bencode::List(vec![Bencode::Integer(42)]);

        let expected = Ok((
            Bencode::List(vec![Bencode::Integer(42), nested_result]),
            &b""[..],
        ));

        assert_eq!(result, expected);
    }

    #[test]
    fn it_decodes_nested_dictionaries() {
        let input = b"d3:foo3:bar3:bazd3:boo4:bump5:blasti42eee";
        let result = decode(input);

        let mut inner_dict = BTreeMap::new();
        inner_dict.insert(b"boo".to_vec(), Bencode::String(b"bump".to_vec()));
        inner_dict.insert(b"blast".to_vec(), Bencode::Integer(42));

        let mut outer_dict = BTreeMap::new();
        outer_dict.insert(b"foo".to_vec(), Bencode::String(b"bar".to_vec()));
        outer_dict.insert(b"baz".to_vec(), Bencode::Dictionary(inner_dict));

        let expected = Ok((Bencode::Dictionary(outer_dict), &b""[..]));

        assert_eq!(result, expected);
    }
}
