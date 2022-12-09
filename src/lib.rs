pub(crate) mod p1;
pub(crate) mod p2;
pub(crate) mod term;

use p1::parse_term;
use p2::Pass2;
use term::*;

pub fn parse(b: Vec<u8>) -> Block {
    let v = &mut b.into_iter();
    let pass1 = parse_term(v);
    let mut pass2_parser = Pass2::new();
    let mut v = Vec::new();
    pass2_parser.parse(pass1, &mut v);

    Block(v)
}
