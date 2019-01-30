use std::ops::Add;
use std::ops::Mul;
use std::ops::Div;
use octet::Octet;
use std::ops::AddAssign;

#[derive(Clone, Debug, PartialEq)]
pub struct Symbol {
    value: Vec<Octet>
}

impl Symbol {
    pub fn new(value: Vec<u8>) -> Symbol {
        Symbol {
            value: value.iter().map(|&x| Octet::from(x)).collect()
        }
    }

    pub fn zero(size: usize) -> Symbol {
        Symbol {
            value: vec![Octet::zero(); size]
        }
    }

    pub fn len(&self) -> usize {
        self.value.len()
    }

    pub fn bytes(&self) -> Vec<u8> {
        self.value.iter().map(|octet| octet.clone().into()).collect()
    }

    pub fn mul_scalar(&self, scalar: &Octet) -> Symbol {
        let mut result = Vec::with_capacity(self.value.len());
        for i in 0..self.value.len() {
            result.push(self.value[i].clone() * scalar.clone());
        }
        Symbol {
            value: result
        }
    }

    pub fn fused_addassign_mul_scalar(&mut self, other: &Symbol, scalar: &Octet) {
        for i in 0..self.value.len() {
            self.value[i] += &other.value[i] * &scalar;
        }
    }
}

impl Add for Symbol {
    type Output = Symbol;

    fn add(self, other: Symbol) -> Symbol {
        let mut result = Vec::with_capacity(self.value.len());
        for i in 0..self.value.len() {
            result.push(self.value[i].clone() + other.value[i].clone());
        }
        Symbol {
            value: result
        }
    }
}

impl AddAssign for Symbol {
    fn add_assign(&mut self, other: Symbol) {
        for i in 0..self.value.len() {
            self.value[i] += other.value[i].clone();
        }
    }
}

impl Mul for Symbol {
    type Output = Symbol;

    fn mul(self, other: Symbol) -> Symbol {
        let mut result = Vec::with_capacity(self.value.len());
        for i in 0..self.value.len() {
            result.push(self.value[i].clone() * other.value[i].clone())
        }
        Symbol {
            value: result
        }
    }
}

impl Div for Symbol {
    type Output = Symbol;

    fn div(self, rhs: Symbol) -> Symbol {
        let mut result = Vec::with_capacity(self.value.len());
        for i in 0..self.value.len() {
            result.push(self.value[i].clone() / rhs.value[i].clone())
        }
        Symbol {
            value: result
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use symbol::tests::rand::Rng;
    use symbol::Symbol;

    #[test]
    fn addition() {
        let elements = 4;
        let mut data: Vec<u8> = vec![0; elements];
        for i in 0..elements {
            data[i] = rand::thread_rng().gen();
        }
        let symbol = Symbol::new(data);
        let symbol2 = symbol.clone();
        // See section 5.7.2. u is its own additive inverse
        assert_eq!(Symbol::zero(elements), symbol + symbol2);
    }

    #[test]
    fn multiplication_identity() {
        let elements = 4;
        let mut data: Vec<u8> = vec![0; elements];
        for i in 0..elements {
            data[i] = rand::thread_rng().gen();
        }
        let symbol = Symbol::new(data);
        let one = Symbol::new(vec![1, 1, 1, 1]);
        assert_eq!(symbol, symbol.clone() * one);
    }

    #[test]
    fn multiplicative_inverse() {
        let elements = 4;
        let mut data: Vec<u8> = vec![0; elements];
        for i in 0..elements {
            data[i] = rand::thread_rng().gen();
        }
        let symbol = Symbol::new(data);
        let one = Symbol::new(vec![1, 1, 1, 1]);
        assert_eq!(one.clone(), symbol.clone() * (one.clone() / symbol.clone()));
    }

    #[test]
    fn division() {
        let elements = 4;
        let mut data: Vec<u8> = vec![0; elements];
        for i in 0..elements {
            data[i] = rand::thread_rng().gen();
        }
        let symbol = Symbol::new(data);
        let symbol2 = symbol.clone();
        let one = Symbol::new(vec![1, 1, 1, 1]);
        assert_eq!(one, symbol / symbol2);
    }
}
