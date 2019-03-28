#[cfg(test)]
mod codec_tests {
    use rand::seq::SliceRandom;
    use rand::Rng;
    use raptorq::Decoder;
    use raptorq::Encoder;
    use raptorq::SourceBlockDecoder;
    use raptorq::SourceBlockEncoder;

    #[test]
    fn random_erasure() {
        let elements: usize = rand::thread_rng().gen_range(1, 1_000_000);
        let mut data: Vec<u8> = vec![0; elements];
        for i in 0..elements {
            data[i] = rand::thread_rng().gen();
        }

        // MTU is set to not be too small, otherwise this test may take a very long time
        let mtu = rand::thread_rng().gen_range(elements as u16 / 100, 10_000);

        let encoder = Encoder::with_defaults(&data, mtu);

        let mut packets = encoder.get_encoded_packets(15);
        packets.shuffle(&mut rand::thread_rng());
        // Erase 10 packets at random
        let length = packets.len();
        packets.truncate(length - 10);

        let mut decoder = Decoder::new(encoder.get_config());

        let mut result = None;
        while !packets.is_empty() {
            result = decoder.decode(packets.pop().unwrap());
            if result != None {
                break;
            }
        }

        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn round_trip() {
        let elements = 1024;
        let mut data: Vec<u8> = vec![0; elements];
        for i in 0..elements {
            data[i] = rand::thread_rng().gen();
        }

        let encoder = SourceBlockEncoder::new(1, 8, &data);

        let mut decoder = SourceBlockDecoder::new(1, 8, elements as u64);

        let mut result = None;
        for packet in encoder.source_packets() {
            assert_eq!(result, None);
            result = decoder.decode(packet);
        }

        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn repair() {
        let elements = 1024;
        let mut data: Vec<u8> = vec![0; elements];
        for i in 0..elements {
            data[i] = rand::thread_rng().gen();
        }

        let encoder = SourceBlockEncoder::new(1, 8, &data);

        let mut decoder = SourceBlockDecoder::new(1, 8, elements as u64);

        let mut result = None;
        let mut parsed_packets = 0;
        // This test can theoretically fail with ~1/256^5 probability
        for packet in encoder.repair_packets(0, (elements / 8 + 4) as u32) {
            if parsed_packets < elements / 8 {
                assert_eq!(result, None);
            }
            result = decoder.decode(packet);
            parsed_packets += 1;
        }

        assert_eq!(result.unwrap(), data);
    }
}
