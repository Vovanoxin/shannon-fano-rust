use std::vec::Vec;
use std::collections::VecDeque;
use std::ops::{Shr, Shl};

#[derive(Debug)]
pub struct Encoding {
    pub(crate) byte: u8,
    pub(crate) bit_vec: Option<BitVector>,
    pub(crate) prob: f32,
    pub(crate) count: u32,
}


#[derive(Debug, Clone)]
pub struct BitVector {
    bytes: Vec<u8>,
    bit_size: usize,
}

impl BitVector {
    //bits available to write at last byte of bit_vector
    fn left_bits(&self) -> usize {
        return if self.bit_size % 8 == 0 {
            0
        } else {
            8 - (self.bit_size % 8)
        };
    }

    fn last_byte_index(&self) -> usize {
        if self.bit_size == 0 {
            return 0;
        }
        return if self.bit_size % 8 == 0 {
            self.bit_size / 8 - 1
        } else {
            self.bit_size / 8
        }
    }

    pub fn add_zero(&mut self) {
        if self.bytes.len() < self.bit_size / 8 + 1 {
            self.bytes.push(0u8);
        }
        self.bit_size += 1;
    }
    pub fn add_one(&mut self) {
        if self.bytes.len() < self.bit_size / 8 + 1 {
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
        if self.bytes.len() < (self.bit_size + 8) / 8 + ((self.bit_size + 8) % 8 != 0) as usize {
            self.bytes.push(0u8);
        }
        let last_byte = self.bit_size / 8;
        if self.bit_size % 8 == 0 {
            self.bytes[last_byte] = byte;
        } else {
            self.bytes[last_byte] = self.bytes[last_byte] | byte.shr(self.bit_size % 8);
            self.bytes[last_byte + 1] = self.bytes[last_byte + 1] | byte.shl(self.left_bits());
        }
        self.bit_size += 8;
    }

    pub fn append(&mut self, another: &BitVector) {
        for byte in &another.bytes {
            self.append_byte(*byte);
        }
        self.bit_size -= another.left_bits();
    }

    pub fn get_bytes(&self) -> &[u8] {
        return &self.bytes;
    }

    pub fn get_bytes_but_last(&self) -> &[u8] {
        return &self.bytes[0..self.bytes.len() - 1];
    }

    pub fn bit_size(&self) -> usize {
        return self.bit_size;
    }

    pub fn shrink(&mut self) {
        if self.last_byte_index() + 1 < self.bytes.len() {
            self.bytes.pop();
        }
    }

    pub fn from_vec(vec: &[u8], redundant_bits: usize) -> BitVector {
        let mut bit_vec = BitVector::with_capacity(512);
        bit_vec.bytes = vec.to_vec();
        bit_vec.bit_size = vec.len() * 8 - redundant_bits;
        return bit_vec;
    }

    pub fn from_last_byte(another: &BitVector) -> BitVector {
        let mut bit_vec = BitVector::with_capacity(512);
        bit_vec.append_byte(another.bytes[another.last_byte_index()]);
        bit_vec.bit_size = another.bit_size % 8 + 8 * ((another.bit_size%8 == 0) as usize);
        return bit_vec;
    }

    pub fn get_byte(&self, index: usize) -> u8{
        let left_part = self.bytes[BitVector::byte_index(index)] << index%8;
        let right_part = self.bytes[BitVector::byte_index(index) + 1].checked_shr((8 - index % 8) as u32).unwrap_or(0);
    return left_part | right_part;
    }

    pub fn byte_index(index: usize) -> usize {
         return index/8;

    }
}


impl<'a> IntoIterator for &'a BitVector {
    type Item = bool;
    type IntoIter = BitVectorIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BitVectorIterator { bit_vec: self, index: 0 }
    }
}

pub struct BitVectorIterator<'a> {
    bit_vec: &'a BitVector,
    index: usize,
}

impl<'a> Iterator for BitVectorIterator<'a> {
    type Item = bool;
    fn next(&mut self) -> Option<bool> {
        let result = self.bit_vec.get_bit(self.index);
        self.index += 1;
        return result;
    }
}

pub fn generate_codes(start: usize, end: usize, enc: &mut Vec<Encoding>, bit_vec: BitVector, sum: f32) {
    if end - start == 1 {
        enc[start].bit_vec = Option::Some(bit_vec);
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

    let mut left_code = bit_vec.clone();
    left_code.add_zero();

    let mut right_code = bit_vec.clone();
    right_code.add_one();

    generate_codes(start, split_index, enc, left_code, left_sum);
    generate_codes(split_index, end, enc, right_code, right_sum);
}

pub struct CodeTree {
    pub(crate) root: Option<Box<Node>>,
}

impl CodeTree {
    pub fn insert(&mut self, bit_vec: &BitVector, value: u8) {
        let mut current_node = self.root.as_mut().unwrap();
        for bit in bit_vec {
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

    pub fn from_bit_vector(bit_vector: &BitVector) -> CodeTree {
        let mut tree: CodeTree = CodeTree { root: None };
        let mut i = 0;
        while i < bit_vector.bit_size() {
            if bit_vector.get_bit(i).unwrap() {
                tree.push_value(Some(bit_vector.get_byte(i + 1)));
                i += 8;
            } else {
                tree.push_value(None);
            }
            i += 1;
        }
        return tree;
    }

    pub fn push_value(&mut self, value: Option<u8>) {
        if self.root.is_none() {
            self.root = Some(Box::new(Node::with_value(value)));
            return;
        }
        let mut queue: VecDeque<&mut Box<Node>> = VecDeque::new();
        queue.push_back(self.root.as_mut().unwrap());

        while !queue.is_empty() {
            let mut curent_node = queue.pop_front().unwrap();
            if curent_node.byte.is_none() {
                if curent_node.left_child.is_none() {
                    curent_node.left_child = Some(Box::new(Node::with_value(value)));
                    return;
                }
                if curent_node.right_child.is_none() {
                    curent_node.right_child = Some(Box::new(Node::with_value(value)));
                    return;
                }
                queue.push_back(curent_node.left_child.as_mut().unwrap());
                queue.push_back(curent_node.right_child.as_mut().unwrap());
            } else {
                continue;
            }
        }
    }
    pub fn move_node(node: &Node, bit: bool) -> &Node {
        return if bit {
            node.right_child.as_ref().unwrap()
        } else {
            node.left_child.as_ref().unwrap()
        };
    }
    pub fn get_root(&self) -> &Node {
        return self.root.as_ref().unwrap();
    }
}

#[derive(Clone)]
pub struct Node {
    pub byte: Option<u8>,
    pub left_child: Option<Box<Node>>,
    pub right_child: Option<Box<Node>>,
}

impl Node {
    pub fn new() -> Node {
        Node { byte: None, left_child: None, right_child: None }
    }
    pub fn with_value(value: Option<u8>) -> Node { Node { byte: value, left_child: None, right_child: None } }
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