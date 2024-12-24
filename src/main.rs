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

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use crate::{decode_integer, decode_string, Bencode};

    mod decode_integer_tests {
        use super::*;

        #[test]
        fn it_decodes_integer() {
            let input = b"i123e";
            let result = decode_integer(&input[..]);
            let expected = Ok((Bencode::Integer(123), &b""[..]));

            assert_eq!(result, expected);
        }

        #[test]
        fn it_decodes_integer_and_returns_rest() {
            let input = b"i123ei456e";
            let result = decode_integer(&input[..]);
            let expected = Ok((Bencode::Integer(123), &b"i456e"[..]));

            assert_eq!(result, expected);
        }

        #[test]
        fn it_handles_invalid_integer() {
            let input = b"i12a3e";
            let result = decode_integer(&input[..]);
            assert!(result.is_err());
        }

        #[test]
        fn it_handles_no_end_marker() {
            let input = b"i123";
            let result = decode_integer(&input[..]);
            let expected = Err(String::from("No end marker found"));

            assert_eq!(result, expected);
        }
    }

    mod decode_string_tests {
        use super::*;

        #[test]
        fn it_decodes_string() {
            let input = b"7:bencode";
            let result = decode_string(input);
            let expected = Ok((Bencode::String(b"bencode".to_vec()), &b""[..]));

            assert_eq!(result, expected);
        }
    }
}
