pub fn add_assign_binary(dest: &mut [u64], src: &[u64]) {
    for i in 0..dest.len() {
        // Addition over GF(2) is defined as XOR
        dest[i] ^= src[i];
    }
}
