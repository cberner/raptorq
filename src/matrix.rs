use std::ops::Mul;
use octet::Octet;
use octets::fused_addassign_mul_scalar;
use octets::add_assign;
use util::get_both_indices;

#[derive(Clone, Debug, PartialEq)]
pub struct OctetMatrix {
    height: usize,
    width: usize,
    elements: Vec<Vec<u8>>
}

impl OctetMatrix {
    pub fn new(height: usize, width: usize) -> OctetMatrix {
        let mut elements: Vec<Vec<u8>> = vec![];
        for _ in 0..height {
            elements.push(vec![0; width]);
        }
        OctetMatrix {
            height,
            width,
            elements
        }
    }

    pub fn set(&mut self, i: usize, j: usize, value: Octet) {
        self.elements[i][j] = value.byte();
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn get_row(&self, i: usize) -> &Vec<u8> {
        &self.elements[i]
    }

    pub fn get(&self, i: usize, j: usize) -> Octet {
        Octet::new(self.elements[i][j])
    }

    pub fn swap_rows(&mut self, i: usize, j:usize) {
        self.elements.swap(i, j);
    }

    pub fn swap_columns(&mut self, i: usize, j:usize) {
        for row in 0..self.elements.len() {
            self.elements[row].swap(i, j);
        }
    }

    // other must be a rows x rows matrix
    // sets self[0..rows][..] = X * self[0..rows][..]
    pub fn mul_assign_submatrix(&mut self, other: &OctetMatrix, rows: usize) {
        assert_eq!(rows, other.height());
        assert_eq!(rows, other.width());
        assert!(rows <= self.height());
        let temp = self.clone();
        for row in 0..rows {
            let mut elements = vec![0; self.width];
            for i in 0..rows {
                let scalar = other.get(row, i);
                if scalar == Octet::zero() {
                    continue;
                }
                if scalar == Octet::one() {
                    add_assign(&mut elements, &temp.elements[i]);
                }
                else {
                    fused_addassign_mul_scalar(&mut elements, &temp.elements[i], &scalar);
                }
            }
            self.elements[row] = elements;
        }
    }

    pub fn fma_rows(&mut self, dest: usize, multiplicand: usize, scalar: &Octet) {
        assert_ne!(dest, multiplicand);
        let (dest_row, temp_row) = get_both_indices(&mut self.elements, dest, multiplicand);

        if *scalar == Octet::one() {
            add_assign(dest_row, temp_row);
        }
        else {
            fused_addassign_mul_scalar(dest_row, temp_row, scalar);
        }
    }

    pub fn resize(&mut self, new_height: usize, new_width: usize) {
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

impl<'a, 'b> Mul<&'b OctetMatrix> for &'a OctetMatrix {
    type Output = OctetMatrix;

    fn mul(self, rhs: &'b OctetMatrix) -> OctetMatrix {
        assert_eq!(self.width, rhs.height);
        let mut result = OctetMatrix::new(self.height, rhs.width);
        for row in 0..self.height {
            for i in 0..self.width {
                let scalar = self.get(row, i);
                if scalar == Octet::zero() {
                    continue;
                }
                if scalar == Octet::one() {
                    add_assign(&mut result.elements[row], &rhs.elements[i]);
                }
                else {
                    fused_addassign_mul_scalar(&mut result.elements[row], &rhs.elements[i], &scalar);
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use matrix::tests::rand::Rng;
    use matrix::OctetMatrix;
    use octet::Octet;

    fn identity(size: usize) -> OctetMatrix {
        let mut result = OctetMatrix::new(size, size);
        for i in 0..size {
            result.set(i, i, Octet::one());
        }
        result
    }

    #[test]
    fn mul() {
        let identity = identity(4);
        let mut a = OctetMatrix::new(4, 5);
        for i in 0..4 {
            for j in 0..5 {
                a.set(i, j, Octet::new(rand::thread_rng().gen()));
            }
        }
        assert_eq!(a, &identity * &a);
    }
}
