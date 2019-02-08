use octet::Octet;
use std::ops::AddAssign;
use octets::mulassign_scalar;
use octets::fused_addassign_mul_scalar;
use octets::add_assign;

#[derive(Clone, Debug, PartialEq)]
pub struct Symbol {
    value: Vec<u8>
}

impl Symbol {
    pub fn new(value: Vec<u8>) -> Symbol {
        Symbol {
            value
        }
    }

    pub fn zero(size: usize) -> Symbol {
        Symbol {
            value: vec![0; size]
        }
    }

    pub fn len(&self) -> usize {
        self.value.len()
    }

    pub fn bytes(&self) -> Vec<u8> {
        self.value.clone()
    }

    pub fn mulassign_scalar(&mut self, scalar: &Octet) {
        mulassign_scalar(&mut self.value, scalar);
    }

    pub fn fused_addassign_mul_scalar(&mut self, other: &Symbol, scalar: &Octet) {
        fused_addassign_mul_scalar(&mut self.value, &other.value, scalar);
    }
}

impl<'a> AddAssign<&'a Symbol> for Symbol {
    fn add_assign(&mut self, other: &'a Symbol) {
        add_assign(&mut self.value, &other.value);
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use symbol::tests::rand::Rng;
    use symbol::Symbol;

    #[test]
    fn add_assign() {
        let symbol_size = 41;
        let mut data1: Vec<u8> = vec![0; symbol_size];
        let mut data2: Vec<u8> = vec![0; symbol_size];
        let mut result: Vec<u8> = vec![0; symbol_size];
        for i in 0..symbol_size {
            data1[i] = rand::thread_rng().gen();
            data2[i] = rand::thread_rng().gen();
            result[i] = data1[i] ^ data2[i];
        }
        let mut symbol1 = Symbol::new(data1);
        let symbol2 = Symbol::new(data2);

        symbol1 += &symbol2;
        assert_eq!(result, symbol1.bytes());
    }
}
