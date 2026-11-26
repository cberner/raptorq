pub fn add_assign_binary(dest: &mut [u64], src: &[u64]) {
    debug_assert_eq!(dest.len(), src.len());

    for (dest, &src) in dest.iter_mut().zip(src) {
        // Addition over GF(2) is defined as XOR
        *dest ^= src;
    }
}
