use rand::RngExt;
use raptorq::{ObjectTransmissionInformation, SourceBlockEncoder, SourceBlockEncodingPlan};
use std::collections::HashMap;
use std::time::Instant;

const TARGET_TOTAL_BYTES: usize = 512 * 1024 * 1024;
const SYMBOL_COUNTS: [usize; 10] = [10, 100, 250, 500, 1000, 2000, 5000, 10000, 20000, 50000];
// A more realistic set of symbols for live streaming:
// const SYMBOL_COUNTS: [usize; 15] = [2, 3, 5, 7, 10, 14, 20, 30, 42, 90, 100, 120, 150, 200, 250];

fn black_box(value: u64) {
    if value == rand::rng().random::<u64>() {
        println!("{value}");
    }
}

fn benchmark(symbol_size: u16, pre_plan: bool, with_cached: bool) -> u64 {
    let mut black_box_value = 0;
    let mut encoder_cache: HashMap<usize, SourceBlockEncoder> = HashMap::new();

    for symbol_count in SYMBOL_COUNTS.iter() {
        let elements = symbol_count * symbol_size as usize;
        let mut data: Vec<u8> = vec![0; elements];
        for byte in data.iter_mut() {
            *byte = rand::rng().random();
        }

        let plan = if pre_plan || with_cached {
            Some(SourceBlockEncodingPlan::generate(*symbol_count as u16))
        } else {
            None
        };

        let config = ObjectTransmissionInformation::new(0, symbol_size, 0, 1, 1);

        if with_cached && plan.is_some() {
            encoder_cache.entry(*symbol_count).or_insert_with(|| {
                SourceBlockEncoder::with_encoding_plan(1, &config, &data, plan.as_ref().unwrap())
            });
        }

        let now = Instant::now();
        let iterations = TARGET_TOTAL_BYTES / elements;
        for _ in 0..iterations {
            let packets = if with_cached {
                let encoder = encoder_cache.get_mut(symbol_count).unwrap();
                encoder.reset(&data, plan.as_ref().unwrap());
                encoder.repair_packets(0, 1)
            } else {
                let fresh_plan;
                let plan_ref = if let Some(ref plan) = plan {
                    plan
                } else {
                    fresh_plan = SourceBlockEncodingPlan::generate(*symbol_count as u16);
                    &fresh_plan
                };
                SourceBlockEncoder::with_encoding_plan(1, &config, &data, plan_ref)
                    .repair_packets(0, 1)
            };
            black_box_value += packets[0].data()[0] as u64;
        }
        let elapsed = now.elapsed();
        let elapsed = elapsed.as_secs() as f64 + elapsed.subsec_millis() as f64 * 0.001;
        let throughput = (elements * iterations * 8) as f64 / 1024.0 / 1024.0 / elapsed;
        println!(
            "symbol count = {}, iters = {}, encoded {} MB in {:.3}secs ({:.4}ms per iter), throughput: {:.1}Mbit/s",
            symbol_count,
            iterations,
            elements * iterations / 1024 / 1024,
            elapsed,
            elapsed * 1000.0 / iterations as f64,
            throughput
        );
    }
    black_box_value
}

fn main() {
    let symbol_size = 1280;
    let variant = std::env::args().nth(1).unwrap_or_default();
    match variant.as_str() {
        "no-plan" => {
            println!("Symbol size: {symbol_size} bytes (fresh plan each iteration)");
            black_box(benchmark(symbol_size, false, false));
        }
        "pre-plan" => {
            println!("Symbol size: {symbol_size} bytes (with pre-built plan)");
            black_box(benchmark(symbol_size, true, false));
        }
        "cached" => {
            println!("Symbol size: {symbol_size} bytes (cached encoder)");
            black_box(benchmark(symbol_size, true, true));
        }
        _ => {
            println!("usage: encode_benchmark <no-plan|pre-plan|cached>");
            println!("  no-plan   fresh plan generated each iteration (true cold cost)");
            println!("  pre-plan  new encoder each iteration, pre-built plan");
            println!("  cached    encoder reused via reset(), pre-built plan");
        }
    }
}
