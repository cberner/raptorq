use crate::octet::Octet;
use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq)]
pub struct SparseVec<T: Clone> {
    // Kept sorted by the usize (key)
    pub elements: Vec<(usize, T)>,
}

impl<T: Clone> SparseVec<T> {
    pub fn with_capacity(capacity: usize) -> SparseVec<T> {
        SparseVec {
            elements: Vec::with_capacity(capacity),
        }
    }

    pub fn retain<P: Fn(&(usize, T)) -> bool>(&mut self, predicate: P) {
        self.elements.retain(predicate);
    }

    // Returns the internal index into self.elements matching key i, or the index
    // at which it can be inserted (maintaining sorted order)
    fn key_to_internal_index(&self, i: usize) -> Result<usize, usize> {
        self.elements.binary_search_by_key(&i, |(index, _)| *index)
    }

    pub fn remove(&mut self, i: usize) -> Option<T> {
        match self.key_to_internal_index(i) {
            Ok(index) => Some(self.elements.remove(index).1),
            Err(_) => None,
        }
    }

    pub fn get(&self, i: usize) -> Option<&T> {
        match self.key_to_internal_index(i) {
            Ok(index) => Some(&self.elements[index].1),
            Err(_) => None,
        }
    }

    pub fn keys_values(&self) -> impl Iterator<Item = &(usize, T)> {
        self.elements.iter()
    }

    pub fn insert(&mut self, i: usize, value: T) {
        match self.key_to_internal_index(i) {
            Ok(index) => self.elements[index] = (i, value),
            Err(index) => self.elements.insert(index, (i, value)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SparseOctetVec {
    // Kept sorted by the usize (key)
    pub elements: SparseVec<Octet>,
}

impl SparseOctetVec {
    pub fn with_capacity(capacity: usize) -> SparseOctetVec {
        SparseOctetVec {
            elements: SparseVec::with_capacity(capacity),
        }
    }

    // Returns a vector of new column indices that this row contains
    pub fn fma(&mut self, other: &SparseOctetVec, scalar: &Octet) -> Vec<usize> {
        // Fast path for a single value that's being eliminated
        // TODO: Probably wouldn't need this if we implemented "Furthermore, the row operations
        // required for the HDPC rows may be performed for all such rows in one
        // process, by using the algorithm described in Section 5.3.3.3."
        if other.elements.elements.len() == 1 {
            let (other_col, other_value) = &other.elements.elements[0];
            match self.elements.key_to_internal_index(*other_col) {
                Ok(index) => {
                    let elements_len = self.elements.elements.len();
                    let self_value = &mut self.elements.elements[index].1;
                    self_value.fma(other_value, scalar);
                    // XXX: heuristic for handling large rows, since these are somewhat common (HDPC rows)
                    // It would be very expensive to always remove from those rows
                    if elements_len < 1000 && *self_value == Octet::zero() {
                        self.elements.elements.remove(index);
                    }
                }
                Err(index) => {
                    let value = other_value * scalar;
                    self.elements.elements.insert(index, (*other_col, value));
                    return vec![*other_col];
                }
            };
            return vec![];
        }

        let mut result =
            Vec::with_capacity(self.elements.elements.len() + other.elements.elements.len());
        let mut self_iter = self.elements.elements.iter();
        let mut other_iter = other.elements.elements.iter();
        let mut self_entry = self_iter.next();
        let mut other_entry = other_iter.next();

        let mut new_columns = Vec::with_capacity(10);
        loop {
            if let Some((self_col, self_value)) = self_entry {
                if let Some((other_col, other_value)) = other_entry {
                    match self_col.cmp(&other_col) {
                        Ordering::Less => {
                            if *self_value != Octet::zero() {
                                result.push((*self_col, self_value.clone()));
                            }
                            self_entry = self_iter.next();
                        }
                        Ordering::Equal => {
                            let value = self_value + &(other_value * scalar);
                            if value != Octet::zero() {
                                result.push((*self_col, value));
                            }
                            self_entry = self_iter.next();
                            other_entry = other_iter.next();
                        }
                        Ordering::Greater => {
                            if *other_value != Octet::zero() {
                                new_columns.push(*other_col);
                                result.push((*other_col, other_value * scalar));
                            }
                            other_entry = other_iter.next();
                        }
                    }
                } else {
                    if *self_value != Octet::zero() {
                        result.push((*self_col, self_value.clone()));
                    }
                    self_entry = self_iter.next();
                }
            } else if let Some((other_col, other_value)) = other_entry {
                if *other_value != Octet::zero() {
                    new_columns.push(*other_col);
                    result.push((*other_col, other_value * scalar));
                }
                other_entry = other_iter.next();
            } else {
                break;
            }
        }
        self.elements.elements = result;

        return new_columns;
    }

    pub fn remove(&mut self, i: usize) -> Option<Octet> {
        self.elements.remove(i)
    }

    pub fn retain<P: Fn(&(usize, Octet)) -> bool>(&mut self, predicate: P) {
        self.elements.retain(predicate);
    }

    pub fn get(&self, i: usize) -> Option<&Octet> {
        self.elements.get(i)
    }

    pub fn mul_assign(&mut self, scalar: &Octet) {
        for (_, value) in self.elements.elements.iter_mut() {
            *value = value as &Octet * scalar;
        }
    }

    pub fn keys_values(&self) -> impl Iterator<Item = &(usize, Octet)> {
        self.elements.keys_values()
    }

    pub fn insert(&mut self, i: usize, value: Octet) {
        self.elements.insert(i, value);
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::octet::Octet;
    use crate::sparse_vec::SparseOctetVec;

    #[test]
    fn sparse_vec() {
        let size = 100;
        let mut dense = vec![0; size];
        let mut sparse = SparseOctetVec::with_capacity(size);
        for _ in 0..size {
            let i = rand::thread_rng().gen_range(0, size);
            let value = rand::thread_rng().gen();
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
        let mut sparse1 = SparseOctetVec::with_capacity(8);
        for i in 0..4 {
            let value = rand::thread_rng().gen();
            dense1[i * 2] = Octet::new(value);
            sparse1.insert(i * 2, Octet::new(value));
        }

        for i in 0..8 {
            let actual = sparse1.get(i).map(|x| x.clone()).unwrap_or(Octet::zero());
            let expected = dense1[i].clone();
            assert_eq!(
                actual, expected,
                "Mismatch at {}. {:?} != {:?}",
                i, actual, expected
            );
        }

        let mut dense2 = vec![Octet::zero(); 8];
        let mut sparse2 = SparseOctetVec::with_capacity(8);
        for i in 0..4 {
            let value = rand::thread_rng().gen();
            dense2[i] = Octet::new(value);
            sparse2.insert(i, Octet::new(value));
        }

        for i in 0..8 {
            let actual = sparse2.get(i).map(|x| x.clone()).unwrap_or(Octet::zero());
            let expected = dense2[i].clone();
            assert_eq!(
                actual, expected,
                "Mismatch at {}. {:?} != {:?}",
                i, actual, expected
            );
        }

        sparse1.fma(&sparse2, &Octet::new(5));

        for i in 0..8 {
            let actual = sparse1.get(i).map(|x| x.clone()).unwrap_or(Octet::zero());
            let expected = &dense1[i] + &(&Octet::new(5) * &dense2[i]);
            assert_eq!(
                actual, expected,
                "Mismatch at {}. {:?} != {:?}",
                i, actual, expected
            );
        }
    }
}
