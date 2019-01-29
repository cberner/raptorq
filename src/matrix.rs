use std::ops::Mul;
use octet::Octet;
use symbol::Symbol;

#[derive(Clone, Debug, PartialEq)]
pub struct OctetMatrix {
    height: usize,
    width: usize,
    elements: Vec<Vec<Octet>>
}

impl OctetMatrix {
    pub fn new(height: usize, width: usize) -> OctetMatrix {
        let mut elements: Vec<Vec<Octet>> = vec![];
        for _ in 0..height {
            elements.push(vec![0.into(); width]);
        }
        OctetMatrix {
            height,
            width,
            elements
        }
    }

    pub fn identity(size: usize) -> OctetMatrix {
        let mut result = OctetMatrix::new(size, size);
        for i in 0..size {
            result.set(i, i, 1);
        }
        result
    }

    pub fn mul_symbols(&self, symbols: &Vec<Symbol>) -> Vec<Symbol> {
        assert_eq!(self.width, symbols.len());
        assert_ne!(0, symbols.len());
        let mut result: Vec<Symbol> = vec![];
        for i in 0..self.height {
            let mut symbol = Symbol::zero(symbols[0].value.len());
            for j in 0..self.width {
                symbol += symbols[j].mul_scalar(&self.elements[i][j]);
            }
            result.push(symbol);
        }
        result
    }

    // TODO: can probably remove the parameter T, and just take an Octet
    pub fn set<T:Into<Octet>>(&mut self, i: usize, j: usize, value: T) {
        self.elements[i][j] = value.into();
    }

    pub fn get(&self, i: usize, j: usize) -> Octet {
        self.elements[i][j].clone()
    }

    pub fn inverse(&self) -> Option<OctetMatrix> {
        // Calculate inverse using Gauss-Jordan elimination

        // Only implemented for square matrices
        assert_eq!(self.height, self.width);

        // Extend with identity on right side
        let mut intermediate = self.elements.clone();
        for i in 0..self.height {
            for _ in 0..self.width {
                intermediate[i].push(0.into());
            }
            intermediate[i][self.width + i] = 1.into();
        }

        // Convert to row echelon form
        for i in 0..self.width {
            // Swap a row with leading coefficient i into place
            for j in i..self.height {
                if intermediate[j][i] != Octet::zero() {
                    intermediate.swap(i, j);
                    break;
                }
            }

            if intermediate[i][i] == Octet::zero() {
                // If all following rows are zero in this column, then matrix is singular
                return None
            }

            // Scale leading coefficient to 1
            if intermediate[i][i] != 1.into() {
                let element_inverse = Octet::from(1) / intermediate[i][i].clone();
                for j in i..(2*self.width) {
                    intermediate[i][j] = intermediate[i][j].clone() * element_inverse.clone();
                }
            }

            // Zero out all following elements in i'th column
            for j in (i + 1)..self.height {
                if intermediate[j][i] != Octet::zero() {
                    let scalar = intermediate[j][i].clone();
                    // Multiply and subtract i'th row from j'th row
                    for k in i..(2*self.width) {
                        intermediate[j][k] = intermediate[j][k].clone() - scalar.clone() * intermediate[i][k].clone();
                    }
                }
            }
        }

        // Perform backwards elimination
        for i in (0..self.width).rev() {
            // Zero out all preceding elements in i'th column
            for j in 0..i {
                if intermediate[j][i] != Octet::zero() {
                    let scalar = intermediate[j][i].clone();
                    // Multiply and subtract i'th row from j'th row
                    for k in i..(2*self.width) {
                        intermediate[j][k] = intermediate[j][k].clone() - scalar.clone() * intermediate[i][k].clone();
                    }
                }
            }
        }

        // Return inverse
        let mut result = OctetMatrix::new(self.height, self.width);
        for row in 0..self.height {
            for column in 0..self.width {
                result.set(row, column, intermediate[row][column + self.width].clone());
            }
        }

        Some(result)
    }
}

impl Mul for OctetMatrix {
    type Output = OctetMatrix;

    fn mul(self, rhs: OctetMatrix) -> OctetMatrix {
        assert_eq!(self.width, rhs.height);
        let mut result = OctetMatrix::new(self.height, rhs.width);
        for i in 0..self.height {
            for j in 0..rhs.width {
                let mut element = Octet::zero();
                for k in 0..self.width {
                    element = element + self.elements[i][k].clone() * rhs.elements[k][j].clone();
                }
                result.set(i, j, element);
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
    use symbol::Symbol;

    #[test]
    fn mul() {
        let identity = OctetMatrix::identity(4);
        let mut a = OctetMatrix::new(4, 5);
        for i in 0..4 {
            for j in 0..5 {
                a.set::<u8>(i, j, rand::thread_rng().gen());
            }
        }
        assert_eq!(a.clone(), identity * a.clone());
    }

    #[test]
    fn mul_symbol() {
        let identity = OctetMatrix::identity(4);
        let mut symbols: Vec<Symbol> = vec![];
        for _ in 0..4 {
            let mut data = vec![];
            for _ in 0..3 {
                data.push(rand::thread_rng().gen());
            }
            symbols.push(Symbol {
                value: data
            });
        }
        assert_eq!(symbols.clone(), identity.mul_symbols(&symbols));

        let mut a = OctetMatrix::new(4, 4);
        for i in 0..4 {
            for j in 0..4 {
                a.set::<u8>(i, j, rand::thread_rng().gen());
            }
        }
        // Statistically improbable that the random matrix, a, is the identity
        assert_ne!(symbols.clone(), a.mul_symbols(&symbols));
    }

    #[test]
    fn inverse() {
        let identity = OctetMatrix::identity(3);
        assert_eq!(identity, identity.clone() * identity.clone().inverse().unwrap());

        let mut a = OctetMatrix::new(3, 3);
        a.set(0, 0, 1);
        a.set(0, 1, 2);
        a.set(0, 2, 3);

        a.set(1, 0, 4);
        a.set(1, 1, 5);
        a.set(1, 2, 6);

        a.set(2, 0, 7);
        a.set(2, 1, 8);
        a.set(2, 2, 9);
        assert_eq!(identity, a.clone() * a.clone().inverse().unwrap());
    }
}
