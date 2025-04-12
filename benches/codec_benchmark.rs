use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::Throughput;

use rand::Rng;
use raptorq::SourceBlockDecoder;
use raptorq::SourceBlockEncoder;
use raptorq::Symbol;
use raptorq::{ObjectTransmissionInformation, Octet};

fn criterion_benchmark(c: &mut Criterion) {
    let octet1 = Octet::new(rand::rng().random_range(1..255));
    let symbol_size = 512;
    let mut data1: Vec<u8> = vec![0; symbol_size];
    let mut data2: Vec<u8> = vec![0; symbol_size];
    for i in 0..symbol_size {
        data1[i] = rand::rng().random_range(1..255);
        data2[i] = rand::rng().random_range(1..255);
    }
    let symbol1 = Symbol::new(data1);
    let symbol2 = Symbol::new(data2);

    let symbol1_mul_scalar = symbol1.clone();
    let octet1_mul_scalar = octet1.clone();
    let mut group = c.benchmark_group("Symbol mulassign_scalar()");
    group.throughput(Throughput::Bytes(symbol1.len() as u64));
    group.bench_function(BenchmarkId::new("", ""), move |b| {
        b.iter(|| {
            let mut temp = symbol1_mul_scalar.clone();
            temp.mulassign_scalar(&octet1_mul_scalar);
            temp
        })
    });
    group.finish();

    let symbol1_addassign = symbol1.clone();
    let symbol2_addassign = symbol2.clone();
    let mut group = c.benchmark_group("Symbol +=");
    group.throughput(Throughput::Bytes(symbol1.len() as u64));
    group.bench_function(BenchmarkId::new("", ""), move |b| {
        b.iter(|| {
            let mut temp = symbol1_addassign.clone();
            temp += &symbol2_addassign;
            temp
        })
    });
    group.finish();

    let symbol1_fma = symbol1.clone();
    let symbol2_fma = symbol2.clone();
    let octet1_fma = octet1.clone();
    let mut group = c.benchmark_group("Symbol FMA");
    group.throughput(Throughput::Bytes(symbol1.len() as u64));
    group.bench_function(BenchmarkId::new("", ""), move |b| {
        b.iter(|| {
            let mut temp = symbol1_fma.clone();
            temp.fused_addassign_mul_scalar(&symbol2_fma, &octet1_fma);
            temp
        })
    });
    group.finish();

    let elements = 10 * 1024;
    let symbol_size = 512;
    let mut data: Vec<u8> = vec![0; elements];
    for i in 0..elements {
        data[i] = rand::rng().random();
    }

    let encode_data = data.clone();
    let mut group = c.benchmark_group("encode 10KB");
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.bench_function(BenchmarkId::new("", ""), move |b| {
        b.iter(|| {
            let config = ObjectTransmissionInformation::new(0, symbol_size, 0, 1, 1);
            let encoder = SourceBlockEncoder::new(1, &config, &encode_data);
            return encoder.source_packets();
        })
    });
    group.finish();

    let roundtrip_data = data.clone();
    let mut group = c.benchmark_group("roundtrip 10KB");
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.bench_function(BenchmarkId::new("", ""), move |b| {
        b.iter(|| {
            let config = ObjectTransmissionInformation::new(0, symbol_size, 0, 1, 1);
            let encoder = SourceBlockEncoder::new(1, &config, &roundtrip_data);
            let mut decoder = SourceBlockDecoder::new(1, &config, elements as u64);
            return decoder.decode(encoder.source_packets());
        })
    });
    group.finish();

    let repair_data = data.clone();
    let mut group = c.benchmark_group("roundtrip repair 10KB");
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.bench_function(BenchmarkId::new("", ""), move |b| {
        b.iter(|| {
            let config = ObjectTransmissionInformation::new(0, symbol_size, 0, 1, 1);
            let encoder = SourceBlockEncoder::new(1, &config, &repair_data);
            let repair_packets = (elements / symbol_size as usize) as u32;
            let mut decoder = SourceBlockDecoder::new(1, &config, elements as u64);
            return decoder.decode(encoder.repair_packets(0, repair_packets));
        })
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
