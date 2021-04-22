use crate::encode::{encode_file, encode_interactive};
use clap::{Arg, App};
use std::path::Path;

mod utilities;
mod encode;


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
            .requires_all(&["input", "output"]))
        .get_matches();

    if !matches.is_present("file"){
        encode_file(Path::new(matches.value_of("input").unwrap()),
                    Path::new(matches.value_of("output").unwrap()));
    } else {
        encode_interactive();
    }
}