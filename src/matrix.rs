use crate::octet::Octet;
use crate::octets::{add_assign, mulassign_scalar, count_ones_and_nonzeros};
use crate::octets::fused_addassign_mul_scalar;
use crate::util::get_both_indices;
use std::ops::Mul;

pub trait OctetMatrix: Clone {
    fn new(height: usize, width: usize) -> Self;

    fn set(&mut self, i: usize, j: usize, value: Octet);

    fn height(&self) -> usize;

    fn width(&self) -> usize;

    fn count_ones_and_nonzeros(&self, row: usize, start_col: usize, end_col: usize) -> (usize, usize);

    fn mul_assign_row(&mut self, row: usize, value: &Octet);

    fn get(&self, i: usize, j: usize) -> Octet;

    fn swap_rows(&mut self, i: usize, j: usize);

    fn swap_columns(&mut self, i: usize, j: usize, start_row: usize);

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
    fn new(height: usize, width: usize) -> DenseOctetMatrix {
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
        let mut result = DenseOctetMatrix::new(self.height, rhs.width);
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
struct SparseOctetVec {
    // Kept sorted by the usize (key)
    elements: Vec<(usize, Octet)>
}

impl SparseOctetVec {
    pub fn with_capacity(capacity: usize) -> SparseOctetVec {
        SparseOctetVec {
            elements: Vec::with_capacity(capacity)
        }
    }

    pub fn fma(&mut self, other: &SparseOctetVec, scalar: &Octet) {
        let mut result = Vec::with_capacity(self.elements.len() + other.elements.len());
        let mut self_iter = self.elements.iter();
        let mut other_iter = other.elements.iter();
        let mut self_entry = self_iter.next();
        let mut other_entry = other_iter.next();

        loop {
            if let Some((self_col, self_value)) = self_entry {
                if let Some((other_col, other_value)) = other_entry {
                    if self_col < other_col {
                        result.push((*self_col, self_value.clone()));
                        self_entry = self_iter.next();
                    }
                    else if self_col == other_col {
                        result.push((*other_col, self_value + &(other_value * scalar)));
                        self_entry = self_iter.next();
                        other_entry = other_iter.next();
                    }
                    else {
                        result.push((*other_col, other_value * scalar));
                        other_entry = other_iter.next();
                    }
                }
                else {
                    result.push((*self_col, self_value.clone()));
                    self_entry = self_iter.next();
                }
            }
            else {
                if let Some((other_col, other_value)) = other_entry {
                    result.push((*other_col, other_value * scalar));
                    other_entry = other_iter.next();
                }
                else {
                    break;
                }
            }
        }
        self.elements = result;
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

    pub fn swap(&mut self, i: usize, j: usize) {
        let i_value = self.remove(&i);
        let j_value = self.remove(&j);
        if let Some(value) = i_value {
            self.insert(j, value);
        }
        if let Some(value) = j_value {
            self.insert(i, value);
        }
    }

    pub fn remove(&mut self, i: &usize) -> Option<Octet> {
        match self.elements.binary_search_by_key(i, |(col, _)| *col) {
            Ok(index) => Some(self.elements.remove(index).1),
            Err(_) => None
        }
    }

    pub fn get(&self, i: &usize) -> Option<&Octet> {
        match self.elements.binary_search_by_key(i, |(col, _)| *col) {
            Ok(index) => Some(&self.elements[index].1),
            Err(_) => None
        }
    }

    pub fn mul_assign(&mut self, scalar: &Octet) {
        for (_, value) in self.elements.iter_mut() {
            *value = value as &Octet * scalar;
        }
    }

    pub fn keys_values(&self) -> impl Iterator<Item=&(usize, Octet)> {
        self.elements.iter()
    }

    pub fn insert(&mut self, i: usize, value: Octet) {
        match self.elements.binary_search_by_key(&i, |(col, _)| *col) {
            Ok(index) => self.elements[index] = (i, value),
            Err(index) => self.elements.insert(index, (i, value))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SparseOctetMatrix {
    height: usize,
    width: usize,
    elements: Vec<SparseOctetVec>,
}

impl OctetMatrix for SparseOctetMatrix {
    fn new(height: usize, width: usize) -> SparseOctetMatrix {
        let mut elements = Vec::with_capacity(height);
        for _ in 0..height {
            elements.push(SparseOctetVec::with_capacity(10));
        }
        SparseOctetMatrix {
            height,
            width,
            elements,
        }
    }

    fn set(&mut self, i: usize, j: usize, value: Octet) {
        self.elements[i].insert(j, value);
    }

    fn height(&self) -> usize {
        self.height
    }

    fn width(&self) -> usize {
        self.width
    }

    fn count_ones_and_nonzeros(&self, row: usize, start_col: usize, end_col: usize) -> (usize, usize) {
        let mut ones = 0;
        let mut nonzeros = 0;
        for (col, value) in self.elements[row].keys_values() {
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
        self.elements[row].mul_assign(value);
    }

    fn get(&self, i: usize, j: usize) -> Octet {
        self.elements[i].get(&j).unwrap_or(&Octet::zero()).clone()
    }

    fn swap_rows(&mut self, i: usize, j: usize) {
        self.elements.swap(i, j);
    }

    fn swap_columns(&mut self, i: usize, j: usize, start_row: usize) {
        for row in start_row..self.elements.len() {
            self.elements[row].swap(i, j);
        }
    }

    // other must be a rows x rows matrix
    // sets self[0..rows][..] = X * self[0..rows][..]
    fn mul_assign_submatrix(&mut self, other: &SparseOctetMatrix, rows: usize) {
        assert_eq!(rows, other.height());
        assert_eq!(rows, other.width());
        assert!(rows <= self.height());
        let mut temp = vec![SparseOctetVec::with_capacity(10); rows];
        for row in 0..rows {
            for i in 0..rows {
                let scalar = other.get(row, i);
                if scalar == Octet::zero() {
                    continue;
                }
                temp[row].fma(&self.elements[i], &scalar);
            }
        }
        for row in (0..rows).rev() {
            self.elements[row] = temp.pop().unwrap();
        }
    }

    fn fma_rows(&mut self, dest: usize, multiplicand: usize, scalar: &Octet) {
        assert_ne!(dest, multiplicand);
        let (dest_row, temp_row) = get_both_indices(&mut self.elements, dest, multiplicand);

        dest_row.fma(temp_row, scalar);
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

impl<'a, 'b> Mul<&'b SparseOctetMatrix> for &'a SparseOctetMatrix {
    type Output = SparseOctetMatrix;

    fn mul(self, rhs: &'b SparseOctetMatrix) -> SparseOctetMatrix {
        assert_eq!(self.width, rhs.height);
        let mut result = SparseOctetMatrix::new(self.height, rhs.width);
        for row in 0..self.height {
            for i in 0..self.width {
                let scalar = self.get(row, i);
                if scalar == Octet::zero() {
                    continue;
                }
                result.elements[row].fma(&rhs.elements[i], &scalar);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::matrix::{OctetMatrix, SparseOctetMatrix, SparseOctetVec};
    use crate::matrix::DenseOctetMatrix;
    use crate::octet::Octet;

    fn identity(size: usize) -> DenseOctetMatrix {
        let mut result = DenseOctetMatrix::new(size, size);
        for i in 0..size {
            result.set(i, i, Octet::one());
        }
        result
    }

    fn sparse_identity(size: usize) -> SparseOctetMatrix {
        let mut result = SparseOctetMatrix::new(size, size);
        for i in 0..size {
            result.set(i, i, Octet::one());
        }
        result
    }

    fn rand_dense_and_sparse(size: usize) -> (DenseOctetMatrix, SparseOctetMatrix) {
        let mut dense = DenseOctetMatrix::new(size, size);
        let mut sparse = SparseOctetMatrix::new(size, size);
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

    fn assert_matrices_eq(dense: &DenseOctetMatrix, sparse: &SparseOctetMatrix) {
        assert_eq!(dense.height(), sparse.height());
        assert_eq!(dense.width(), sparse.width());
        for i in 0..dense.height() {
            for j in 0..dense.width() {
                assert_eq!(dense.get(i, j), sparse.get(i, j));
            }
        }
    }

    #[test]
    fn set() {
        // rand_dense_and_sparse uses set(), so just check that it works
        let (dense, sparse) = rand_dense_and_sparse(8);
        assert_matrices_eq(&dense, &sparse);
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
        dense.swap_columns(1, 7, 5);
        sparse.swap_columns(0, 4, 0);
        sparse.swap_columns(1, 6, 1);
        sparse.swap_columns(1, 7, 5);
        assert_matrices_eq(&dense, &sparse);
    }

    #[test]
    fn count_ones_and_nonzeros() {
        // rand_dense_and_sparse uses set(), so just check that it works
        let (dense, sparse) = rand_dense_and_sparse(8);
        assert_eq!(dense.count_ones_and_nonzeros(0, 0, 8), sparse.count_ones_and_nonzeros(0, 0, 8));
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
    fn mul() {
        let identity = identity(4);
        let mut a = DenseOctetMatrix::new(4, 5);
        for i in 0..4 {
            for j in 0..5 {
                a.set(i, j, Octet::new(rand::thread_rng().gen()));
            }
        }
        assert_eq!(a, &identity * &a);

        let (dense1, sparse1) = rand_dense_and_sparse(8);
        let (dense2, sparse2) = rand_dense_and_sparse(8);
        assert_matrices_eq(&(&dense1 * &dense2), &(&sparse1 * &sparse2));
    }

    #[test]
    fn sparse_mul() {
        let identity = sparse_identity(4);
        let mut a = SparseOctetMatrix::new(4, 5);
        for i in 0..4 {
            for j in 0..5 {
                a.set(i, j, Octet::new(rand::thread_rng().gen()));
            }
        }
        assert_eq!(a, &identity * &a);
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
