use crate::octet::Octet;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::mem::size_of;

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Hash)]
struct PackedEntry {
    value: u32,
}

impl PackedEntry {
    fn new(index: u32, binary_value: u8) -> PackedEntry {
        debug_assert!(index < 16777216);
        debug_assert!(binary_value < 2);
        PackedEntry {
            value: index << 8 | (binary_value as u32),
        }
    }

    fn value(&self) -> Octet {
        Octet::new((self.value & 0xFF) as u8)
    }

    fn add_value(&mut self, value: Octet) {
        // Index is stored in upper 24-bits, but XOR'ing with zero won't change it.
        self.value ^= value.byte() as u32
    }

    fn index(&self) -> u32 {
        self.value >> 8
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Hash)]
pub struct SparseBinaryVec {
    // Kept sorted by the usize (key)
    elements: Vec<PackedEntry>,
}

impl SparseBinaryVec {
    pub fn with_capacity(capacity: usize) -> SparseBinaryVec {
        SparseBinaryVec {
            elements: Vec::with_capacity(capacity),
        }
    }

    // Returns the internal index into self.elements matching key i, or the index
    // at which it can be inserted (maintaining sorted order)
    fn key_to_internal_index(&self, i: u32) -> Result<usize, usize> {
        self.elements
            .binary_search_by_key(&i, |entry| entry.index())
    }

    pub fn size_in_bytes(&self) -> usize {
        size_of::<Self>() + size_of::<PackedEntry>() * self.elements.len()
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn get_by_raw_index(&self, i: usize) -> (usize, Octet) {
        (self.elements[i].index() as usize, self.elements[i].value())
    }

    // Returns a vector of new column indices that this row contains
    pub fn add_assign(&mut self, other: &SparseBinaryVec) -> Vec<u32> {
        // Fast path for a single value that's being eliminated
        // TODO: Probably wouldn't need this if we implemented "Furthermore, the row operations
        // required for the HDPC rows may be performed for all such rows in one
        // process, by using the algorithm described in Section 5.3.3.3."
        if other.elements.len() == 1 {
            let other_entry = &other.elements[0];
            match self.key_to_internal_index(other_entry.index()) {
                Ok(index) => {
                    let self_entry = &mut self.elements[index];
                    self_entry.add_value(other_entry.value());
                    if self_entry.value() == Octet::zero() {
                        self.elements.remove(index);
                    }
                }
                Err(index) => {
                    self.elements.insert(index, other_entry.clone());
                    return vec![other_entry.index()];
                }
            };
            return vec![];
        }

        let mut result = Vec::with_capacity(self.elements.len() + other.elements.len());
        let mut self_iter = self.elements.iter();
        let mut other_iter = other.elements.iter();
        let mut self_next = self_iter.next();
        let mut other_next = other_iter.next();

        let mut new_columns = Vec::with_capacity(10);
        loop {
            if let Some(self_entry) = self_next {
                if let Some(other_entry) = other_next {
                    match self_entry.index().cmp(&other_entry.index()) {
                        Ordering::Less => {
                            if self_entry.value() != Octet::zero() {
                                result.push(self_entry.clone());
                            }
                            self_next = self_iter.next();
                        }
                        Ordering::Equal => {
                            let value = self_entry.value() + other_entry.value();
                            if value != Octet::zero() {
                                result.push(PackedEntry::new(self_entry.index(), value.byte()));
                            }
                            self_next = self_iter.next();
                            other_next = other_iter.next();
                        }
                        Ordering::Greater => {
                            if other_entry.value() != Octet::zero() {
                                new_columns.push(other_entry.index());
                                result.push(other_entry.clone());
                            }
                            other_next = other_iter.next();
                        }
                    }
                } else {
                    if self_entry.value() != Octet::zero() {
                        result.push(self_entry.clone());
                    }
                    self_next = self_iter.next();
                }
            } else if let Some(other_entry) = other_next {
                if other_entry.value() != Octet::zero() {
                    new_columns.push(other_entry.index());
                    result.push(other_entry.clone());
                }
                other_next = other_iter.next();
            } else {
                break;
            }
        }
        self.elements = result;

        return new_columns;
    }

    pub fn remove(&mut self, i: usize) -> Option<Octet> {
        match self.key_to_internal_index(i as u32) {
            Ok(index) => Some(self.elements.remove(index).value()),
            Err(_) => None,
        }
    }

    pub fn retain<P: Fn(&(usize, Octet)) -> bool>(&mut self, predicate: P) {
        self.elements
            .retain(|entry| predicate(&(entry.index() as usize, entry.value())));
    }

    pub fn get(&self, i: usize) -> Option<Octet> {
        match self.key_to_internal_index(i as u32) {
            Ok(index) => Some(self.elements[index].value()),
            Err(_) => None,
        }
    }

    pub fn keys_values(&self) -> impl Iterator<Item = (usize, Octet)> + '_ {
        self.elements
            .iter()
            .map(|entry| (entry.index() as usize, entry.value()))
    }

    pub fn insert(&mut self, i: usize, value: Octet) {
        match self.key_to_internal_index(i as u32) {
            Ok(index) => self.elements[index] = PackedEntry::new(i as u32, value.byte()),
            Err(index) => self
                .elements
                .insert(index, PackedEntry::new(i as u32, value.byte())),
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Hash)]
// Sparse vector over 24-bit address space
pub struct SparseValuelessVec {
    // Kept sorted
    elements: Vec<u32>,
}

impl SparseValuelessVec {
    pub fn with_capacity(capacity: usize) -> SparseValuelessVec {
        SparseValuelessVec {
            elements: Vec::with_capacity(capacity),
        }
    }

    // Returns the internal index into self.elements matching key i, or the index
    // at which it can be inserted (maintaining sorted order)
    fn key_to_internal_index(&self, i: u32) -> Result<usize, usize> {
        self.elements.binary_search(&i)
    }

    pub fn size_in_bytes(&self) -> usize {
        size_of::<Self>() + size_of::<u32>() * self.elements.len()
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    #[cfg(debug_assertions)]
    pub fn exists(&self, i: usize) -> bool {
        self.key_to_internal_index(i as u32).is_ok()
    }

    pub fn get_by_raw_index(&self, i: usize) -> usize {
        self.elements[i] as usize
    }

    pub fn keys(&self) -> impl Iterator<Item = usize> + '_ {
        self.elements.iter().map(|x| *x as usize)
    }

    pub fn insert(&mut self, i: usize) {
        // Encoding indices can't exceed 24-bits, so neither can these
        debug_assert!(i < 16777216);
        match self.key_to_internal_index(i as u32) {
            Ok(index) => self.elements[index] = i as u32,
            Err(index) => self.elements.insert(index, i as u32),
        }
    }

    pub fn insert_last(&mut self, i: usize) {
        debug_assert!(self.elements.is_empty() || *self.elements.last().unwrap() < i as u32);
        self.elements.push(i as u32);
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::octet::Octet;
    use crate::sparse_vec::SparseBinaryVec;

    #[test]
    fn sparse_vec() {
        let size = 100;
        let mut dense = vec![0; size];
        let mut sparse = SparseBinaryVec::with_capacity(size);
        for _ in 0..size {
            let i = rand::thread_rng().gen_range(0, size);
            let value = rand::thread_rng().gen_range(0, 2);
            dense[i] = value;
            sparse.insert(i, Octet::new(value));
        }
        for i in 0..size {
            assert_eq!(dense[i], sparse.get(i).map(|x| x.byte()).unwrap_or(0));
        }
    }

    #[test]
    fn sparse_vec_fma() {
        let mut dense1 = vec![Octet::zero(); 8];
        let mut sparse1 = SparseBinaryVec::with_capacity(8);
        for i in 0..4 {
            let value = rand::thread_rng().gen_range(0, 2);
            dense1[i * 2] = Octet::new(value);
            sparse1.insert(i * 2, Octet::new(value));
        }

        for i in 0..8 {
            let actual = sparse1.get(i).unwrap_or(Octet::zero());
            let expected = dense1[i].clone();
            assert_eq!(
                actual, expected,
                "Mismatch at {}. {:?} != {:?}",
                i, actual, expected
            );
        }

        let mut dense2 = vec![Octet::zero(); 8];
        let mut sparse2 = SparseBinaryVec::with_capacity(8);
        for i in 0..4 {
            let value = rand::thread_rng().gen_range(0, 2);
            dense2[i] = Octet::new(value);
            sparse2.insert(i, Octet::new(value));
        }

        for i in 0..8 {
            let actual = sparse2.get(i).unwrap_or(Octet::zero());
            let expected = dense2[i].clone();
            assert_eq!(
                actual, expected,
                "Mismatch at {}. {:?} != {:?}",
                i, actual, expected
            );
        }

        sparse1.add_assign(&sparse2);

        for i in 0..8 {
            let actual = sparse1.get(i).unwrap_or(Octet::zero());
            let expected = &dense1[i] + &dense2[i];
            assert_eq!(
                actual, expected,
                "Mismatch at {}. {:?} != {:?}",
                i, actual, expected
            );
        }
    }
}
