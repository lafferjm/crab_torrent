mod bencode;
use bencode::bencode::decode;

fn main() {
    let value = decode(b"7:bencode");
    println!("{:?}", value);
}
