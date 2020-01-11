use crate::iterators::{BorrowedKeyIter, OctetIter};
use crate::matrix::BinaryMatrix;
use crate::octet::Octet;
use crate::octets::add_assign;
use crate::sparse_vec::{SparseBinaryVec, SparseValuelessVec};
use crate::util::get_both_indices;
use serde::{Deserialize, Serialize};
use std::cmp::min;

// Stores a matrix in sparse representation, with an optional dense block for the right most columns
// The logical storage is as follows:
// |---------------------------------------|
// |                          | (optional) |
// |      sparse rows         | dense      |
// |                          | columns    |
// |---------------------------------------|
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Hash)]
pub struct SparseBinaryMatrix {
    height: usize,
    width: usize,
    sparse_elements: Vec<SparseBinaryVec>,
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

impl SparseBinaryMatrix {
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

impl BinaryMatrix for SparseBinaryMatrix {
    fn new(height: usize, width: usize, trailing_dense_column_hint: usize) -> SparseBinaryMatrix {
        let mut col_mapping = vec![0; width];
        let elements = vec![SparseBinaryVec::with_capacity(10); height];
        let mut row_mapping = vec![0; height];
        #[allow(clippy::needless_range_loop)]
        for i in 0..height {
            row_mapping[i] = i;
        }
        let mut dense_elements = Vec::with_capacity(height);
        for _ in 0..height {
            dense_elements.push(vec![0; 2 * trailing_dense_column_hint]);
        }
        #[allow(clippy::needless_range_loop)]
        for i in 0..width {
            col_mapping[i] = i;
        }
        SparseBinaryMatrix {
            height,
            width,
            sparse_elements: elements,
            dense_elements,
            sparse_column_index: vec![],
            logical_row_to_physical: row_mapping.clone(),
            physical_row_to_logical: row_mapping,
            logical_col_to_physical: col_mapping.clone(),
            physical_col_to_logical: col_mapping,
            column_index_disabled: true,
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
            if !self.column_index_disabled {
                self.sparse_column_index[physical_j].insert(physical_i);
            }
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

    fn get_sub_row_as_octets(&self, row: usize, start_col: usize) -> Vec<u8> {
        let first_dense_column = self.width - self.num_dense_columns;
        assert!(start_col >= self.width - self.num_dense_columns);
        let physical_row = self.logical_row_to_physical[row];
        let mut result = self.dense_elements[physical_row]
            [(start_col - first_dense_column)..(self.width - first_dense_column)]
            .to_vec();
        result.reverse();
        return result;
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
        assert_eq!(self.column_index_disabled, false);
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

        let physical_i = self.logical_col_to_physical[i];
        let physical_j = self.logical_col_to_physical[j];
        self.logical_col_to_physical.swap(i, j);
        self.physical_col_to_logical.swap(physical_i, physical_j);
    }

    fn enable_column_acccess_acceleration(&mut self) {
        self.column_index_disabled = false;
        self.sparse_column_index = vec![SparseValuelessVec::with_capacity(50); self.width];
        for (physical_row, elements) in self.sparse_elements.iter().enumerate() {
            for (physical_col, _) in elements.keys_values() {
                self.sparse_column_index[*physical_col].insert_last(physical_row);
            }
        }
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
        assert_eq!(self.column_index_disabled, false);
        self.num_dense_columns += 1;
        for i in 0..self.dense_elements.len() {
            if self.dense_elements[i].len() < self.num_dense_columns {
                // Add 10 more zeros at a time to amortize the cost
                self.dense_elements[i].extend_from_slice(&[0; 10]);
            }
        }
        let physical_i = self.logical_col_to_physical[i];
        for maybe_present_in_row in self.sparse_column_index[physical_i].keys() {
            let physical_row = *maybe_present_in_row;
            if let Some(value) = self.sparse_elements[physical_row].remove(physical_i) {
                self.dense_elements[physical_row][self.num_dense_columns - 1] = value.byte();
            }
        }
    }

    // other must be a rows x rows matrix
    // sets self[0..rows][..] = X * self[0..rows][..]
    fn mul_assign_submatrix(&mut self, other: &SparseBinaryMatrix, rows: usize) {
        assert_eq!(rows, other.height());
        assert_eq!(rows, other.width());
        assert!(rows <= self.height());
        if other.num_dense_columns != 0 {
            unimplemented!();
        }
        // Note: rows are logically indexed
        let mut temp_sparse = vec![SparseBinaryVec::with_capacity(10); rows];
        let mut temp_dense = vec![vec![0; self.num_dense_columns]; rows];
        for row in 0..rows {
            for (i, scalar) in other.get_row_iter(row, 0, rows) {
                let physical_i = self.logical_row_to_physical[i];
                if scalar != Octet::zero() {
                    temp_sparse[row].add_assign(&self.sparse_elements[physical_i]);
                    add_assign(
                        &mut temp_dense[row],
                        &self.dense_elements[physical_i][..self.num_dense_columns],
                    );
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

    fn add_assign_rows(&mut self, dest: usize, src: usize) {
        assert_ne!(dest, src);
        let physical_dest = self.logical_row_to_physical[dest];
        let physical_multiplicand = self.logical_row_to_physical[src];
        // First handle the dense columns
        let (dest_row, temp_row) = get_both_indices(
            &mut self.dense_elements,
            physical_dest,
            physical_multiplicand,
        );

        add_assign(
            &mut dest_row[..self.num_dense_columns],
            &temp_row[..self.num_dense_columns],
        );

        // Then the sparse columns
        let (dest_row, temp_row) = get_both_indices(
            &mut self.sparse_elements,
            physical_dest,
            physical_multiplicand,
        );

        let new_columns = dest_row.add_assign(temp_row);
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
            if logical_row < new_height {
                new_sparse[logical_row] = sparse;
            }
        }

        for i in (0..self.dense_elements.len()).rev() {
            let logical_row = self.physical_row_to_logical[i];
            let dense = self.dense_elements.pop();
            if logical_row < new_height {
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
