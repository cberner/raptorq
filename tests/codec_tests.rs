extern crate rand;
extern crate raptorq;

#[cfg(test)]
mod codec_tests {
    use rand::Rng;
    use raptorq::SourceBlockEncoder;
    use raptorq::SourceBlockDecoder;
    use raptorq::Octet;

    #[test]
    fn round_trip() {
        Octet::static_init();

        let elements = 1024;
        let mut data: Vec<u8> = vec![0; elements];
        for i in 0..elements {
            data[i] = rand::thread_rng().gen();
        }

        let encoder = SourceBlockEncoder::new(1, 8, &data);

        let mut decoder = SourceBlockDecoder::new(1, 8, elements as u64);

        let mut result = None;
        for packet in encoder.all_source_packets() {
            assert_eq!(result, None);
            result = decoder.parse(packet);
        }

        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn repair() {
        Octet::static_init();

        let elements = 1024;
        let mut data: Vec<u8> = vec![0; elements];
        for i in 0..elements {
            data[i] = rand::thread_rng().gen();
        }

        let encoder = SourceBlockEncoder::new(1, 8, &data);

        let mut decoder = SourceBlockDecoder::new(1, 8, elements as u64);

        let mut result = None;
        // TODO: make this test pass deterministically. Right now it will fail with 1% probability
        for packet in encoder.repair_packets(0, (elements / 8) as u32) {
            assert_eq!(result, None);
            result = decoder.parse(packet);
        }

        assert_eq!(result.unwrap(), data);
    }
}
