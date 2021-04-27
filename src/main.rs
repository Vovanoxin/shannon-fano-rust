use crate::encode::{encode_file, encode_interactive};
use crate::decode::{decode_file, decode_interactive};
use clap::{Arg, App, ArgGroup};
use std::path::Path;
use std::fs::OpenOptions;
use std::io::Write;
use crate::utilities::{CodeTree, Node, BitVector};

mod utilities;
mod encode;
mod decode;


fn main() {
    let matches = App::new("Shannon-Fano encoding/decoding")
        .version("0.0.1")
        .author("Volodymyr Tesliuk <vovatesluk6@gmail.com>")
        .arg(Arg::new("input")
            .short('i')
            .long("input")
            .takes_value(true)
            .about("input file"))
        .arg(Arg::new("output")
            .short('o')
            .long("output")
            .takes_value(true)
            .about("output file"))
        .arg(Arg::new("file")
            .long("file")
            .takes_value(false)
            .requires_all(&["input", "output"])
            .about("use this flag to use file mod of program"))
        .arg(Arg::new("encode")
            .short('e')
            .long("encode")
            .takes_value(false))
        .arg(Arg::new("decode")
            .short('d')
            .long("decode")
            .takes_value(false))
        .group(ArgGroup::new("mode")
            .args(&["encode", "decode"])
            .required(true))
        .get_matches();


    if matches.is_present("file") {
        if matches.is_present("encode") {
            println!("Encoding file");
            encode_file(Path::new(matches.value_of("input").unwrap()),
                        Path::new(matches.value_of("output").unwrap()));
        } else {
            decode_file(Path::new(matches.value_of("input").unwrap()),
                        Path::new(matches.value_of("output").unwrap()))
        }
    } else {
        if matches.is_present("encode") {
            encode_interactive();
        } else {
            decode_interactive();
        }
    }

    // let mut bit_vec1 = BitVector::with_capacity(10);
    // let mut bit_vec2 = BitVector::with_capacity(10);
    // bit_vec1.add_zero();
    // println!("{:?}",bit_vec1);
    // bit_vec1.append_byte(255u8);
    // println!("{:?}",bit_vec1);
    // bit_vec1.append_byte(255u8);
    // println!("{:?}",bit_vec1);
    //
    // bit_vec2.append(&bit_vec1);

}