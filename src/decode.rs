use std::fs::{File, OpenOptions};
use std::path::Path;
use std::io::{Read, Write};
use crate::utilities::{BitVector, CodeTree};
use std::mem::swap;

pub(crate) fn decode_file(input: &Path, output: &Path) {
    let mut input_file = File::open(input).unwrap();

    //read first two bytes
    let mut tree_size: [u8; 2] = [0u8; 2];
    input_file.read(&mut tree_size);
    let tree_size = ((tree_size[0] as u16) << 8) + tree_size[1] as u16;

    //read third byte
    let mut tree_redundant: [u8; 1] = [0u8; 1];
    input_file.read(&mut tree_redundant);


    let mut tree_buffer = vec![0u8; tree_size as usize];

    input_file.read_exact(&mut tree_buffer);

    let tree_bit_vec = BitVector::from_vec(&tree_buffer, tree_redundant[0] as usize);

    let tree = CodeTree::from_bit_vector(&tree_bit_vec);

    drop(tree_buffer);


    let mut encoded_redundant: [u8; 1] = [0u8; 1];
    input_file.read(&mut encoded_redundant);


    let mut f = OpenOptions::new().write(true).create(true).open(output).unwrap();

    const BUFFER_SIZE: usize = 2048;
    let mut buffer: [u8; BUFFER_SIZE] = [0u8; BUFFER_SIZE];
    let mut new_buffer: [u8; BUFFER_SIZE] = [0u8; BUFFER_SIZE];

    let mut current_node = Some(tree.get_root());

    //flag to finish read loop
    let mut last = false;

    let mut bit_vector: BitVector;

    // while !last {
    //     let mut decoded_buffer: Vec<u8> = Vec::with_capacity(4096);
    //     let bytes = input_file.read(&mut buffer).unwrap();
    //
    //     if bytes < BUFFER_SIZE {
    //         //TODO
    //         //if(input_file.)
    //         bit_vector = BitVector::from_vec(&buffer[0..bytes], encoded_redundant[0] as usize);
    //         last = true;
    //     } else {
    //         bit_vector = BitVector::from_vec(&buffer, encoded_redundant[0] as usize);
    //
    //     }
    //
    //     for bit in &bit_vector {
    //         current_node = Some(CodeTree::move_node(current_node.as_ref().unwrap(), bit));
    //
    //         if let Some(value) = current_node.as_ref().unwrap().byte {
    //             //println!("{}", value);
    //             decoded_buffer.push(value);
    //             current_node = Some(tree.get_root());
    //         }
    //     }
    //
    //     f.write(&decoded_buffer);
    //
    // }

    let mut bytes = input_file.read(&mut buffer).unwrap();

    if bytes == 0 {
        return;
    }

    while !last {
        let mut decoded_buffer: Vec<u8> = Vec::with_capacity(4096);
        let bytes_new = input_file.read(&mut new_buffer).unwrap();
        if bytes_new == 0 {
            bit_vector = BitVector::from_vec(&buffer[0..bytes], encoded_redundant[0] as usize);
            last = true;
        } else {
            bit_vector = BitVector::from_vec(&buffer, 0);
            swap(&mut buffer, &mut new_buffer);
            bytes = bytes_new;
        }

        for bit in &bit_vector {
            current_node = Some(CodeTree::move_node(current_node.as_ref().unwrap(), bit));

            if let Some(value) = current_node.as_ref().unwrap().byte {
                //println!("{}", value);
                decoded_buffer.push(value);
                current_node = Some(tree.get_root());
            }
        }
        f.write(&decoded_buffer);
    }
}

pub fn decode_interactive() {
    //TODO
}
