extern crate rand;
extern crate raptorq;

use rand::Rng;
use raptorq::SourceBlockEncoder;


fn main() {
    let elements = 10*1024;
    let symbol_size = 512;
    let mut data: Vec<u8> = vec![0; elements];
    for i in 0..elements {
        data[i] = rand::thread_rng().gen();
    }

    let mut junk = 0;
    for _ in 0..5000 {
        let encoder = SourceBlockEncoder::new(1, symbol_size, &data);
        let packets = encoder.repair_packets(0, 1);
        junk += packets[0].data()[0] as u32;
    }
    println!("{}", junk);
}
