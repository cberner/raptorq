use crate::octet::Octet;
use crate::octets::{add_assign, mulassign_scalar, count_ones_and_nonzeros};
use crate::octets::fused_addassign_mul_scalar;
use crate::util::get_both_indices;
use std::cmp::min;

pub struct KeyIter {
    sparse: bool,
    dense_index: usize,
    dense_end: usize,
    sparse_rows: Option<Vec<usize>>
}

impl Iterator for KeyIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.sparse {
            return self.sparse_rows.as_mut().unwrap().pop();
        }
        else {
            if self.dense_index == self.dense_end {
                return None;
            }
            else {
                let old_index = self.dense_index;
                self.dense_index += 1;
                return Some(old_index);
            }
        }
    }
}

pub struct ClonedOctetIter {
    sparse: bool,
    end_col: usize,
    dense_elements: Option<Vec<u8>>,
    dense_index: usize,
    sparse_elements: Option<Vec<(usize, Octet)>>,
    sparse_index: usize
}

impl Iterator for ClonedOctetIter {
    type Item = (usize, Octet);

    fn next(&mut self) -> Option<Self::Item> {
        if self.sparse {
            let elements = self.sparse_elements.as_ref().unwrap();
            if self.sparse_index == elements.len() {
                return None;
            }
            else {
                let old_index = self.sparse_index;
                self.sparse_index += 1;
                return Some(elements[old_index].clone());
            }
        }
        else {
            if self.dense_index == self.end_col {
                return None;
            }
            else {
                let old_index = self.dense_index;
                self.dense_index += 1;
                return Some((old_index, Octet::new(self.dense_elements.as_ref().unwrap()[old_index].clone())));
            }
        }
    }
}

pub struct OctetIter<'a> {
    sparse: bool,
    start_col: usize,
    end_col: usize,
    dense_elements: Option<&'a Vec<u8>>,
    dense_index: usize,
    sparse_elements: Option<&'a Vec<(usize, Octet)>>,
    sparse_index: usize,
    sparse_physical_col_to_logical: Option<&'a Vec<usize>>
}

impl <'a> OctetIter<'a> {
    pub fn clone(&self) -> ClonedOctetIter {
        // Convert to logical indices, since ClonedOctetIter doesn't handle physical
        let sparse_elements = self.sparse_elements.map(|x| x.iter()
            .map(|(physical_col, value)| (self.sparse_physical_col_to_logical.unwrap()[*physical_col], value.clone()))
            .filter(|(logical_col, _)| *logical_col >= self.start_col && *logical_col < self.end_col)
            .collect());
        ClonedOctetIter {
            sparse: self.sparse,
            end_col: self.end_col,
            dense_elements: self.dense_elements.map(|x| x.clone()),
            dense_index: self.dense_index,
            sparse_elements,
            sparse_index: self.sparse_index
        }
    }
}

impl <'a> Iterator for OctetIter<'a> {
    type Item = (usize, Octet);

    fn next(&mut self) -> Option<Self::Item> {
        if self.sparse {
            let elements = self.sparse_elements.unwrap();
            // Need to iterate over the whole array, since they're not sorted by logical col
            if self.sparse_index >= elements.len() {
                return None;
            }
            else {
                while self.sparse_index < elements.len() {
                    let entry = &elements[self.sparse_index];
                    self.sparse_index += 1;
                    let logical_col = self.sparse_physical_col_to_logical.unwrap()[entry.0];
                    if logical_col >= self.start_col && logical_col < self.end_col {
                        return Some((logical_col, entry.1.clone()));
                    }
                }
                return None;
            }
        }
        else {
            if self.dense_index == self.end_col {
                return None;
            }
            else {
                let old_index = self.dense_index;
                self.dense_index += 1;
                return Some((old_index, Octet::new(self.dense_elements.unwrap()[old_index].clone())));
            }
        }
    }
}

pub trait OctetMatrix: Clone {
    fn new(height: usize, width: usize, trailing_dense_column_hint: usize) -> Self;

    fn set(&mut self, i: usize, j: usize, value: Octet);

    fn height(&self) -> usize;

    fn width(&self) -> usize;

    fn count_ones_and_nonzeros(&self, row: usize, start_col: usize, end_col: usize) -> (usize, usize);

    fn mul_assign_row(&mut self, row: usize, value: &Octet);

    // Once "impl Trait" is supported in traits, it would be better to return "impl Iterator<...>"
    fn get_row_iter(&self, row: usize, start_col: usize, end_col: usize) -> OctetIter;

    // An iterator over rows for the given col, that may have non-zero values
    fn get_col_index_iter(&self, col: usize, start_row: usize, end_row: usize) -> KeyIter;

    fn get(&self, i: usize, j: usize) -> Octet;

    fn swap_rows(&mut self, i: usize, j: usize);

    // start_row_hint indicates that all preceding rows don't need to be swapped, because they have
    // identical values
    fn swap_columns(&mut self, i: usize, j: usize, start_row_hint: usize);

    // After calling this method swap_columns() and other column oriented methods, may be much slower
    fn disable_column_acccess_acceleration(&mut self);

    // Hints that column i will not be swapped again, and is likely to become dense'ish
    fn hint_column_dense_and_frozen(&mut self, i: usize);

    // other must be a rows x rows matrix
    // sets self[0..rows][..] = X * self[0..rows][..]
    fn mul_assign_submatrix(&mut self, other: &Self, rows: usize);

    fn fma_rows(&mut self, dest: usize, multiplicand: usize, scalar: &Octet);

    fn resize(&mut self, new_height: usize, new_width: usize);
}

#[derive(Clone, Debug, PartialEq)]
pub struct DenseOctetMatrix {
    height: usize,
    width: usize,
    elements: Vec<Vec<u8>>,
}

impl OctetMatrix for DenseOctetMatrix {
    fn new(height: usize, width: usize, _: usize) -> DenseOctetMatrix {
        let mut elements: Vec<Vec<u8>> = Vec::with_capacity(height);
        for _ in 0..height {
            elements.push(vec![0; width]);
        }
        DenseOctetMatrix {
            height,
            width,
            elements,
        }
    }

    fn set(&mut self, i: usize, j: usize, value: Octet) {
        self.elements[i][j] = value.byte();
    }

    fn height(&self) -> usize {
        self.height
    }

    fn width(&self) -> usize {
        self.width
    }

    fn count_ones_and_nonzeros(&self, row: usize, start_col: usize, end_col: usize) -> (usize, usize) {
        count_ones_and_nonzeros(&self.elements[row][start_col..end_col])
    }

    fn mul_assign_row(&mut self, row: usize, value: &Octet) {
        mulassign_scalar(&mut self.elements[row], value);
    }

    fn get_row_iter(&self, row: usize, start_col: usize, end_col: usize) -> OctetIter {
        OctetIter {
            sparse: false,
            start_col,
            end_col,
            dense_elements: Some(&self.elements[row]),
            dense_index: start_col,
            sparse_elements: None,
            sparse_index: 0,
            sparse_physical_col_to_logical: None
        }
    }

    fn get_col_index_iter(&self, _: usize, start_row: usize, end_row: usize) -> KeyIter {
        KeyIter {
            sparse: false,
            dense_index: start_row,
            dense_end: end_row,
            sparse_rows: None
        }
    }

    fn get(&self, i: usize, j: usize) -> Octet {
        Octet::new(self.elements[i][j])
    }

    fn swap_rows(&mut self, i: usize, j: usize) {
        self.elements.swap(i, j);
    }

    fn swap_columns(&mut self, i: usize, j: usize, start_row_hint: usize) {
        for row in start_row_hint..self.elements.len() {
            self.elements[row].swap(i, j);
        }
    }

    fn disable_column_acccess_acceleration(&mut self) {
        // No-op
    }

    fn hint_column_dense_and_frozen(&mut self, _: usize) {
        // No-op
    }

    // other must be a rows x rows matrix
    // sets self[0..rows][..] = X * self[0..rows][..]
    fn mul_assign_submatrix(&mut self, other: &DenseOctetMatrix, rows: usize) {
        assert_eq!(rows, other.height());
        assert_eq!(rows, other.width());
        assert!(rows <= self.height());
        let mut temp = vec![vec![0; self.width]; rows];
        for row in 0..rows {
            for i in 0..rows {
                let scalar = other.get(row, i);
                if scalar == Octet::zero() {
                    continue;
                }
                if scalar == Octet::one() {
                    add_assign(&mut temp[row], &self.elements[i]);
                } else {
                    fused_addassign_mul_scalar(&mut temp[row], &self.elements[i], &scalar);
                }
            }
        }
        for row in (0..rows).rev() {
            self.elements[row] = temp.pop().unwrap();
        }
    }

    fn fma_rows(&mut self, dest: usize, multiplicand: usize, scalar: &Octet) {
        assert_ne!(dest, multiplicand);
        let (dest_row, temp_row) = get_both_indices(&mut self.elements, dest, multiplicand);

        if *scalar == Octet::one() {
            add_assign(dest_row, temp_row);
        } else {
            fused_addassign_mul_scalar(dest_row, temp_row, scalar);
        }
    }

    fn resize(&mut self, new_height: usize, new_width: usize) {
        assert!(new_height <= self.height);
        assert!(new_width <= self.width);
        self.elements.truncate(new_height);
        for row in 0..self.elements.len() {
            self.elements[row].truncate(new_width);
        }
        self.height = new_height;
        self.width = new_width;
    }
}

#[derive(Clone, Debug, PartialEq)]
struct SparseVec<T: Clone> {
    // Kept sorted by the usize (key)
    elements: Vec<(usize, T)>
}

impl <T: Clone> SparseVec<T> {
    pub fn with_capacity(capacity: usize) -> SparseVec<T> {
        SparseVec {
            elements: Vec::with_capacity(capacity)
        }
    }

    pub fn retain<P: Fn(&(usize, T)) -> bool>(&mut self, predicate: P) {
        self.elements.retain(predicate);
    }

    pub fn remove(&mut self, i: usize) -> Option<T> {
        match self.elements.binary_search_by_key(&i, |(col, _)| *col) {
            Ok(index) => Some(self.elements.remove(index).1),
            Err(_) => None
        }
    }

    pub fn get(&self, i: &usize) -> Option<&T> {
        match self.elements.binary_search_by_key(i, |(col, _)| *col) {
            Ok(index) => Some(&self.elements[index].1),
            Err(_) => None
        }
    }

    pub fn keys_values(&self) -> impl Iterator<Item=&(usize, T)> {
        self.elements.iter()
    }

    pub fn insert(&mut self, i: usize, value: T) {
        match self.elements.binary_search_by_key(&i, |(col, _)| *col) {
            Ok(index) => self.elements[index] = (i, value),
            Err(index) => self.elements.insert(index, (i, value))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct SparseOctetVec {
    // Kept sorted by the usize (key)
    elements: SparseVec<Octet>
}

impl SparseOctetVec {
    pub fn with_capacity(capacity: usize) -> SparseOctetVec {
        SparseOctetVec {
            elements: SparseVec::with_capacity(capacity)
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
            // XXX: heuristic for handling large rows, since these are somewhat common (HDPC rows)
            if self.elements.elements.len() > 1000 {
                let self_value= self.elements.get(other_col)
                    .map(|x| x.clone())
                    .unwrap_or(Octet::zero());
                let value = &self_value + &(other_value * scalar);
                self.elements.insert(*other_col, value.clone());
                if value != Octet::zero() {
                    return vec![*other_col];
                }
            }
            else {
                if let Some(self_value) = self.elements.remove(*other_col) {
                    let value = &self_value + &(other_value * scalar);
                    if value != Octet::zero() {
                        self.elements.insert(*other_col, value);
                    }
                }
                else {
                    self.elements.insert(*other_col, other_value * scalar);
                    return vec![*other_col];
                }
            }
            return vec![];
        }

        let mut result = Vec::with_capacity(self.elements.elements.len() + other.elements.elements.len());
        let mut self_iter = self.elements.elements.iter();
        let mut other_iter = other.elements.elements.iter();
        let mut self_entry = self_iter.next();
        let mut other_entry = other_iter.next();

        let mut new_columns = Vec::with_capacity(10);
        loop {
            if let Some((self_col, self_value)) = self_entry {
                if let Some((other_col, other_value)) = other_entry {
                    if self_col < other_col {
                        if *self_value != Octet::zero() {
                            result.push((*self_col, self_value.clone()));
                        }
                        self_entry = self_iter.next();
                    }
                    else if self_col == other_col {
                        let value = self_value + &(other_value * scalar);
                        if value != Octet::zero() {
                            result.push((*self_col, value));
                        }
                        self_entry = self_iter.next();
                        other_entry = other_iter.next();
                    }
                    else {
                        if *other_value != Octet::zero() {
                            new_columns.push(*other_col);
                            result.push((*other_col, other_value * scalar));
                        }
                        other_entry = other_iter.next();
                    }
                }
                else {
                    if *self_value != Octet::zero() {
                        result.push((*self_col, self_value.clone()));
                    }
                    self_entry = self_iter.next();
                }
            }
            else {
                if let Some((other_col, other_value)) = other_entry {
                    if *other_value != Octet::zero() {
                        new_columns.push(*other_col);
                        result.push((*other_col, other_value * scalar));
                    }
                    other_entry = other_iter.next();
                }
                else {
                    break;
                }
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

    pub fn get(&self, i: &usize) -> Option<&Octet> {
        self.elements.get(i)
    }

    pub fn mul_assign(&mut self, scalar: &Octet) {
        for (_, value) in self.elements.elements.iter_mut() {
            *value = value as &Octet * scalar;
        }
    }

    pub fn keys_values(&self) -> impl Iterator<Item=&(usize, Octet)> {
        self.elements.keys_values()
    }

    pub fn insert(&mut self, i: usize, value: Octet) {
        self.elements.insert(i, value);
    }
}

// Stores a matrix in sparse representation, with an optional dense block for the right most columns
#[derive(Clone, Debug, PartialEq)]
pub struct SparseOctetMatrix {
    height: usize,
    width: usize,
    sparse_elements: Vec<SparseOctetVec>,
    // Note these are stored with the right-most element first in the vec.
    // That is, for a matrix with width 10 and num_dense 3, the last three will be stored in these
    // Vecs, and will be in the order: [9, 8, 7]
    dense_elements: Vec<Vec<u8>>,
    // Sparse vector indicating which rows may have a non-zero value in the given column
    // Does not guarantee that the row has a non-zero value, since FMA may have added to zero
    sparse_column_index: Vec<SparseVec<()>>,
    // Mapping of logical row numbers to index in sparse_elements, dense_elements, and sparse_column_index
    logical_row_to_physical: Vec<usize>,
    physical_row_to_logical: Vec<usize>,
    logical_col_to_physical: Vec<usize>,
    physical_col_to_logical: Vec<usize>,
    column_index_disabled: bool,
    num_dense_columns: usize
}

impl SparseOctetMatrix {
    #[cfg(debug_assertions)]
    fn verify(&self) {
        if self.column_index_disabled {
            return;
        }
        for row in 0..self.height {
            for (col, value) in self.sparse_elements[row].keys_values() {
                if *value != Octet::zero() {
                    self.sparse_column_index[*col].get(&row).unwrap();
                }
            }
        }
    }
}

impl OctetMatrix for SparseOctetMatrix {
    fn new(height: usize, width: usize, trailing_dense_column_hint: usize) -> SparseOctetMatrix {
        let mut row_mapping = vec![0; height];
        let mut col_mapping = vec![0; width];
        let mut elements = Vec::with_capacity(height);
        for i in 0..height {
            elements.push(SparseOctetVec::with_capacity(10));
            row_mapping[i] = i;
        }
        let mut dense_elements = Vec::with_capacity(height);
        for _ in 0..height {
            dense_elements.push(vec![0; trailing_dense_column_hint]);
        }
        let mut column_index = Vec::with_capacity(width);
        for i in 0..width {
            column_index.push(SparseVec::with_capacity(10));
            col_mapping[i] = i;
        }
        SparseOctetMatrix {
            height,
            width,
            sparse_elements: elements,
            dense_elements,
            sparse_column_index: column_index,
            logical_row_to_physical: row_mapping.clone(),
            physical_row_to_logical: row_mapping,
            logical_col_to_physical: col_mapping.clone(),
            physical_col_to_logical: col_mapping,
            column_index_disabled: false,
            num_dense_columns: trailing_dense_column_hint
        }
    }

    fn set(&mut self, i: usize, j: usize, value: Octet) {
        let physical_i = self.logical_row_to_physical[i];
        let physical_j = self.logical_col_to_physical[j];
        if self.width - j <= self.num_dense_columns {
            self.dense_elements[physical_i][self.width - j - 1] = value.byte();
        }
        else {
            self.sparse_elements[physical_i].insert(physical_j, value);
        }
        if !self.column_index_disabled {
            self.sparse_column_index[physical_j].insert(physical_i, ());
        }
    }

    fn height(&self) -> usize {
        self.height
    }

    fn width(&self) -> usize {
        self.width
    }

    fn count_ones_and_nonzeros(&self, row: usize, start_col: usize, end_col: usize) -> (usize, usize) {
        if end_col > self.width - self.num_dense_columns {
            unimplemented!("It was assumed that this wouldn't be needed, because the method would only be called on the V section of matrix A");
        }
        let mut ones = 0;
        let mut nonzeros = 0;
        let physical_row = self.logical_row_to_physical[row];
        for (physical_col, value) in self.sparse_elements[physical_row].keys_values() {
            let col = self.physical_col_to_logical[*physical_col];
            if col >= start_col && col < end_col {
                if *value == Octet::one() {
                    ones += 1;
                }
                if *value != Octet::zero() {
                    nonzeros += 1;
                }
            }
        }
        return (ones, nonzeros);
    }

    fn mul_assign_row(&mut self, row: usize, value: &Octet) {
        let physical_row = self.logical_row_to_physical[row];
        self.sparse_elements[physical_row].mul_assign(value);
        mulassign_scalar(&mut self.dense_elements[physical_row], value);
    }

    fn get(&self, i: usize, j: usize) -> Octet {
        let physical_i = self.logical_row_to_physical[i];
        let physical_j = self.logical_col_to_physical[j];
        if self.width - j <= self.num_dense_columns {
            return Octet::new(self.dense_elements[physical_i][self.width - j - 1]);
        }
        else {
            return self.sparse_elements[physical_i].get(&physical_j).unwrap_or(&Octet::zero()).clone();
        }
    }

    fn get_row_iter(&self, row: usize, start_col: usize, end_col: usize) -> OctetIter {
        if end_col > self.width - self.num_dense_columns {
            unimplemented!("It was assumed that this wouldn't be needed, because the method would only be called on the V section of matrix A");
        }
        let physical_row = self.logical_row_to_physical[row];
        let sparse_elements = &self.sparse_elements[physical_row].elements.elements;
        OctetIter {
            sparse: true,
            start_col,
            end_col,
            dense_elements: None,
            dense_index: 0,
            sparse_elements: Some(sparse_elements),
            sparse_index: 0,
            sparse_physical_col_to_logical: Some(&self.physical_col_to_logical)
        }
    }

    fn get_col_index_iter(&self, col: usize, start_row: usize, end_row: usize) -> KeyIter {
        let physical_col = self.logical_col_to_physical[col];
        let rows = self.sparse_column_index[physical_col]
            .keys_values()
            .map(|(physical_row, _)| self.physical_row_to_logical[*physical_row])
            .filter(|row| *row >= start_row && *row < end_row)
            .collect();
        KeyIter {
            sparse: true,
            dense_index: 0,
            dense_end: 0,
            sparse_rows: Some(rows)
        }
    }

    fn swap_rows(&mut self, i: usize, j: usize) {
        let physical_i = self.logical_row_to_physical[i];
        let physical_j = self.logical_row_to_physical[j];
        self.logical_row_to_physical.swap(i, j);
        self.physical_row_to_logical.swap(physical_i, physical_j);
    }

    fn swap_columns(&mut self, i: usize, j: usize, _: usize) {
        if j >= self.width - self.num_dense_columns {
            unimplemented!("It was assumed that this wouldn't be needed, because the method would only be called on the V section of matrix A");
        }
        assert_eq!(self.column_index_disabled, false);

        let physical_i = self.logical_col_to_physical[i];
        let physical_j = self.logical_col_to_physical[j];
        self.logical_col_to_physical.swap(i, j);
        self.physical_col_to_logical.swap(physical_i, physical_j);
    }

    fn disable_column_acccess_acceleration(&mut self) {
        self.column_index_disabled = true;
        self.sparse_column_index.clear();
    }

    fn hint_column_dense_and_frozen(&mut self, i: usize) {
        assert_eq!(self.width - self.num_dense_columns - 1, i, "Can only freeze the last sparse column");
        let mut physical_row = 0;
        let physical_i = self.logical_col_to_physical[i];
        for (maybe_present_in_row, _) in self.sparse_column_index[physical_i].keys_values() {
            while physical_row < *maybe_present_in_row {
                self.dense_elements[physical_row].push(0);
                physical_row += 1;
            }
            if let Some(value) = self.sparse_elements[physical_row].remove(physical_i) {
                self.dense_elements[physical_row].push(value.byte());
            }
            else {
                self.dense_elements[physical_row].push(0);
            }
            physical_row += 1;
        }
        while physical_row < self.height {
            self.dense_elements[physical_row].push(0);
            physical_row += 1;
        }
        self.num_dense_columns += 1;
    }

    // other must be a rows x rows matrix
    // sets self[0..rows][..] = X * self[0..rows][..]
    fn mul_assign_submatrix(&mut self, other: &SparseOctetMatrix, rows: usize) {
        assert_eq!(rows, other.height());
        assert_eq!(rows, other.width());
        assert!(rows <= self.height());
        if other.num_dense_columns != 0 {
            unimplemented!();
        }
        // Note: rows are logically indexed
        let mut temp_sparse = vec![SparseOctetVec::with_capacity(10); rows];
        let mut temp_dense = vec![vec![0; self.num_dense_columns]; rows];
        for row in 0..rows {
            for (i, scalar) in other.get_row_iter(row, 0, rows) {
                let physical_i = self.logical_row_to_physical[i];
                if scalar != Octet::zero() {
                    temp_sparse[row].fma(&self.sparse_elements[physical_i], &scalar);
                    if scalar == Octet::one() {
                        add_assign(&mut temp_dense[row], &self.dense_elements[physical_i]);
                    } else {
                        fused_addassign_mul_scalar(&mut temp_dense[row], &self.dense_elements[physical_i], &scalar);
                    }
                }
            }
        }
        for row in (0..rows).rev() {
            let physical_row = self.logical_row_to_physical[row];
            self.sparse_elements[physical_row] = temp_sparse.pop().unwrap();
            self.dense_elements[physical_row] = temp_dense.pop().unwrap();
            if !self.column_index_disabled {
                for (col, _) in self.sparse_elements[physical_row].keys_values() {
                    self.sparse_column_index[*col].insert(physical_row, ())
                }
            }
        }

        #[cfg(debug_assertions)]
        self.verify();
    }

    fn fma_rows(&mut self, dest: usize, multiplicand: usize, scalar: &Octet) {
        assert_ne!(dest, multiplicand);
        let physical_dest = self.logical_row_to_physical[dest];
        let physical_multiplicand = self.logical_row_to_physical[multiplicand];
        // First handle the dense columns
        let (dest_row, temp_row) = get_both_indices(&mut self.dense_elements, physical_dest, physical_multiplicand);

        if *scalar == Octet::one() {
            add_assign(dest_row, temp_row);
        } else {
            fused_addassign_mul_scalar(dest_row, temp_row, scalar);
        }

        // Then the sparse columns
        let (dest_row, temp_row) = get_both_indices(&mut self.sparse_elements, physical_dest, physical_multiplicand);

        let new_columns = dest_row.fma(temp_row, scalar);
        if !self.column_index_disabled {
            for new_col in new_columns {
                self.sparse_column_index[new_col].insert(physical_dest, ());
            }
        }

        #[cfg(debug_assertions)]
        self.verify();
    }

    fn resize(&mut self, new_height: usize, new_width: usize) {
        assert!(new_height <= self.height);
        assert!(new_width <= self.width);
        if !self.column_index_disabled {
            unimplemented!("Resize should only be used in phase 2, after column indexing is no longer needed");
        }
        let mut new_sparse = vec![None; new_height];
        let mut new_dense = vec![None; new_height];

        for i in (0..self.sparse_elements.len()).rev() {
            let logical_row = self.physical_row_to_logical[i];
            let sparse = self.sparse_elements.pop();
            let dense = self.dense_elements.pop();
            if logical_row < new_height {
                new_sparse[logical_row] = sparse;
                new_dense[logical_row] = dense;
            }
        }
        self.logical_row_to_physical.truncate(new_height);
        self.physical_row_to_logical.truncate(new_height);
        for i in 0..new_height {
            self.logical_row_to_physical[i] = i;
            self.physical_row_to_logical[i] = i;
        }
        for row in new_sparse.drain(0..new_height) {
            self.sparse_elements.push(row.unwrap());
        }
        for row in new_dense.drain(0..new_height) {
            self.dense_elements.push(row.unwrap());
        }

        let mut columns_to_remove = self.width - new_width;
        let dense_columns_to_remove = min(self.num_dense_columns, columns_to_remove);
        // First remove from dense
        for row in 0..self.dense_elements.len() {
            self.dense_elements[row].truncate(self.num_dense_columns - dense_columns_to_remove);
        }
        columns_to_remove -= dense_columns_to_remove;

        // Next remove sparse columns
        if columns_to_remove > 0 {
            let physical_to_logical = &self.physical_col_to_logical;
            for row in 0..self.sparse_elements.len() {
                // Current number of sparse columns - number to remove
                let sparse_width = self.width - self.num_dense_columns - columns_to_remove;
                self.sparse_elements[row].retain(|(col, _)| physical_to_logical[*col] < sparse_width);
            }
        }
        self.num_dense_columns -= dense_columns_to_remove;

        self.height = new_height;
        self.width = new_width;

        #[cfg(debug_assertions)]
        self.verify();
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::matrix::{OctetMatrix, SparseOctetMatrix, SparseOctetVec};
    use crate::matrix::DenseOctetMatrix;
    use crate::octet::Octet;

    fn dense_identity(size: usize) -> DenseOctetMatrix {
        let mut result = DenseOctetMatrix::new(size, size, 0);
        for i in 0..size {
            result.set(i, i, Octet::one());
        }
        result
    }

    fn sparse_identity(size: usize) -> SparseOctetMatrix {
        let mut result = SparseOctetMatrix::new(size, size, 0);
        for i in 0..size {
            result.set(i, i, Octet::one());
        }
        result
    }

    fn rand_dense_and_sparse(size: usize) -> (DenseOctetMatrix, SparseOctetMatrix) {
        let mut dense = DenseOctetMatrix::new(size, size, 0);
        let mut sparse = SparseOctetMatrix::new(size, size, 1);
        // Generate 50% filled random matrices
        for _ in 0..(size * size / 2) {
            let i = rand::thread_rng().gen_range(0, size);
            let j = rand::thread_rng().gen_range(0, size);
            let value = rand::thread_rng().gen();
            dense.set(i, j, Octet::new(value));
            sparse.set(i, j, Octet::new(value));
        }

        return (dense, sparse);
    }

    fn assert_matrices_eq<T: OctetMatrix, U: OctetMatrix>(matrix1: &T, matrix2: &U) {
        assert_eq!(matrix1.height(), matrix2.height());
        assert_eq!(matrix1.width(), matrix2.width());
        for i in 0..matrix1.height() {
            for j in 0..matrix1.width() {
                assert_eq!(matrix1.get(i, j), matrix2.get(i, j), "Matrices are not equal at row={} col={}", i, j);
            }
        }
    }

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
            assert_eq!(dense[i], sparse.get(&i).map(|x| x.byte()).unwrap_or(0));
        }
    }

    #[test]
    fn swap_rows() {
        // rand_dense_and_sparse uses set(), so just check that it works
        let (mut dense, mut sparse) = rand_dense_and_sparse(8);
        dense.swap_rows(0, 4);
        dense.swap_rows(1, 6);
        dense.swap_rows(1, 7);
        sparse.swap_rows(0, 4);
        sparse.swap_rows(1, 6);
        sparse.swap_rows(1, 7);
        assert_matrices_eq(&dense, &sparse);
    }

    #[test]
    fn swap_columns() {
        // rand_dense_and_sparse uses set(), so just check that it works
        let (mut dense, mut sparse) = rand_dense_and_sparse(8);
        dense.swap_columns(0, 4, 0);
        dense.swap_columns(1, 6, 0);
        dense.swap_columns(1, 1, 0);
        sparse.swap_columns(0, 4, 0);
        sparse.swap_columns(1, 6, 0);
        sparse.swap_columns(1, 1, 0);
        assert_matrices_eq(&dense, &sparse);
    }

    #[test]
    fn count_ones_and_nonzeros() {
        // rand_dense_and_sparse uses set(), so just check that it works
        let (dense, sparse) = rand_dense_and_sparse(8);
        assert_eq!(dense.count_ones_and_nonzeros(0, 0, 5), sparse.count_ones_and_nonzeros(0, 0, 5));
        assert_eq!(dense.count_ones_and_nonzeros(2, 2, 6), sparse.count_ones_and_nonzeros(2, 2, 6));
        assert_eq!(dense.count_ones_and_nonzeros(3, 1, 2), sparse.count_ones_and_nonzeros(3, 1, 2));
    }

    #[test]
    fn mul_assign_row() {
        // rand_dense_and_sparse uses set(), so just check that it works
        let (mut dense, mut sparse) = rand_dense_and_sparse(8);
        dense.mul_assign_row(0, &Octet::new(5));
        dense.mul_assign_row(2, &Octet::one());
        dense.mul_assign_row(7, &Octet::new(66));
        sparse.mul_assign_row(0, &Octet::new(5));
        sparse.mul_assign_row(2, &Octet::one());
        sparse.mul_assign_row(7, &Octet::new(66));
        assert_matrices_eq(&dense, &sparse);
    }

    #[test]
    fn mul_assign_submatrix() {
        // rand_dense_and_sparse uses set(), so just check that it works
        let (mut dense, mut sparse) = rand_dense_and_sparse(8);
        let original = dense.clone();

        let identity = dense_identity(5);
        dense.mul_assign_submatrix(&identity, 5);
        assert_matrices_eq(&dense, &original);

        let identity = dense_identity(8);
        dense.mul_assign_submatrix(&identity, 8);
        assert_matrices_eq(&dense, &original);

        let identity = sparse_identity(5);
        sparse.mul_assign_submatrix(&identity, 5);
        assert_matrices_eq(&sparse, &original);

        let identity = sparse_identity(8);
        sparse.mul_assign_submatrix(&identity, 8);
        assert_matrices_eq(&sparse, &original);
    }

    #[test]
    fn fma_rows() {
        // rand_dense_and_sparse uses set(), so just check that it works
        let (mut dense, mut sparse) = rand_dense_and_sparse(8);
        dense.fma_rows(0, 1, &Octet::new(5));
        dense.fma_rows(0, 2, &Octet::new(55));
        dense.fma_rows(2, 1, &Octet::one());
        sparse.fma_rows(0, 1, &Octet::new(5));
        sparse.fma_rows(0, 2, &Octet::new(55));
        sparse.fma_rows(2, 1, &Octet::one());
        assert_matrices_eq(&dense, &sparse);
    }

    #[test]
    fn resize() {
        // rand_dense_and_sparse uses set(), so just check that it works
        let (mut dense, mut sparse) = rand_dense_and_sparse(8);
        dense.disable_column_acccess_acceleration();
        sparse.disable_column_acccess_acceleration();
        dense.resize(5, 5);
        sparse.resize(5, 5);
        assert_matrices_eq(&dense, &sparse);
    }

    #[test]
    fn hint_column_dense_and_frozen() {
        // rand_dense_and_sparse uses set(), so just check that it works
        let (dense, mut sparse) = rand_dense_and_sparse(8);
        sparse.hint_column_dense_and_frozen(6);
        sparse.hint_column_dense_and_frozen(5);
        assert_matrices_eq(&dense, &sparse);
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
            let actual = sparse1.get(&i).map(|x| x.clone()).unwrap_or(Octet::zero());
            let expected = dense1[i].clone();
            assert_eq!(actual, expected, "Mismatch at {}. {:?} != {:?}", i, actual, expected);
        }

        let mut dense2 = vec![Octet::zero(); 8];
        let mut sparse2 = SparseOctetVec::with_capacity(8);
        for i in 0..4 {
            let value = rand::thread_rng().gen();
            dense2[i] = Octet::new(value);
            sparse2.insert(i, Octet::new(value));
        }

        for i in 0..8 {
            let actual = sparse2.get(&i).map(|x| x.clone()).unwrap_or(Octet::zero());
            let expected = dense2[i].clone();
            assert_eq!(actual, expected, "Mismatch at {}. {:?} != {:?}", i, actual, expected);
        }

        sparse1.fma(&sparse2, &Octet::new(5));

        for i in 0..8 {
            let actual = sparse1.get(&i).map(|x| x.clone()).unwrap_or(Octet::zero());
            let expected = &dense1[i] + &(&Octet::new(5) * &dense2[i]);
            assert_eq!(actual, expected, "Mismatch at {}. {:?} != {:?}", i, actual, expected);
        }
    }
}
