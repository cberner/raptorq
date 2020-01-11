use crate::iterators::{BorrowedKeyIter, OctetIter};
use crate::octet::Octet;
use crate::octets::fused_addassign_mul_scalar;
use crate::octets::{add_assign, count_ones_and_nonzeros, mulassign_scalar};
use crate::util::get_both_indices;
use serde::{Deserialize, Serialize};

pub trait OctetMatrix: Clone {
    fn new(
        height: usize,
        width: usize,
        trailing_dense_column_hint: usize,
        start_dense_row_hint: usize,
        num_dense_rows_hint: usize,
    ) -> Self;

    fn set(&mut self, i: usize, j: usize, value: Octet);

    fn height(&self) -> usize;

    fn width(&self) -> usize;

    fn count_ones_and_nonzeros(
        &self,
        row: usize,
        start_col: usize,
        end_col: usize,
    ) -> (usize, usize);

    fn mul_assign_row(&mut self, row: usize, value: &Octet);

    // Once "impl Trait" is supported in traits, it would be better to return "impl Iterator<...>"
    fn get_row_iter(&self, row: usize, start_col: usize, end_col: usize) -> OctetIter;

    // An iterator over rows for the given col, that may have non-zero values
    fn get_col_index_iter(&self, col: usize, start_row: usize, end_row: usize) -> BorrowedKeyIter;

    // Get a slice of columns from a row
    fn get_sub_row_as_octets(&self, row: usize, start_col: usize) -> Vec<u8>;

    fn get(&self, i: usize, j: usize) -> Octet;

    fn swap_rows(&mut self, i: usize, j: usize);

    // start_row_hint indicates that all preceding rows don't need to be swapped, because they have
    // identical values
    fn swap_columns(&mut self, i: usize, j: usize, start_row_hint: usize);

    fn enable_column_acccess_acceleration(&mut self);

    // After calling this method swap_columns() and other column oriented methods, may be much slower
    fn disable_column_acccess_acceleration(&mut self);

    // Hints that the dense rows should be compacted because they likely have a large fraction zeros
    fn hint_compact_dense_rows(&mut self);

    // Hints that column i will not be swapped again, and is likely to become dense'ish
    fn hint_column_dense_and_frozen(&mut self, i: usize);

    // other must be a rows x rows matrix
    // sets self[0..rows][..] = X * self[0..rows][..]
    fn mul_assign_submatrix(&mut self, other: &Self, rows: usize);

    fn fma_rows(&mut self, dest: usize, multiplicand: usize, scalar: &Octet);

    fn resize(&mut self, new_height: usize, new_width: usize);
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Hash)]
pub struct DenseOctetMatrix {
    height: usize,
    width: usize,
    elements: Vec<Vec<u8>>,
}

impl DenseOctetMatrix {
    pub fn fma_sub_row(&mut self, row: usize, start_col: usize, scalar: &Octet, other: &[u8]) {
        if *scalar == Octet::one() {
            add_assign(
                &mut self.elements[row][start_col..(start_col + other.len())],
                other,
            );
        } else {
            fused_addassign_mul_scalar(
                &mut self.elements[row][start_col..(start_col + other.len())],
                other,
                scalar,
            );
        }
    }
}

impl OctetMatrix for DenseOctetMatrix {
    fn new(height: usize, width: usize, _: usize, _: usize, _: usize) -> DenseOctetMatrix {
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

    fn count_ones_and_nonzeros(
        &self,
        row: usize,
        start_col: usize,
        end_col: usize,
    ) -> (usize, usize) {
        count_ones_and_nonzeros(&self.elements[row][start_col..end_col])
    }

    fn mul_assign_row(&mut self, row: usize, value: &Octet) {
        mulassign_scalar(&mut self.elements[row], value);
    }

    fn get_row_iter(&self, row: usize, start_col: usize, end_col: usize) -> OctetIter {
        OctetIter::new_dense(start_col, end_col, &self.elements[row], start_col)
    }

    fn get_col_index_iter(&self, _: usize, start_row: usize, end_row: usize) -> BorrowedKeyIter {
        BorrowedKeyIter::new_dense(start_row, end_row)
    }

    fn get_sub_row_as_octets(&self, row: usize, start_col: usize) -> Vec<u8> {
        self.elements[row][start_col..].to_vec()
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

    fn enable_column_acccess_acceleration(&mut self) {
        // No-op
    }

    fn disable_column_acccess_acceleration(&mut self) {
        // No-op
    }

    fn hint_compact_dense_rows(&mut self) {
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
        #[allow(clippy::needless_range_loop)]
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

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::matrix::{DenseOctetMatrix, OctetMatrix};
    use crate::octet::Octet;
    use crate::sparse_matrix::SparseOctetMatrix;

    fn dense_identity(size: usize) -> DenseOctetMatrix {
        let mut result = DenseOctetMatrix::new(size, size, 0, 0, 0);
        for i in 0..size {
            result.set(i, i, Octet::one());
        }
        result
    }

    fn sparse_identity(size: usize, dense_rows: usize) -> SparseOctetMatrix {
        let mut result = SparseOctetMatrix::new(size, size, 0, 0, dense_rows);
        for i in 0..size {
            result.set(i, i, Octet::one());
        }
        result
    }

    fn rand_dense_and_sparse(
        size: usize,
        dense_rows: usize,
    ) -> (DenseOctetMatrix, SparseOctetMatrix) {
        let mut dense = DenseOctetMatrix::new(size, size, 0, 0, 0);
        let mut sparse = SparseOctetMatrix::new(size, size, 1, 0, dense_rows);
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
                assert_eq!(
                    matrix1.get(i, j),
                    matrix2.get(i, j),
                    "Matrices are not equal at row={} col={}",
                    i,
                    j
                );
            }
        }
    }

    #[test]
    fn swap_rows() {
        // rand_dense_and_sparse uses set(), so just check that it works
        let (mut dense, mut sparse) = rand_dense_and_sparse(8, 3);
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
        let (mut dense, mut sparse) = rand_dense_and_sparse(8, 3);
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
        let (dense, sparse) = rand_dense_and_sparse(8, 3);
        assert_eq!(
            dense.count_ones_and_nonzeros(0, 0, 5),
            sparse.count_ones_and_nonzeros(0, 0, 5)
        );
        assert_eq!(
            dense.count_ones_and_nonzeros(2, 2, 6),
            sparse.count_ones_and_nonzeros(2, 2, 6)
        );
        assert_eq!(
            dense.count_ones_and_nonzeros(3, 1, 2),
            sparse.count_ones_and_nonzeros(3, 1, 2)
        );
    }

    #[test]
    fn mul_assign_row() {
        // rand_dense_and_sparse uses set(), so just check that it works
        let (mut dense, mut sparse) = rand_dense_and_sparse(8, 3);
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
        let (mut dense, mut sparse) = rand_dense_and_sparse(8, 0);
        let original = dense.clone();

        let identity = dense_identity(5);
        dense.mul_assign_submatrix(&identity, 5);
        assert_matrices_eq(&dense, &original);

        let identity = dense_identity(8);
        dense.mul_assign_submatrix(&identity, 8);
        assert_matrices_eq(&dense, &original);

        let identity = sparse_identity(5, 0);
        sparse.mul_assign_submatrix(&identity, 5);
        assert_matrices_eq(&sparse, &original);

        let identity = sparse_identity(8, 0);
        sparse.mul_assign_submatrix(&identity, 8);
        assert_matrices_eq(&sparse, &original);
    }

    #[test]
    fn fma_rows() {
        // rand_dense_and_sparse uses set(), so just check that it works
        let (mut dense, mut sparse) = rand_dense_and_sparse(8, 1);
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
        let (mut dense, mut sparse) = rand_dense_and_sparse(8, 0);
        dense.disable_column_acccess_acceleration();
        sparse.disable_column_acccess_acceleration();
        dense.resize(5, 5);
        sparse.resize(5, 5);
        assert_matrices_eq(&dense, &sparse);
    }

    #[test]
    fn hint_column_dense_and_frozen() {
        // rand_dense_and_sparse uses set(), so just check that it works
        let (dense, mut sparse) = rand_dense_and_sparse(8, 3);
        sparse.enable_column_acccess_acceleration();
        sparse.hint_column_dense_and_frozen(6);
        sparse.hint_column_dense_and_frozen(5);
        assert_matrices_eq(&dense, &sparse);
    }
}
