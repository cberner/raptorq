#![no_main]

use arbitrary::Unstructured;
use libfuzzer_sys::arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use rand::Rng;
use rand::prelude::*;
use raptorq::{Decoder, EncodingPacket, Encoder};
use std::mem::size_of;

#[derive(Debug, Clone)]
pub(crate) struct BoundedU16<const MIN: u16, const MAX: u16> {
    pub value: u16
}

impl<const MIN: u16, const MAX: u16> Arbitrary<'_> for BoundedU16<MIN, MAX> {
    fn arbitrary(u: &mut Unstructured<'_>) -> arbitrary::Result<Self> {
        let value: u16 = u.int_in_range(MIN..=MAX)?;
        Ok(Self {
            value
        })
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        (size_of::<u16>(), Some(size_of::<u16>()))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BoundedUsize<const MIN: usize, const MAX: usize> {
    pub value: usize
}

impl<const MIN: usize, const MAX: usize> Arbitrary<'_> for BoundedUsize<MIN, MAX> {
    fn arbitrary(u: &mut Unstructured<'_>) -> arbitrary::Result<Self> {
        let value: usize = u.int_in_range(MIN..=MAX)?;
        Ok(Self {
            value
        })
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        (size_of::<usize>(), Some(size_of::<usize>()))
    }
}

#[derive(Arbitrary, Debug, Clone)]
pub(crate) struct FuzzConfig {
    max_packet_size: BoundedU16<8, 1000>,
    data_len: BoundedUsize<1, 1_000_000>,
    seed: <rand::rngs::StdRng as rand::SeedableRng>::Seed,
}

fuzz_target!(|config: FuzzConfig| {
    let expected_symbols = config.data_len.value / config.max_packet_size.value as usize;
    let len = if expected_symbols > 56403 {
        56403 * config.max_packet_size.value as usize
    } else {
        config.data_len.value
    };

    let mut rng = rand::rngs::StdRng::from_seed(config.seed);
    // Generate some random data to send
    let mut data: Vec<u8> = vec![0; len];
    for i in 0..data.len() {
        data[i] = rng.gen();
    }

    let encoder = Encoder::with_defaults(&data, config.max_packet_size.value);

    // Perform the encoding, and serialize to Vec<u8> for transmission
    let mut packets: Vec<Vec<u8>> = encoder
        .get_encoded_packets((expected_symbols as u32) + 20)
        .iter()
        .map(|packet| packet.serialize())
        .collect();

    // Here we simulate losing 10 of the packets randomly. Normally, you would send them over
    // (potentially lossy) network here.
    packets.shuffle(&mut rng);
    // Erase 10 packets at random
    let length = packets.len();
    packets.truncate(length - 10);

    // The Decoder MUST be constructed with the configuration of the Encoder.
    // The ObjectTransmissionInformation configuration should be transmitted over a reliable
    // channel
    let mut decoder = Decoder::new(encoder.get_config());

    // Perform the decoding
    let mut result = None;
    while !packets.is_empty() {
        result = decoder.decode(EncodingPacket::deserialize(&packets.pop().unwrap()));
        if result != None {
            break;
        }
    }

    // Check that even though some of the data was lost we are able to reconstruct the original message
    assert_eq!(result.unwrap(), data);
});
