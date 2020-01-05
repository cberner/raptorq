use crate::iterators::{BorrowedKeyIter, OctetIter};
use crate::matrix::OctetMatrix;
use crate::octet::Octet;
use crate::octets::fused_addassign_mul_scalar;
use crate::octets::{add_assign, mulassign_scalar};
use crate::sparse_vec::{SparseOctetVec, SparseValuelessVec};
use crate::util::get_both_indices;
use std::cmp::min;

// Stores a matrix in sparse representation, with an optional dense block for the right most columns,
// and optional dense rows.
// The logical storage is as follows:
// |---------------------------------------|
// |  sparse rows             | (optional) |
// |--------------------------| dense      |
// |  (optional) dense rows   | columns    |
// |---------------------------------------|
// The physical ids of rows are, and dense columns are read first before the rows:
// |--------------------------|
// |  sparse rows             |
// |--------------------------|
// |  (optional) dense rows   |
// |--------------------------|
#[derive(Clone, Debug, PartialEq)]
pub struct SparseOctetMatrix {
    height: usize,
    width: usize,
    sparse_elements: Vec<SparseOctetVec>,
    // Optional dense rows. These have physical indices starting at sparse_elements.len()
    dense_rows: Vec<Vec<u8>>,
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
        for row in 0..self.sparse_elements.len() {
            for (col, value) in self.sparse_elements[row].keys_values() {
                if *value != Octet::zero() {
                    debug_assert!(self.sparse_column_index[*col].exists(row));
                }
            }
        }
        for i in 0..self.dense_rows.len() {
            for (col, value) in self.dense_rows[i].iter().enumerate() {
                if Octet::new(*value) != Octet::zero() {
                    debug_assert!(
                        self.sparse_column_index[col].exists(i + self.sparse_elements.len())
                    );
                }
            }
        }
    }
}

impl OctetMatrix for SparseOctetMatrix {
    fn new(
        height: usize,
        width: usize,
        trailing_dense_column_hint: usize,
        start_dense_row_hint: usize,
        num_dense_rows_hint: usize,
    ) -> SparseOctetMatrix {
        let mut col_mapping = vec![0; width];
        let elements = vec![SparseOctetVec::with_capacity(10); height - num_dense_rows_hint];
        let mut dense_rows = Vec::with_capacity(num_dense_rows_hint);
        for _ in 0..num_dense_rows_hint {
            dense_rows.push(vec![0; width - trailing_dense_column_hint]);
        }
        let mut logical_row_to_physical = vec![0; height];
        let mut physical_row_to_logical = vec![0; height];
        // HDPC rows are stored in dense format. They are in the middle (logically) in the matrix
        for i in 0..start_dense_row_hint {
            logical_row_to_physical[i] = i;
            physical_row_to_logical[i] = i;
        }
        for i in start_dense_row_hint..(start_dense_row_hint + num_dense_rows_hint) {
            logical_row_to_physical[i] = i - start_dense_row_hint + elements.len();
            physical_row_to_logical[i - start_dense_row_hint + elements.len()] = i;
        }
        for i in (start_dense_row_hint + num_dense_rows_hint)..height {
            logical_row_to_physical[i] = i - num_dense_rows_hint;
            physical_row_to_logical[i - num_dense_rows_hint] = i;
        }
        let mut dense_elements = Vec::with_capacity(height);
        for _ in 0..height {
            dense_elements.push(vec![0; 2 * trailing_dense_column_hint]);
        }
        let mut column_index = Vec::with_capacity(width);
        #[allow(clippy::needless_range_loop)]
        for i in 0..width {
            column_index.push(SparseValuelessVec::with_capacity(50));
            col_mapping[i] = i;
        }
        SparseOctetMatrix {
            height,
            width,
            sparse_elements: elements,
            dense_rows,
            dense_elements,
            sparse_column_index: column_index,
            logical_row_to_physical,
            physical_row_to_logical,
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
        } else if physical_i >= self.sparse_elements.len() {
            self.dense_rows[physical_i - self.sparse_elements.len()][physical_j] = value.byte();
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
        if physical_row >= self.sparse_elements.len() {
            for (physical_col, value) in self.dense_rows[physical_row - self.sparse_elements.len()]
                .iter()
                .enumerate()
            {
                let value = Octet::new(*value);
                let col = self.physical_col_to_logical[physical_col];
                if col >= start_col && col < end_col {
                    if value == Octet::one() {
                        ones += 1;
                    }
                    if value != Octet::zero() {
                        nonzeros += 1;
                    }
                }
            }
        } else {
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
        }
        return (ones, nonzeros);
    }

    fn mul_assign_row(&mut self, row: usize, value: &Octet) {
        let physical_row = self.logical_row_to_physical[row];
        if physical_row >= self.sparse_elements.len() {
            mulassign_scalar(
                &mut self.dense_rows[physical_row - self.sparse_elements.len()],
                value,
            );
        } else {
            self.sparse_elements[physical_row].mul_assign(value);
        }
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
        } else if physical_i >= self.sparse_elements.len() {
            return Octet::new(
                self.dense_rows[physical_i - self.sparse_elements.len()][physical_j],
            );
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
        if physical_row >= self.sparse_elements.len() {
            todo!("Handle dense rows");
        }
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

    fn hint_compact_dense_rows(&mut self) {
        for row in self.dense_rows.drain(..) {
            let mut sparse = SparseOctetVec::with_capacity(10);
            for (physical_col, value) in row.iter().enumerate() {
                let value = Octet::new(*value);
                if value != Octet::zero() {
                    sparse.insert(physical_col, value);
                }
            }
            self.sparse_elements.push(sparse);
        }
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
        let physical_i = self.logical_col_to_physical[i];
        for maybe_present_in_row in self.sparse_column_index[physical_i].keys() {
            let physical_row = *maybe_present_in_row;
            if physical_row >= self.sparse_elements.len() {
                // The value is left in dense_rows, since the physical col isn't removed from
                // row storage. The dense cols are consulted first when looking up a value.
                let value = self.dense_rows[physical_row - self.sparse_elements.len()][physical_i];
                self.dense_elements[physical_row][self.num_dense_columns - 1] = value;
                self.dense_rows[physical_row - self.sparse_elements.len()][physical_i] = 0;
            } else if let Some(value) = self.sparse_elements[physical_row].remove(physical_i) {
                self.dense_elements[physical_row][self.num_dense_columns - 1] = value.byte();
            }
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
        if !self.dense_rows.is_empty() {
            todo!();
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
        if physical_multiplicand >= self.sparse_elements.len() {
            todo!();
        }
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
        if physical_dest >= self.sparse_elements.len() {
            for (physical_col, multiplicand) in
                self.sparse_elements[physical_multiplicand].keys_values()
            {
                if *multiplicand != Octet::zero() {
                    let mut value = Octet::new(
                        self.dense_rows[physical_dest - self.sparse_elements.len()][*physical_col],
                    );
                    value.fma(multiplicand, scalar);
                    self.dense_rows[physical_dest - self.sparse_elements.len()][*physical_col] =
                        value.byte();

                    if !self.column_index_disabled && value != Octet::zero() {
                        self.sparse_column_index[*physical_col].insert(physical_dest);
                    }
                }
            }
        } else {
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
        }

        #[cfg(debug_assertions)]
        self.verify();
    }

    fn resize(&mut self, new_height: usize, new_width: usize) {
        assert!(new_height <= self.height);
        assert!(new_width <= self.width);
        if !self.column_index_disabled || !self.dense_rows.is_empty() {
            unimplemented!(
                "Resize should only be used in phase 2, after column indexing and dense rows are no longer needed"
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
