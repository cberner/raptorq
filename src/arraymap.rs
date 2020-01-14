use serde::{Deserialize, Serialize};
use std::mem::size_of;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct UndirectedGraph {
    edges: Vec<(u16, u16)>,
}

impl UndirectedGraph {
    pub fn with_capacity(edges: usize) -> UndirectedGraph {
        UndirectedGraph {
            edges: Vec::with_capacity(edges * 2),
        }
    }

    pub fn add_edge(&mut self, node1: u16, node2: u16) {
        self.edges.push((node1, node2));
        self.edges.push((node2, node1));
    }

    pub fn build(&mut self) {
        self.edges.sort_unstable();
    }

    pub fn get_adjacent_nodes(&self, node: u16) -> Vec<u16> {
        let first_candidate = match self.edges.binary_search(&(node as u16, 0)) {
            Ok(index) => index,
            Err(index) => index,
        };
        if first_candidate == self.edges.len() {
            return vec![];
        }
        if self.edges[first_candidate].0 != node as u16 {
            return vec![];
        }

        let mut result = vec![];
        for i in first_candidate..self.edges.len() {
            if self.edges[i].0 == node as u16 {
                result.push(self.edges[i].1);
            } else {
                break;
            }
        }
        return result;
    }

    pub fn nodes(&self) -> Vec<u16> {
        let mut result = vec![];
        for &(node, _) in self.edges.iter() {
            if result.is_empty() || result[result.len() - 1] != node {
                result.push(node);
            }
        }

        result
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct U16ArrayMap {
    offset: usize,
    elements: Vec<u16>,
}

impl U16ArrayMap {
    pub fn new(start_key: usize, end_key: usize) -> U16ArrayMap {
        U16ArrayMap {
            offset: start_key,
            elements: vec![0; end_key - start_key],
        }
    }

    pub fn size_in_bytes(&self) -> usize {
        size_of::<Self>() + size_of::<u16>() * self.elements.len()
    }

    pub fn swap(&mut self, key: usize, other_key: usize) {
        self.elements.swap(key, other_key);
    }

    pub fn insert(&mut self, key: usize, value: u16) {
        self.elements[key - self.offset] = value;
    }

    pub fn get(&self, key: usize) -> u16 {
        self.elements[key - self.offset]
    }

    pub fn decrement(&mut self, key: usize) {
        self.elements[key - self.offset] -= 1;
    }

    #[allow(dead_code)]
    pub fn increment(&mut self, key: usize) {
        self.elements[key - self.offset] += 1;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct U32VecMap {
    offset: usize,
    elements: Vec<u32>,
}

impl U32VecMap {
    pub fn new(start_key: usize) -> U32VecMap {
        U32VecMap {
            offset: start_key,
            elements: vec![0; 1],
        }
    }

    fn grow_if_necessary(&mut self, index: usize) {
        if index >= self.elements.len() {
            self.elements
                .extend(vec![0; index - self.elements.len() + 1]);
        }
    }

    pub fn size_in_bytes(&self) -> usize {
        size_of::<Self>() + size_of::<u32>() * self.elements.len()
    }

    #[allow(dead_code)]
    pub fn swap(&mut self, key: usize, other_key: usize) {
        self.elements.swap(key, other_key);
    }

    #[allow(dead_code)]
    pub fn insert(&mut self, key: usize, value: u32) {
        self.grow_if_necessary(key - self.offset);
        self.elements[key - self.offset] = value;
    }

    pub fn get(&self, key: usize) -> u32 {
        if key - self.offset >= self.elements.len() {
            return 0;
        }
        self.elements[key - self.offset]
    }

    pub fn decrement(&mut self, key: usize) {
        self.grow_if_necessary(key - self.offset);
        self.elements[key - self.offset] -= 1;
    }

    pub fn increment(&mut self, key: usize) {
        self.grow_if_necessary(key - self.offset);
        self.elements[key - self.offset] += 1;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct BoolArrayMap {
    offset: usize,
    elements: Vec<bool>,
}

impl BoolArrayMap {
    pub fn new(start_key: usize, end_key: usize) -> BoolArrayMap {
        BoolArrayMap {
            offset: start_key,
            elements: vec![false; end_key - start_key],
        }
    }

    pub fn insert(&mut self, key: usize, value: bool) {
        self.elements[key - self.offset] = value;
    }

    pub fn get(&self, key: usize) -> bool {
        self.elements[key - self.offset]
    }
}
