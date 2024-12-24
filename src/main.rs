use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum Bencode {
    Integer(i64),
    String(Vec<u8>),
    List(Vec<Bencode>),
    Dictionary(HashMap<Vec<u8>, Bencode>),
}

fn decode_integer(input: &[u8]) -> Result<(Bencode, &[u8]), &'static str> {
    let end_position = input.iter().position(|&x| x == b'e').unwrap();
    let num = std::str::from_utf8(&input[..end_position])
        .unwrap()
        .parse::<i64>()
        .unwrap();

    Ok((Bencode::Integer(num), &input[end_position+1..]))
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use crate::{decode_integer, Bencode};

    #[test]
    fn it_decodes_integer() {
        let input = b"i123ei456e";
        let result = decode_integer(&input[1..]);
        let expected = Ok((Bencode::Integer(123), &b"i456e"[..]));

        assert_eq!(result, expected);
    }
}