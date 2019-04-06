use crate::octet::Octet;
use crate::octets::{add_assign, mulassign_scalar, count_ones_and_nonzeros};
use crate::octets::fused_addassign_mul_scalar;
use crate::util::get_both_indices;
use std::ops::Mul;
use std::collections::HashMap;

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
    elements: HashMap<usize, Octet>
}

impl SparseOctetVec {
    pub fn with_capacity(capacity: usize) -> SparseOctetVec {
        SparseOctetVec {
            elements: HashMap::with_capacity(capacity)
        }
    }

    pub fn fma(&mut self, other: &SparseOctetVec, scalar: &Octet) {
        for (col, value) in other.elements.iter() {
            let optional_dest_value = self.elements.get_mut(&col);
            if let Some(dest_value) = optional_dest_value {
                dest_value.fma(value, scalar);
            }
            else {
                self.elements.insert(*col, value * scalar);
            }
        }
    }

    pub fn truncate(&mut self, new_length: usize) {
        let mut to_remove = Vec::with_capacity(self.elements.len());
        for col in self.elements.keys() {
            if *col >= new_length {
                to_remove.push(*col);
            }
        }
        for col in to_remove {
            self.elements.remove(&col);
        }
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        let i_value = self.elements.remove(&i);
        let j_value = self.elements.remove(&j);
        if let Some(value) = i_value {
            self.elements.insert(j, value);
        }
        if let Some(value) = j_value {
            self.elements.insert(i, value);
        }
    }

    pub fn get(&self, i: &usize) -> Option<&Octet> {
        self.elements.get(i)
    }

    pub fn mul_assign(&mut self, scalar: &Octet) {
        for entry in self.elements.values_mut() {
            *entry = entry as &Octet * scalar;
        }
    }

    pub fn keys_values(&self) -> impl Iterator<Item=(&usize, &Octet)> {
        self.elements.iter()
    }

    pub fn insert(&mut self, i: usize, value: Octet) {
        self.elements.insert(i, value);
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

    use crate::matrix::{OctetMatrix, SparseOctetMatrix};
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
}
