use std::vec::Vec;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Encoding {
    pub(crate) byte: u8,
    pub(crate) code: Option<Code>,
    pub(crate) prob: f32,
    pub(crate) count: u32,
}

#[derive(Debug, Clone)]
pub struct Code {
    pub num: BitVector,
}

#[derive(Debug, Clone)]
pub struct BitVector {
    bytes: Vec<u8>,
    bit_size: usize,
}

impl BitVector {
    pub fn add_zero(&mut self) {
        if self.bytes.len() < self.bit_size / 8 + 1 {
            self.bytes.push(0u8);
        }
        self.bit_size += 1;
    }
    pub fn add_one(&mut self) {
        if self.bytes.len() < self.bit_size / 8 + 1{
            self.bytes.push(0u8);
        }
        self.bytes[self.bit_size / 8] = self.bytes[self.bit_size / 8] | (128u8 >> (self.bit_size % 8));
        self.bit_size += 1;
    }

    pub fn get_bit(&self, index: usize) -> Option<bool> {
        let result = if index < self.bit_size {
            self.bytes[index / 8] & (128u8 >> (index % 8)) != 0
        } else {
            return None;
        };
        return Some(result);
    }

    //capacity in BYTES required
    pub fn with_capacity(capacity: usize) -> BitVector {
        BitVector { bytes: Vec::with_capacity(capacity), bit_size: 0 }
    }

    //appends byte to the bitvector
    pub fn append_byte(&mut self, byte: u8) {
        if self.bytes.len() < (self.bit_size+8) / 8 + 1 {
            self.bytes.push(0u8);
        }
        self.bytes[BitVector::get_last_byte_index(self.bit_size)] = self.bytes[BitVector::get_last_byte_index(self.bit_size)] | (byte >> (self.bit_size%8));
        if self.bit_size%8 != 0 {
            self.bytes[BitVector::get_last_byte_index(self.bit_size) + 1] = self.bytes[BitVector::get_last_byte_index(self.bit_size) + 1] | (byte << ((8 - self.bit_size%8)%8));
        }
        self.bit_size += 8;
     }

    pub fn append(&mut self, another: &BitVector) {
        for i in 0..another.bytes.len() {
            self.append_byte(another.bytes[i]);
            let redundant = 8 - (another.bit_size % 8);
            self.bit_size -= redundant;
        }

    }

    pub fn get_bytes(&self) -> &Vec<u8> {
        return &self.bytes
    }

    pub fn bit_size(&self) -> usize {
        return self.bit_size
    }

    fn get_last_byte_index(bit_size: usize) -> usize {
        if bit_size != 0 {
            return (bit_size - 1) / 8
        }
        return 0;
    }

    pub fn shrink(&mut self) {
        if  BitVector::get_last_byte_index(self.bit_size) + 1 < self.bytes.len() {
            self.bytes.pop();
        }
    }

}


impl Code {
    pub fn add_zero(&mut self) { self.num.add_zero(); }

    pub fn add_one(&mut self) { self.num.add_one(); }

    pub fn append(&mut self, byte: u8) { self.num.append_byte(byte); }


    pub fn new() -> Code {
        Code { num: BitVector::with_capacity(8) }
    }

}

impl<'a> IntoIterator for &'a Code {
    type Item = bool;
    type IntoIter = CodeIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        CodeIterator { code: self, index: 0 }
    }
}

pub struct CodeIterator<'a> {
    code: &'a Code,
    index: usize,
}

impl<'a> Iterator for CodeIterator<'a> {
    type Item = bool;
    fn next(&mut self) -> Option<bool> {
        let result = self.code.num.get_bit(self.index);
        self.index += 1;
        return result;
    }
}

pub fn generate_codes(start: usize, end: usize, enc: &mut Vec<Encoding>, code: Code, sum: f32) {
    if end - start == 1 {
        enc[start].code = Option::Some(code);
        return;
    }
    let mut best_diff = f32::INFINITY;
    let mut left_sum = 0f32;
    let mut right_sum = 0f32;
    let mut split_index: usize = 0;
    for i in start..end {
        left_sum += enc[i].prob;
        right_sum = sum - left_sum;

        let diff = (left_sum - right_sum).abs();
        if diff < best_diff {
            best_diff = diff;
        } else {
            left_sum -= enc[i].prob;
            split_index = i;
            break;
        }
    }

    let mut left_code = code.clone();
    left_code.add_zero();

    let mut right_code = code.clone();
    right_code.add_one();

    generate_codes(start, split_index, enc, left_code, left_sum);
    generate_codes(split_index, end, enc, right_code, right_sum);
}

pub struct CodeTree {
    root: Option<Box<Node>>,
}

impl CodeTree {
    //pub fn move_create_if_none()
    pub fn insert(&mut self, code: &Code, value: u8) {
        let mut current_node = self.root.as_mut().unwrap();
        for bit in code {
            if bit {
                if !current_node.right_child.is_some() {
                    current_node.create_right();
                }
                current_node = current_node.right_child.as_mut().unwrap();
            } else {
                if !current_node.left_child.is_some() {
                    current_node.create_left();
                }
                current_node = current_node.left_child.as_mut().unwrap();
            }
        }
        current_node.insert_byte(value);
    }
    pub fn new() -> CodeTree {
        CodeTree { root: Some(Box::new(Node::new())) }
    }

    pub fn to_bitvector(&self) -> BitVector {
        let mut bit_vector = BitVector::with_capacity(10);
        let mut queue: VecDeque<&Box<Node>> = VecDeque::new();
        queue.push_back(self.root.as_ref().unwrap());
        while !queue.is_empty() {
            let node = queue.pop_front().unwrap();
            if node.has_children() {
                queue.push_back(node.left_child.as_ref().unwrap());
                queue.push_back(node.right_child.as_ref().unwrap());
                bit_vector.add_zero();
            } else {
                bit_vector.add_one();
                bit_vector.append_byte(node.byte.unwrap());
            }
        }
        return bit_vector;
    }
}

struct Node {
    pub byte: Option<u8>,
    pub left_child: Option<Box<Node>>,
    pub right_child: Option<Box<Node>>,
}

impl Node {
    pub fn new() -> Node {
        Node { byte: None, left_child: None, right_child: None }
    }
    pub fn has_children(&self) -> bool {
        self.left_child.is_some() || self.right_child.is_some()
    }
    pub fn create_right(&mut self) {
        self.right_child = Some(Box::new(Node::new()))
    }
    pub fn create_left(&mut self) {
        self.left_child = Some(Box::new(Node::new()))
    }
    pub fn insert_byte(&mut self, value: u8) {
        self.byte = Some(value);
    }
}