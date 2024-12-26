pub mod bencode {
    use std::collections::HashMap;

    #[derive(Debug, PartialEq)]
    pub enum Bencode {
        Integer(i64),
        String(Vec<u8>),
        List(Vec<Bencode>),
        Dictionary(HashMap<Vec<u8>, Bencode>),
    }

    pub fn decode(input: &[u8]) -> Result<(Bencode, &[u8]), String> {
        match input.first() {
            Some(b'i') => decode_integer(input),
            Some(b'0'..=b'9') => decode_string(input),
            Some(b'l') => decode_list(input),
            Some(b'd') => decode_dictionary(input),
            _ => Err("invalid input".to_string()),
        }
    }

    fn decode_integer(input: &[u8]) -> Result<(Bencode, &[u8]), String> {
        let end_position = input
            .iter()
            .position(|&x| x == b'e')
            .ok_or_else(|| "no end marker found")?;

        let num = std::str::from_utf8(&input[1..end_position])
            .map_err(|_| "invalid utf8 sequence".to_string())?
            .parse::<i64>()
            .map_err(|_| "invalid number".to_string())?;

        Ok((Bencode::Integer(num), &input[end_position + 1..]))
    }

    fn decode_string(input: &[u8]) -> Result<(Bencode, &[u8]), String> {
        let end_position = input
            .iter()
            .position(|&x| x == b':')
            .ok_or_else(|| "no string delimiter found")?;

        let length = std::str::from_utf8(&input[..end_position])
            .map_err(|_| "invalid utf8 sequence".to_string())?
            .parse::<usize>()
            .map_err(|_| "invalid number".to_string())?;

        let start = end_position + 1;
        let end = end_position + 1 + length;

        Ok((Bencode::String(input[start..end].to_vec()), &input[end..]))
    }

    fn decode_list(input: &[u8]) -> Result<(Bencode, &[u8]), String> {
        let mut list: Vec<Bencode> = Vec::new();
        let mut rest = &input[1..];

        while !rest.is_empty() && rest[0] != b'e' {
            let (value, rest_input) = decode(rest)?;
            list.push(value);
            rest = rest_input;
        }

        Ok((Bencode::List(list), &rest[1..]))
    }

    fn decode_dictionary(input: &[u8]) -> Result<(Bencode, &[u8]), String> {
        let mut dictionary: HashMap<Vec<u8>, Bencode> = HashMap::new();
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
    use crate::bencode::bencode::{decode, Bencode};
    use std::collections::HashMap;

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

        let mut expected_dict = HashMap::new();
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
        let expected = Err(String::from("no end marker found"));

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
        let expected = Err(String::from("no string delimiter found"));

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

        let mut inner_dict = HashMap::new();
        inner_dict.insert(b"boo".to_vec(), Bencode::String(b"bump".to_vec()));
        inner_dict.insert(b"blast".to_vec(), Bencode::Integer(42));

        let mut outer_dict = HashMap::new();
        outer_dict.insert(b"foo".to_vec(), Bencode::String(b"bar".to_vec()));
        outer_dict.insert(b"baz".to_vec(), Bencode::Dictionary(inner_dict));

        let expected = Ok((Bencode::Dictionary(outer_dict), &b""[..]));

        assert_eq!(result, expected);
    }
}
