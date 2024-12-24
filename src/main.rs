use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum Bencode {
    Integer(i64),
    String(Vec<u8>),
    List(Vec<Bencode>),
    Dictionary(HashMap<Vec<u8>, Bencode>),
}

fn decode_integer(input: &[u8]) -> Bencode {
    let end_position = input.iter().position(|&x| x == b'e').unwrap();
    let num = std::str::from_utf8(&input[..end_position])
        .unwrap()
        .parse::<i64>()
        .unwrap();

    Bencode::Integer(num)
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use crate::{decode_integer, Bencode};

    #[test]
    fn it_decodes_integer() {
        let result = decode_integer(b"123e");
        assert_eq!(result, Bencode::Integer(123));
    }
}