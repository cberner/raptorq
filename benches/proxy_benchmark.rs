use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rand::Rng;
use raptorq::{
    ObjectTransmissionInformation, SourceBlockDecoder, SourceBlockEncoder, SourceBlockEncodingPlan,
};

fn bench_encode(c: &mut Criterion) {
    let symbol_size: u16 = 1280;
    let mut group = c.benchmark_group("encode");

    for &symbol_count in &[10u32, 50, 100, 250, 500, 1000] {
        let elements = symbol_count as usize * symbol_size as usize;
        let mut data: Vec<u8> = vec![0; elements];
        for b in data.iter_mut() {
            *b = rand::rng().random();
        }
        let config = ObjectTransmissionInformation::new(0, symbol_size, 0, 1, 1);

        group.throughput(Throughput::Bytes(elements as u64));
        group.bench_with_input(
            BenchmarkId::new("no_plan", symbol_count),
            &symbol_count,
            |b, _| {
                b.iter(|| {
                    let encoder = SourceBlockEncoder::new(1, &config, &data);
                    encoder.repair_packets(0, 1)
                });
            },
        );

        let plan = SourceBlockEncodingPlan::generate(symbol_count as u16);
        group.bench_with_input(
            BenchmarkId::new("with_plan", symbol_count),
            &symbol_count,
            |b, _| {
                b.iter(|| {
                    let encoder = SourceBlockEncoder::with_encoding_plan(1, &config, &data, &plan);
                    encoder.repair_packets(0, 1)
                });
            },
        );
    }
    group.finish();
}

fn bench_decode(c: &mut Criterion) {
    let symbol_size: u16 = 1280;
    let mut group = c.benchmark_group("decode");

    for &symbol_count in &[10u32, 50, 100, 250, 500, 1000] {
        let elements = symbol_count as usize * symbol_size as usize;
        let mut data: Vec<u8> = vec![0; elements];
        for b in data.iter_mut() {
            *b = rand::rng().random();
        }
        let config = ObjectTransmissionInformation::new(0, symbol_size, 0, 1, 1);
        let encoder = SourceBlockEncoder::new(1, &config, &data);
        // Generate enough repair packets for all iterations
        let repair_count = symbol_count + 4;
        let packets = encoder.repair_packets(0, repair_count * 100);

        group.throughput(Throughput::Bytes(elements as u64));
        group.bench_with_input(
            BenchmarkId::new("repair_only", symbol_count),
            &symbol_count,
            |b, _| {
                let mut pkt_iter = packets.chunks(repair_count as usize);
                b.iter(|| {
                    let chunk = pkt_iter.next().unwrap_or_else(|| {
                        // Should not happen, but be safe
                        packets.chunks(repair_count as usize).next().unwrap()
                    });
                    let mut decoder = SourceBlockDecoder::new(1, &config, elements as u64);
                    decoder.decode(chunk.iter().cloned())
                });
            },
        );
    }
    group.finish();
}

fn bench_roundtrip(c: &mut Criterion) {
    let symbol_size: u16 = 1280;
    let mut group = c.benchmark_group("roundtrip");

    for &symbol_count in &[10u32, 50, 100, 250, 500] {
        let elements = symbol_count as usize * symbol_size as usize;
        let mut data: Vec<u8> = vec![0; elements];
        for b in data.iter_mut() {
            *b = rand::rng().random();
        }
        let config = ObjectTransmissionInformation::new(0, symbol_size, 0, 1, 1);

        group.throughput(Throughput::Bytes(elements as u64));
        group.bench_with_input(
            BenchmarkId::new("source_only", symbol_count),
            &symbol_count,
            |b, _| {
                b.iter(|| {
                    let encoder = SourceBlockEncoder::new(1, &config, &data);
                    let mut decoder = SourceBlockDecoder::new(1, &config, elements as u64);
                    decoder.decode(encoder.source_packets())
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_encode, bench_decode, bench_roundtrip);
criterion_main!(benches);
