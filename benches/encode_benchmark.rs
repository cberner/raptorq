use rand::Rng;
use raptorq::SourceBlockEncoder;
use std::time::Instant;

const TARGET_TOTAL_BYTES: usize = 100 * 1024 * 1024;
const SYMBOL_COUNTS: [usize; 11] = [10, 100, 250, 500, 1000, 2000, 4000, 10000, 20000, 40000, 56403];

fn main() {
    let mut junk = 0;
    for symbol_count in SYMBOL_COUNTS.iter() {
        let elements = symbol_count * 512;
        let symbol_size = 512;
        let mut data: Vec<u8> = vec![0; elements];
        for i in 0..elements {
            data[i] = rand::thread_rng().gen();
        }

        let now = Instant::now();
        let iterations = TARGET_TOTAL_BYTES / elements;
        for _ in 0..iterations {
            let encoder = SourceBlockEncoder::new(1, symbol_size, &data);
            let packets = encoder.repair_packets(0, 1);
            junk += packets[0].data()[0] as u32;
        }
        let elapsed = now.elapsed();
        let elapsed = elapsed.as_secs() as f64 + elapsed.subsec_millis() as f64 * 0.001;
        let throughput = (elements * iterations * 8) as f64 / 1024.0 / 1024.0 / elapsed;
        println!("symbol count = {}, encoded {} MB in {:.3}secs, throughput: {:.1}Mbit/s",
                 symbol_count,
                 elements * iterations / 1024 / 1024,
                 elapsed,
                 throughput);
    }
    println!("{}", junk);
}
