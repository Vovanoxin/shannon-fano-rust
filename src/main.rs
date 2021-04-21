use std::fs::{File, OpenOptions};
use std::io::{Read, Write, BufWriter, Seek, SeekFrom};
use std::collections::HashMap;
use crate::utilities::{Encoding, generate_codes, Code, CodeTree, BitVector};

use std::env;

use clap::{Arg, App};

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
        .arg(Arg::new("mode")
            .short('m')
            .long("mode")
            .takes_value(true)
            .possible_value("encode")
            .possible_value("decode")
            .required(true)
        ).get_matches();
    const BUFFER_SIZE: usize = 4096;
    let mut buffer: [u8; BUFFER_SIZE] = [0u8; BUFFER_SIZE];
    let mut input_file = File::open("enwik8").unwrap();
    let mut byte_counter: HashMap<u8, u32> = HashMap::with_capacity(256);
    let mut byte_number: u32 = 0;
    loop {
        let bytes = input_file.read(&mut buffer).unwrap();
        for i in 0..bytes {
            let count = byte_counter.entry(buffer[i]).or_insert(0);
            *count += 1;
            byte_number += 1;
        }
        if bytes != BUFFER_SIZE {
            break;
        }
    }
    let mut encodings: Vec<Encoding> = byte_counter.into_iter()
        .map(|(byte, count)|
            Encoding {
                byte,
                code: Option::None,
                prob: (count as f32) / (byte_number as f32),
                count,
            })
        .collect();
    encodings.sort_by(|a, b| b.count.cmp(&a.count));
    generate_codes(0, encodings.len(), &mut encodings, Code::new(), 1f32);
    let mut code_table = HashMap::new();
    let mut code_tree = CodeTree::new();
    let mut encoded_size: u32 = 0;
    for encoding in encodings.iter() {
        encoded_size += encoding.code.as_ref().unwrap().num.bit_size() as u32 * encoding.count;
        code_table.insert(encoding.byte, encoding.code.as_ref().unwrap().clone());
        code_tree.insert(encoding.code.as_ref().unwrap(), encoding.byte)
    }

    let f = OpenOptions::new().write(true).create(true).open("output").unwrap();
    let mut bw = BufWriter::new(f);
    let code_tree_binary = code_tree.to_bitvector();
    //println!("{:?}", code_tree_binary.get_bytes().len());
    //println!("{:?}", code_tree_binary.bit_size());

    //write size of code tree
    bw.write(&[code_tree_binary.get_bytes().len() as u8]);

    let redundant_bits = (code_tree_binary.get_bytes().len() * 8 - code_tree_binary.bit_size()) as u8;

    //write number of redundant bits
    bw.write(&[redundant_bits]);

    //write code tree
    bw.write(code_tree_binary.get_bytes());

    let redundant_bits: u8 = ((encoded_size / 8 + u32::from(encoded_size % 8 != 0)) * 8 - encoded_size) as u8;
    bw.write(&[redundant_bits]);
    input_file.seek(SeekFrom::Start(0));

    let mut bit_vec = BitVector::with_capacity(10);

    //read from file again
    loop {
        let bytes = input_file.read(&mut buffer).unwrap();
        let mut encoded_buffer = BitVector::with_capacity(512);
        for i in 0..bytes {
            let code = &code_table[&buffer[i]];
            encoded_buffer.append(&code.num);
        }
        encoded_buffer.shrink();
        bw.write(encoded_buffer.get_bytes());
        if bytes != BUFFER_SIZE {
            break;
        }
    }
}