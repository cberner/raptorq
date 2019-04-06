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

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::matrix::OctetMatrix;
    use crate::matrix::DenseOctetMatrix;
    use crate::octet::Octet;

    fn identity(size: usize) -> DenseOctetMatrix {
        let mut result = DenseOctetMatrix::new(size, size);
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
}
