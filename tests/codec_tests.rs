extern crate rand;
extern crate raptorq;

#[cfg(test)]
mod codec_tests {
    use rand::Rng;
    use raptorq::SourceBlockEncoder;
    use raptorq::SourceBlockDecoder;

    #[test]
    fn round_trip() {
        let elements = 1024*1024;
        let mut data: Vec<u8> = vec![0; elements];
        for i in 0..elements {
            data[i] = rand::thread_rng().gen();
        }

        let encoder = SourceBlockEncoder::new(1, 8, data.clone());

        let mut decoder = SourceBlockDecoder::new(1, 8, elements as u64);

        let mut result = None;
        for packet in encoder.all_source_packets() {
            assert_eq!(result, None);
            result = decoder.parse(packet);
        }

        assert_eq!(result.unwrap(), data);
    }
}
