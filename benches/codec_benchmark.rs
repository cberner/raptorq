#[macro_use]
extern crate criterion;
extern crate rand;
extern crate raptorq;

use criterion::Criterion;
use rand::Rng;
use raptorq::SourceBlockEncoder;
use raptorq::SourceBlockDecoder;


fn criterion_benchmark(c: &mut Criterion) {
    let elements = 10*1024;
    let symbol_size = 512;
    let mut data: Vec<u8> = vec![0; elements];
    for i in 0..elements {
        data[i] = rand::thread_rng().gen();
    }

    let encode_data = data.clone();
    c.bench_function("encode 10KB", move |b| b.iter(|| {
        let encoder = SourceBlockEncoder::new(1, symbol_size, encode_data.clone());
        return encoder.all_source_packets();
    }));

    let roundtrip_data = data.clone();
    c.bench_function("roundtrip 10KB", move |b| b.iter(|| {
        let encoder = SourceBlockEncoder::new(1, symbol_size, roundtrip_data.clone());
        let mut decoder = SourceBlockDecoder::new(1, symbol_size, elements as u64);
        let mut result = None;
        for packet in encoder.all_source_packets() {
            result = decoder.parse(packet);
        }
        return result
    }));

    let repair_data = data.clone();
    c.bench_function("roundtrip repair 10KB", move |b| b.iter(|| {
        let encoder = SourceBlockEncoder::new(1, symbol_size, repair_data.clone());
        let mut decoder = SourceBlockDecoder::new(1, symbol_size, elements as u64);
        let mut result = None;
        for packet in encoder.repair_packets(0, (elements / symbol_size as usize) as u32) {
            result = decoder.parse(packet);
        }
        return result
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
