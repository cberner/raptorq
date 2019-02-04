#[macro_use]
extern crate criterion;
extern crate rand;
extern crate raptorq;

use criterion::Criterion;
use rand::Rng;
use raptorq::SourceBlockEncoder;
use raptorq::SourceBlockDecoder;
use raptorq::Octet;


fn criterion_benchmark(c: &mut Criterion) {
    let octet1 = Octet::from(rand::thread_rng().gen_range(1, 255));
    let octet2 = Octet::from(rand::thread_rng().gen_range(1, 255));

    let octet1_add = octet1.clone();
    let octet2_add = octet2.clone();
    c.bench_function("Octet add", move |b| b.iter(|| {
        octet1_add.clone() + octet2_add.clone()
    }));

    let octet1_add_ref = octet1.clone();
    let octet2_add_ref = octet2.clone();
    c.bench_function("Octet add by ref", move |b| b.iter(|| {
        &octet1_add_ref + &octet2_add_ref
    }));

    let octet1_mul = octet1.clone();
    let octet2_mul = octet2.clone();
    c.bench_function("Octet mul", move |b| b.iter(|| {
        octet1_mul.clone() * octet2_mul.clone()
    }));

    let octet1_mul_ref = octet1.clone();
    let octet2_mul_ref = octet2.clone();
    c.bench_function("Octet mul by ref", move |b| b.iter(|| {
        &octet1_mul_ref * &octet2_mul_ref
    }));


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
