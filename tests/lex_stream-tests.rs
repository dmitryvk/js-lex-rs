extern crate js_lex_rs;

use js_lex_rs::lex_stream::LexStream;

#[test]
pub fn read() {
    let mut s = LexStream::new("abc".chars());
    let c = s.read();
    assert_eq!(Some('a'), c);
    let c = s.read();
    assert_eq!(Some('b'), c);
    let c = s.read();
    assert_eq!(Some('c'), c);
    let c = s.read();
    assert_eq!(None, c);
}

#[test]
pub fn lookahead_read1() {
    let mut s = LexStream::new("abc".chars());
    let c = s.lookahead(0);
    assert_eq!(Some('a'), c);
    let c = s.lookahead(1);
    assert_eq!(Some('b'), c);
    let c = s.lookahead(2);
    assert_eq!(Some('c'), c);
    let c = s.lookahead(3);
    assert_eq!(None, c);
}

#[test]
pub fn lookahead_read2() {
    let mut s = LexStream::new("abc".chars());
    let c = s.lookahead(2);
    assert_eq!(Some('c'), c);
    let c = s.read();
    assert_eq!(Some('a'), c);
    let c = s.lookahead(0);
    assert_eq!(Some('b'), c);
    let c = s.read();
    assert_eq!(Some('b'), c);
    let c = s.read();
    assert_eq!(Some('c'), c);
    let c = s.read();
    assert_eq!(None, c);
}

#[test]
pub fn skip() {
    let mut s = LexStream::new("abc".chars());
    s.skip(1);
    let c = s.lookahead(0);
    assert_eq!(Some('b'), c);
    s.skip(2);
    let c = s.read();
    assert_eq!(None, c);
}
