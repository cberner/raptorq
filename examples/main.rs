#[cfg(not(feature = "python"))]
use rand::seq::SliceRandom;

#[cfg(not(feature = "python"))]
use rand::Rng;

#[cfg(not(feature = "python"))]
use raptorq::{Decoder, Encoder, EncodingPacket};

#[cfg(not(feature = "python"))]
fn main() {
    // Generate some random data to send
    let mut data: Vec<u8> = vec![0; 10_000];
    for i in 0..data.len() {
        data[i] = rand::thread_rng().gen();
    }

    // Create the Encoder, with an MTU of 1400 (common for Ethernet)
    let encoder = Encoder::with_defaults(&data, 1400);

    // Perform the encoding, and serialize to Vec<u8> for transmission
    let mut packets: Vec<Vec<u8>> = encoder
        .get_encoded_packets(15)
        .iter()
        .map(|packet| packet.serialize())
        .collect();

    // Here we simulate losing 10 of the packets randomly. Normally, you would send them over
    // (potentially lossy) network here.
    packets.shuffle(&mut rand::thread_rng());
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
        if result.is_some() {
            break;
        }
    }

    // Check that even though some of the data was lost we are able to reconstruct the original message
    assert_eq!(result.unwrap(), data);
}

#[cfg(feature = "python")]
fn main() {
    panic!("This is not indented to compile for `python` feature.");
}
