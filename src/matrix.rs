use crate::octet::Octet;
use crate::octets::{add_assign, mulassign_scalar, count_ones_and_nonzeros};
use crate::octets::fused_addassign_mul_scalar;
use crate::util::get_both_indices;
use std::ops::Mul;
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
            if self.sparse_index == elements.len() || elements[self.sparse_index].0 >= self.end_col {
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
    end_col: usize,
    dense_elements: Option<&'a Vec<u8>>,
    dense_index: usize,
    sparse_elements: Option<&'a Vec<(usize, Octet)>>,
    sparse_index: usize
}

impl <'a> OctetIter<'a> {
    pub fn clone(&self) -> ClonedOctetIter {
        ClonedOctetIter {
            sparse: self.sparse,
            end_col: self.end_col,
            dense_elements: self.dense_elements.map(|x| x.clone()),
            dense_index: self.dense_index,
            sparse_elements: self.sparse_elements.map(|x| x.clone()),
            sparse_index: self.sparse_index
        }
    }
}

impl <'a> Iterator for OctetIter<'a> {
    type Item = (usize, Octet);

    fn next(&mut self) -> Option<Self::Item> {
        if self.sparse {
            let elements = self.sparse_elements.unwrap();
            if self.sparse_index == elements.len() || elements[self.sparse_index].0 >= self.end_col {
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

    fn swap_columns(&mut self, i: usize, j: usize, start_row: usize);

    // After calling this method swap_columns can no longer be called
    fn freeze_columns(&mut self);

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
            end_col,
            dense_elements: Some(&self.elements[row]),
            dense_index: start_col,
            sparse_elements: None,
            sparse_index: 0
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

    fn swap_columns(&mut self, i: usize, j: usize, start_row: usize) {
        for row in start_row..self.elements.len() {
            self.elements[row].swap(i, j);
        }
    }

    fn freeze_columns(&mut self) {
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

impl<'a, 'b> Mul<&'b DenseOctetMatrix> for &'a DenseOctetMatrix {
    type Output = DenseOctetMatrix;

    fn mul(self, rhs: &'b DenseOctetMatrix) -> DenseOctetMatrix {
        assert_eq!(self.width, rhs.height);
        let mut result = DenseOctetMatrix::new(self.height, rhs.width, 0);
        for row in 0..self.height {
            for i in 0..self.width {
                let scalar = self.get(row, i);
                if scalar == Octet::zero() {
                    continue;
                }
                if scalar == Octet::one() {
                    add_assign(&mut result.elements[row], &rhs.elements[i]);
                } else {
                    fused_addassign_mul_scalar(
                        &mut result.elements[row],
                        &rhs.elements[i],
                        &scalar,
                    );
                }
            }
        }
        result
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

    pub fn truncate(&mut self, new_length: usize) {
        let mut to_remove = 0;
        for (col, _) in self.elements.iter().rev() {
            if *col >= new_length {
                to_remove += 1;
            }
            else {
                break;
            }
        }
        self.elements.truncate(self.elements.len() - to_remove);
    }

    pub fn remove(&mut self, i: usize) -> Option<T> {
        match self.elements.binary_search_by_key(&i, |(col, _)| *col) {
            Ok(index) => Some(self.elements.remove(index).1),
            Err(_) => None
        }
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        let (i_index, i_present) = match self.elements.binary_search_by_key(&i, |(col, _)| *col) {
            Ok(index) => (index, true),
            Err(index) => (index, false)
        };
        let (j_index, j_present) = match self.elements.binary_search_by_key(&j, |(col, _)| *col) {
            Ok(index) => (index, true),
            Err(index) => (index, false)
        };

        // If both keys are present, just swap the values
        if i_present && j_present {
            let temp = self.elements[i_index].1.clone();
            self.elements[i_index].1 = self.elements[j_index].1.clone();
            self.elements[j_index].1 = temp;
            return;
        }
        // If neither is present, this is a no-op since we're swapping implicit zeros
        if !i_present && !j_present {
            return;
        }

        let from_index;
        let to_index;
        let entry;
        if i_present {
            from_index = i_index;
            to_index = j_index;
            entry = (j, self.elements[i_index].1.clone());
        }
        else {
            from_index = j_index;
            to_index = i_index;
            entry = (i, self.elements[j_index].1.clone());
        }

        // Move all entries that are in between
        if from_index < to_index {
            let mut index = from_index;
            while index < to_index - 1 {
                self.elements[index] = self.elements[index + 1].clone();
                index += 1;
            }
            self.elements[to_index - 1] = entry;
        }
        else {
            let mut index = from_index;
            while index > to_index {
                self.elements[index] = self.elements[index - 1].clone();
                index -= 1;
            }
            self.elements[to_index] = entry;
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
    elements: SparseVec<Octet>,
    // Number of zero elements in elements vec
    zeros: usize
}

impl SparseOctetVec {
    pub fn with_capacity(capacity: usize) -> SparseOctetVec {
        SparseOctetVec {
            elements: SparseVec::with_capacity(capacity),
            zeros: 0
        }
    }

    // Returns a vector of new column indices that this row contains
    pub fn fma(&mut self, other: &SparseOctetVec, scalar: &Octet) -> Vec<usize> {
        // Fast path for a single value that's being eliminated
        // TODO: Probably wouldn't need this if we implemented "Furthermore, the row operations
        // required for the HDPC rows may be performed for all such rows in one
        // process, by using the algorithm described in Section 5.3.3.3."
        if other.elements.elements.len() == 1 &&
            (self.zeros as f32 <= 0.9 * self.elements.elements.len() as f32) { // Heuristic to compress out the zeros, if they are 90% of the vector
            let (other_col, other_value) = &other.elements.elements[0];
            // XXX: heuristic for handling large rows, since these are somewhat common (HDPC rows)
            if self.elements.elements.len() > 1000 {
                let self_value= self.elements.get(other_col)
                    .map(|x| x.clone())
                    .unwrap_or(Octet::zero());
                let value = &self_value + &(other_value * scalar);
                if value == Octet::zero() {
                    // Keep track of stored zeros, so they can be GC'ed later
                    self.zeros += 1;
                }
                self.elements.insert(*other_col, value);
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
        self.zeros = 0;

        return new_columns;
    }

    pub fn remove(&mut self, i: usize) -> Option<Octet> {
        self.elements.remove(i)
    }

    pub fn truncate(&mut self, new_length: usize) {
        self.elements.truncate(new_length);
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.elements.swap(i, j);
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
    columns_frozen: bool,
    num_dense_columns: usize
}

impl OctetMatrix for SparseOctetMatrix {
    fn new(height: usize, width: usize, trailing_dense_column_hint: usize) -> SparseOctetMatrix {
        let mut elements = Vec::with_capacity(height);
        for _ in 0..height {
            elements.push(SparseOctetVec::with_capacity(10));
        }
        let mut dense_elements = Vec::with_capacity(height);
        for _ in 0..height {
            dense_elements.push(vec![0; trailing_dense_column_hint]);
        }
        let mut column_index = Vec::with_capacity(width);
        for _ in 0..width {
            column_index.push(SparseVec::with_capacity(10));
        }
        SparseOctetMatrix {
            height,
            width,
            sparse_elements: elements,
            dense_elements,
            sparse_column_index: column_index,
            columns_frozen: false,
            num_dense_columns: trailing_dense_column_hint
        }
    }

    fn set(&mut self, i: usize, j: usize, value: Octet) {
        if self.width - j <= self.num_dense_columns {
            self.dense_elements[i][self.width - j - 1] = value.byte();
        }
        else {
            self.sparse_elements[i].insert(j, value);
        }
        if !self.columns_frozen {
            self.sparse_column_index[j].insert(i, ());
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
        for (col, value) in self.sparse_elements[row].keys_values() {
            if *col >= start_col && *col < end_col {
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
        self.sparse_elements[row].mul_assign(value);
        mulassign_scalar(&mut self.dense_elements[row], value);
    }

    fn get(&self, i: usize, j: usize) -> Octet {
        if self.width - j <= self.num_dense_columns {
            return Octet::new(self.dense_elements[i][self.width - j - 1]);
        }
        else {
            return self.sparse_elements[i].get(&j).unwrap_or(&Octet::zero()).clone();
        }
    }

    fn get_row_iter(&self, row: usize, start_col: usize, end_col: usize) -> OctetIter {
        if end_col > self.width - self.num_dense_columns {
            unimplemented!("It was assumed that this wouldn't be needed, because the method would only be called on the V section of matrix A");
        }
        let sparse_elements = &self.sparse_elements[row].elements.elements;
        let sparse_index = match sparse_elements.binary_search_by_key(&start_col, |(col, _)| *col) {
            Ok(index) => index,
            Err(index) => index
        };
        OctetIter {
            sparse: true,
            end_col,
            dense_elements: None,
            dense_index: 0,
            sparse_elements: Some(sparse_elements),
            sparse_index
        }
    }

    fn get_col_index_iter(&self, col: usize, start_row: usize, end_row: usize) -> KeyIter {
        let rows = self.sparse_column_index[col]
            .keys_values()
            .map(|(row, _)| *row)
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
        self.sparse_elements.swap(i, j);
        self.dense_elements.swap(i, j);

        if self.columns_frozen {
            // No need to update the column index, if they are frozen
            return;
        }

        let mut i_iter = self.sparse_elements[i].keys_values().map(|(col, _)| col);
        let mut j_iter = self.sparse_elements[j].keys_values().map(|(col, _)| col);
        let mut i_entry = i_iter.next();
        let mut j_entry = j_iter.next();

        loop {
            if let Some(&i_col) = i_entry {
                if let Some(&j_col) = j_entry {
                    if i_col < j_col {
                        self.sparse_column_index[i_col].swap(i, j);
                        i_entry = i_iter.next();
                    }
                    else if i_col == j_col {
                        self.sparse_column_index[i_col].swap(i, j);
                        i_entry = i_iter.next();
                        j_entry = j_iter.next();
                    }
                    else {
                        self.sparse_column_index[j_col].swap(i, j);
                        j_entry = j_iter.next();
                    }
                }
                else {
                    self.sparse_column_index[i_col].swap(i, j);
                    i_entry = i_iter.next();
                }
            }
            else {
                if let Some(&j_col) = j_entry {
                    self.sparse_column_index[j_col].swap(i, j);
                    j_entry = j_iter.next();
                }
                else {
                    break;
                }
            }
        }
    }

    fn swap_columns(&mut self, i: usize, j: usize, start_row: usize) {
        if j >= self.width - self.num_dense_columns {
            unimplemented!("It was assumed that this wouldn't be needed, because the method would only be called on the V section of matrix A");
        }
        assert_eq!(self.columns_frozen, false);

        self.sparse_column_index.swap(i, j);
        let mut i_iter = self.sparse_column_index[i]
            .keys_values()
            .map(|(row, _)| row)
            .filter(|row| **row >= start_row);
        let mut j_iter = self.sparse_column_index[j]
            .keys_values()
            .map(|(row, _)| row)
            .filter(|row| **row >= start_row);
        let mut i_entry = i_iter.next();
        let mut j_entry = j_iter.next();

        loop {
            if let Some(&i_row) = i_entry {
                if let Some(&j_row) = j_entry {
                    if i_row < j_row {
                        self.sparse_elements[i_row].swap(i, j);
                        i_entry = i_iter.next();
                    }
                    else if i_row == j_row {
                        self.sparse_elements[i_row].swap(i, j);
                        i_entry = i_iter.next();
                        j_entry = j_iter.next();
                    }
                    else {
                        self.sparse_elements[j_row].swap(i, j);
                        j_entry = j_iter.next();
                    }
                }
                else {
                    self.sparse_elements[i_row].swap(i, j);
                    i_entry = i_iter.next();
                }
            }
            else {
                if let Some(&j_row) = j_entry {
                    self.sparse_elements[j_row].swap(i, j);
                    j_entry = j_iter.next();
                }
                else {
                    break;
                }
            }
        }
    }

    fn freeze_columns(&mut self) {
        self.columns_frozen = true;
        self.sparse_column_index.clear();
    }

    fn hint_column_dense_and_frozen(&mut self, i: usize) {
        assert_eq!(self.width - self.num_dense_columns - 1, i, "Can only freeze the last sparse column");
        let mut row = 0;
        for (maybe_present_in_row, _) in self.sparse_column_index[i].keys_values() {
            while row < *maybe_present_in_row {
                self.dense_elements[row].push(0);
                row += 1;
            }
            if let Some(value) = self.sparse_elements[row].remove(i) {
                self.dense_elements[row].push(value.byte());
            }
            else {
                self.dense_elements[row].push(0);
            }
            row += 1;
        }
        while row < self.height {
            self.dense_elements[row].push(0);
            row += 1;
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
        let mut temp_sparse = vec![SparseOctetVec::with_capacity(10); rows];
        let mut temp_dense = vec![vec![0; self.num_dense_columns]; rows];
        for row in 0..rows {
            for (i, scalar) in other.get_row_iter(row, 0, rows) {
                if scalar != Octet::zero() {
                    temp_sparse[row].fma(&self.sparse_elements[i], &scalar);
                    if scalar == Octet::one() {
                        add_assign(&mut temp_dense[row], &self.dense_elements[i]);
                    } else {
                        fused_addassign_mul_scalar(&mut temp_dense[row], &self.dense_elements[i], &scalar);
                    }
                }
            }
        }
        for row in (0..rows).rev() {
            self.sparse_elements[row] = temp_sparse.pop().unwrap();
            self.dense_elements[row] = temp_dense.pop().unwrap();
            if !self.columns_frozen {
                for (col, _) in self.sparse_elements[row].keys_values() {
                    self.sparse_column_index[*col].insert(row, ())
                }
            }
        }
    }

    fn fma_rows(&mut self, dest: usize, multiplicand: usize, scalar: &Octet) {
        assert_ne!(dest, multiplicand);
        // First handle the dense columns
        let (dest_row, temp_row) = get_both_indices(&mut self.dense_elements, dest, multiplicand);

        if *scalar == Octet::one() {
            add_assign(dest_row, temp_row);
        } else {
            fused_addassign_mul_scalar(dest_row, temp_row, scalar);
        }

        // Then the sparse columns
        let (dest_row, temp_row) = get_both_indices(&mut self.sparse_elements, dest, multiplicand);

        let new_columns = dest_row.fma(temp_row, scalar);
        if !self.columns_frozen {
            for new_col in new_columns {
                self.sparse_column_index[new_col].insert(dest, ());
            }
        }
    }

    fn resize(&mut self, new_height: usize, new_width: usize) {
        assert!(new_height <= self.height);
        assert!(new_width <= self.width);
        self.sparse_elements.truncate(new_height);
        self.dense_elements.truncate(new_height);

        let mut columns_to_remove = self.width - new_width;
        let dense_columns_to_remove = min(self.num_dense_columns, columns_to_remove);
        // First remove from dense
        for row in 0..self.dense_elements.len() {
            self.dense_elements[row].truncate(self.num_dense_columns - dense_columns_to_remove);
        }
        columns_to_remove -= dense_columns_to_remove;

        // Next remove sparse columns
        if columns_to_remove > 0 {
            for row in 0..self.sparse_elements.len() {
                // Current number of sparse columns - number to remove
                self.sparse_elements[row].truncate(self.width - self.num_dense_columns - columns_to_remove);
            }
        }
        self.num_dense_columns -= dense_columns_to_remove;

        if !self.columns_frozen {
            self.sparse_column_index.truncate(new_width);
            for col in 0..self.sparse_column_index.len() {
                self.sparse_column_index[col].truncate(new_height);
            }
        }
        self.height = new_height;
        self.width = new_width;
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
        dense.swap_columns(1, 6, 1);
        dense.swap_columns(1, 1, 5);
        sparse.swap_columns(0, 4, 0);
        sparse.swap_columns(1, 6, 1);
        sparse.swap_columns(1, 1, 5);
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
        dense.resize(5, 5);
        sparse.resize(5, 5);
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
