use crate::iterators::{BorrowedKeyIter, OctetIter};
use crate::matrix::OctetMatrix;
use crate::octet::Octet;
use crate::octets::fused_addassign_mul_scalar;
use crate::octets::{add_assign, mulassign_scalar};
use crate::sparse_vec::{SparseOctetVec, SparseValuelessVec};
use crate::util::get_both_indices;
use std::cmp::min;

// Stores a matrix in sparse representation, with an optional dense block for the right most columns
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct SparseOctetMatrix {
    height: usize,
    width: usize,
    sparse_elements: Vec<SparseOctetVec>,
    // Note these are stored with the right-most element first in the vec.
    // That is, for a matrix with width 10 and num_dense 3, the last three will be stored in these
    // Vecs, and will be in the order: [9, 8, 7]
    // There may be extra zeros padded at the end too, for efficiency
    dense_elements: Vec<Vec<u8>>,
    // Sparse vector indicating which rows may have a non-zero value in the given column
    // Does not guarantee that the row has a non-zero value, since FMA may have added to zero
    sparse_column_index: Vec<SparseValuelessVec>,
    // Mapping of logical row numbers to index in sparse_elements, dense_elements, and sparse_column_index
    logical_row_to_physical: Vec<usize>,
    physical_row_to_logical: Vec<usize>,
    logical_col_to_physical: Vec<usize>,
    physical_col_to_logical: Vec<usize>,
    column_index_disabled: bool,
    num_dense_columns: usize,
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
                    debug_assert!(self.sparse_column_index[*col].exists(row));
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
        #[allow(clippy::needless_range_loop)]
        for i in 0..height {
            elements.push(SparseOctetVec::with_capacity(10));
            row_mapping[i] = i;
        }
        let mut dense_elements = Vec::with_capacity(height);
        for _ in 0..height {
            dense_elements.push(vec![0; 2 * trailing_dense_column_hint]);
        }
        let mut column_index = Vec::with_capacity(width);
        #[allow(clippy::needless_range_loop)]
        for i in 0..width {
            column_index.push(SparseValuelessVec::with_capacity(10));
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
            num_dense_columns: trailing_dense_column_hint,
        }
    }

    fn set(&mut self, i: usize, j: usize, value: Octet) {
        let physical_i = self.logical_row_to_physical[i];
        let physical_j = self.logical_col_to_physical[j];
        if self.width - j <= self.num_dense_columns {
            self.dense_elements[physical_i][self.width - j - 1] = value.byte();
        } else {
            self.sparse_elements[physical_i].insert(physical_j, value);
        }
        if !self.column_index_disabled {
            self.sparse_column_index[physical_j].insert(physical_i);
        }
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
        mulassign_scalar(
            &mut self.dense_elements[physical_row][..self.num_dense_columns],
            value,
        );
    }

    fn get(&self, i: usize, j: usize) -> Octet {
        let physical_i = self.logical_row_to_physical[i];
        let physical_j = self.logical_col_to_physical[j];
        if self.width - j <= self.num_dense_columns {
            return Octet::new(self.dense_elements[physical_i][self.width - j - 1]);
        } else {
            return self.sparse_elements[physical_i]
                .get(physical_j)
                .unwrap_or(&Octet::zero())
                .clone();
        }
    }

    fn get_row_iter(&self, row: usize, start_col: usize, end_col: usize) -> OctetIter {
        if end_col > self.width - self.num_dense_columns {
            unimplemented!("It was assumed that this wouldn't be needed, because the method would only be called on the V section of matrix A");
        }
        let physical_row = self.logical_row_to_physical[row];
        let sparse_elements = &self.sparse_elements[physical_row];
        OctetIter::new_sparse(
            start_col,
            end_col,
            sparse_elements,
            &self.physical_col_to_logical,
        )
    }

    fn get_col_index_iter(&self, col: usize, start_row: usize, end_row: usize) -> BorrowedKeyIter {
        let physical_col = self.logical_col_to_physical[col];
        BorrowedKeyIter::new_sparse(
            &self.sparse_column_index[physical_col],
            start_row,
            end_row,
            &self.physical_row_to_logical,
        )
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
        assert_eq!(
            self.width - self.num_dense_columns - 1,
            i,
            "Can only freeze the last sparse column"
        );
        self.num_dense_columns += 1;
        for i in 0..self.dense_elements.len() {
            if self.dense_elements[i].len() < self.num_dense_columns {
                // Add 10 more zeros at a time to amortize the cost
                self.dense_elements[i].extend_from_slice(&[0; 10]);
            }
        }
        let mut physical_row = 0;
        let physical_i = self.logical_col_to_physical[i];
        for maybe_present_in_row in self.sparse_column_index[physical_i].keys() {
            while physical_row < *maybe_present_in_row {
                physical_row += 1;
            }
            if let Some(value) = self.sparse_elements[physical_row].remove(physical_i) {
                self.dense_elements[physical_row][self.num_dense_columns - 1] = value.byte();
            }
            physical_row += 1;
        }
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
                        add_assign(
                            &mut temp_dense[row],
                            &self.dense_elements[physical_i][..self.num_dense_columns],
                        );
                    } else {
                        fused_addassign_mul_scalar(
                            &mut temp_dense[row],
                            &self.dense_elements[physical_i][..self.num_dense_columns],
                            &scalar,
                        );
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
                    self.sparse_column_index[*col].insert(physical_row)
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
        let (dest_row, temp_row) = get_both_indices(
            &mut self.dense_elements,
            physical_dest,
            physical_multiplicand,
        );

        if *scalar == Octet::one() {
            add_assign(
                &mut dest_row[..self.num_dense_columns],
                &temp_row[..self.num_dense_columns],
            );
        } else {
            fused_addassign_mul_scalar(
                &mut dest_row[..self.num_dense_columns],
                &temp_row[..self.num_dense_columns],
                scalar,
            );
        }

        // Then the sparse columns
        let (dest_row, temp_row) = get_both_indices(
            &mut self.sparse_elements,
            physical_dest,
            physical_multiplicand,
        );

        let new_columns = dest_row.fma(temp_row, scalar);
        if !self.column_index_disabled {
            for new_col in new_columns {
                self.sparse_column_index[new_col].insert(physical_dest);
            }
        }

        #[cfg(debug_assertions)]
        self.verify();
    }

    fn resize(&mut self, new_height: usize, new_width: usize) {
        assert!(new_height <= self.height);
        assert!(new_width <= self.width);
        if !self.column_index_disabled {
            unimplemented!(
                "Resize should only be used in phase 2, after column indexing is no longer needed"
            );
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
                self.sparse_elements[row]
                    .retain(|(col, _)| physical_to_logical[*col] < sparse_width);
            }
        }
        self.num_dense_columns -= dense_columns_to_remove;

        self.height = new_height;
        self.width = new_width;

        #[cfg(debug_assertions)]
        self.verify();
    }
}
