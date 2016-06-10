extern crate js_lex_rs;
use js_lex_rs::*;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let filename = std::env::args().nth(1).expect("File name not passed");
    
    let mut f = File::open(&filename).unwrap();
    let sz = f.metadata().unwrap().len() as usize;
    let mut s = String::with_capacity(sz);
    
    f.read_to_string(&mut s).unwrap();

    let tokens = tokenize_str(&s);
    
    for token in tokens {
        println!("{:?}", token);
    }
}
