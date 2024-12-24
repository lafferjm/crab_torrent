use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum Bencode {
    Integer(i64),
    String(Vec<u8>),
    List(Vec<Bencode>),
    Dictionary(HashMap<Vec<u8>, Bencode>),
}

fn decode_integer(input: &[u8]) -> Result<(Bencode, &[u8]), String> {
    let end_position = input
        .iter()
        .position(|&x| x == b'e')
        .ok_or_else(|| "No end marker found")?;

    let num = std::str::from_utf8(&input[..end_position])
        .map_err(|_| "invalid utf8 sequence".to_string())?
        .parse::<i64>()
        .map_err(|_| "invalid number".to_string())?;

    Ok((Bencode::Integer(num), &input[end_position + 1..]))
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use crate::{decode_integer, Bencode};

    mod decode_integer_tests {
        use super::*;

        #[test]
        fn it_decodes_integer() {
            let input = b"i123ei456e";
            let result = decode_integer(&input[1..]);
            let expected = Ok((Bencode::Integer(123), &b"i456e"[..]));

            assert_eq!(result, expected);
        }

        fn it_handles_invalid_integer() {
            let input = b"i12a3e";
            let result = decode_integer(&input[1..]);
            assert!(result.is_err());
        }

        fn it_handles_no_end_marker() {
            let input = b"i123";
            let result = decode_integer(&input[1..]);
            let expected = Err(String::from("No end marker found"));

            assert_eq!(result, expected);
        }
    }
}
