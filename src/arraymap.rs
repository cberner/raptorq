use serde::{Deserialize, Serialize};
use std::mem::size_of;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct ArrayMap<T> {
    offset: usize,
    elements: Vec<Option<T>>,
}

impl<T: std::clone::Clone> ArrayMap<T> {
    pub fn new(start_key: usize, end_key: usize) -> ArrayMap<T> {
        ArrayMap {
            offset: start_key,
            elements: vec![None; end_key - start_key],
        }
    }

    pub fn size_in_bytes(&self) -> usize {
        size_of::<Self>() + size_of::<Option<T>>() * self.elements.len()
    }

    pub fn reset<F: Fn(&mut T) -> ()>(
        &mut self,
        new_start: usize,
        new_end: usize,
        reset_value_fn: F,
    ) {
        self.offset = new_start;
        self.elements.resize(new_end - new_start, None);
        for value in self.elements.iter_mut() {
            if let Some(ref mut x) = value {
                reset_value_fn(x);
            }
        }
    }

    pub fn insert(&mut self, key: usize, value: T) {
        self.elements[key - self.offset] = Some(value);
    }

    pub fn get(&self, key: usize) -> Option<&T> {
        self.elements[key - self.offset].as_ref()
    }

    pub fn get_mut(&mut self, key: usize) -> Option<&mut T> {
        self.elements[key - self.offset].as_mut()
    }

    pub fn keys<'s>(&'s self) -> impl DoubleEndedIterator<Item = usize> + 's {
        self.elements
            .iter()
            .enumerate()
            .filter_map(move |(i, elem)| match elem {
                Some(_) => Some(i + self.offset),
                None => None,
            })
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
