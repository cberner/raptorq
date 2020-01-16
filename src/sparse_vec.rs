use crate::octet::Octet;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::mem::size_of;

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Hash)]
pub struct SparseBinaryVec {
    // Kept sorted by the usize (key). Only ones are stored, zeros are implicit
    elements: Vec<u16>,
}

impl SparseBinaryVec {
    pub fn with_capacity(capacity: usize) -> SparseBinaryVec {
        // Matrix width can never exceed maximum L
        debug_assert!(capacity < 65536);
        SparseBinaryVec {
            elements: Vec::with_capacity(capacity),
        }
    }

    // Returns the internal index into self.elements matching key i, or the index
    // at which it can be inserted (maintaining sorted order)
    fn key_to_internal_index(&self, i: u16) -> Result<usize, usize> {
        self.elements.binary_search(&i)
    }

    pub fn size_in_bytes(&self) -> usize {
        size_of::<Self>() + size_of::<u16>() * self.elements.len()
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn get_by_raw_index(&self, i: usize) -> (usize, Octet) {
        (self.elements[i] as usize, Octet::one())
    }

    // Returns true, if a new column was added
    pub fn add_assign(&mut self, other: &SparseBinaryVec) -> bool {
        // Fast path for a single value that's being eliminated
        // TODO: Probably wouldn't need this if we implemented "Furthermore, the row operations
        // required for the HDPC rows may be performed for all such rows in one
        // process, by using the algorithm described in Section 5.3.3.3."
        if other.elements.len() == 1 {
            let other_index = &other.elements[0];
            match self.key_to_internal_index(*other_index) {
                Ok(index) => {
                    // Adding 1 + 1 = 0 in GF(256), so remove this
                    self.elements.remove(index);
                }
                Err(index) => {
                    self.elements.insert(index, *other_index);
                    return true;
                }
            };
            return false;
        }

        let mut result = Vec::with_capacity(self.elements.len() + other.elements.len());
        let mut self_iter = self.elements.iter();
        let mut other_iter = other.elements.iter();
        let mut self_next = self_iter.next();
        let mut other_next = other_iter.next();

        let mut column_added = false;
        loop {
            if let Some(self_index) = self_next {
                if let Some(other_index) = other_next {
                    match self_index.cmp(&other_index) {
                        Ordering::Less => {
                            result.push(self_index.clone());
                            self_next = self_iter.next();
                        }
                        Ordering::Equal => {
                            // Adding 1 + 1 = 0 in GF(256), so skip this index
                            self_next = self_iter.next();
                            other_next = other_iter.next();
                        }
                        Ordering::Greater => {
                            column_added = true;
                            result.push(*other_index);
                            other_next = other_iter.next();
                        }
                    }
                } else {
                    result.push(*self_index);
                    self_next = self_iter.next();
                }
            } else if let Some(other_index) = other_next {
                column_added = true;
                result.push(*other_index);
                other_next = other_iter.next();
            } else {
                break;
            }
        }
        self.elements = result;

        return column_added;
    }

    pub fn remove(&mut self, i: usize) -> Option<Octet> {
        match self.key_to_internal_index(i as u16) {
            Ok(index) => {
                self.elements.remove(index);
                Some(Octet::one())
            }
            Err(_) => None,
        }
    }

    pub fn retain<P: Fn(&(usize, Octet)) -> bool>(&mut self, predicate: P) {
        self.elements
            .retain(|entry| predicate(&(*entry as usize, Octet::one())));
    }

    pub fn get(&self, i: usize) -> Option<Octet> {
        match self.key_to_internal_index(i as u16) {
            Ok(_) => Some(Octet::one()),
            Err(_) => None,
        }
    }

    pub fn keys_values(&self) -> impl Iterator<Item = (usize, Octet)> + '_ {
        self.elements
            .iter()
            .map(|entry| (*entry as usize, Octet::one()))
    }

    pub fn insert(&mut self, i: usize, value: Octet) {
        debug_assert!(i < 65536);
        if value == Octet::zero() {
            self.remove(i);
        } else {
            match self.key_to_internal_index(i as u16) {
                Ok(_) => {}
                Err(index) => self.elements.insert(index, i as u16),
            }
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
