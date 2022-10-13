use crate::base::EncodingPacket as EncodingPacketNative;
use crate::base::ObjectTransmissionInformation;
use crate::decoder::Decoder as DecoderNative;
use crate::encoder::Encoder as EncoderNative;

use crate::PayloadId;
use js_sys::Uint8Array;
use nom::bits::complete::take as bit_take;
use nom::combinator::{rest, verify};
use nom::complete::tag as bit_tag;
use nom::sequence::{preceded, tuple};
use nom::{bits, IResult};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Decoder {
    decoder: DecoderNative,
}

#[wasm_bindgen]
impl Decoder {
    #[wasm_bindgen]
    pub fn with_defaults(transfer_length: u64, maximum_transmission_unit: u16) -> Decoder {
        let config = ObjectTransmissionInformation::with_defaults(
            transfer_length,
            maximum_transmission_unit,
        );
        Decoder {
            decoder: DecoderNative::new(config),
        }
    }

    #[wasm_bindgen]
    pub fn decode(&mut self, packet: &[u8]) -> Option<Vec<u8>> {
        self.decoder
            .decode(EncodingPacketNative::deserialize(packet))
    }
}

#[wasm_bindgen]
pub struct Encoder {
    encoder: EncoderNative,
}

#[wasm_bindgen]
impl Encoder {
    #[wasm_bindgen]
    pub fn with_defaults(data: &[u8], maximum_transmission_unit: u16) -> Encoder {
        Encoder {
            encoder: EncoderNative::with_defaults(data, maximum_transmission_unit),
        }
    }

    #[wasm_bindgen]
    pub fn encode(&mut self, repair_packets_per_block: u32) -> Vec<Uint8Array> {
        self.encoder
            .get_encoded_packets(repair_packets_per_block)
            .iter()
            .map(|packet| Uint8Array::from(packet.serialize().as_slice()))
            .collect()
    }
}

#[wasm_bindgen]
pub struct EncodingPacket {
    source_block_number: u8,
    encoding_symbol_id: u32,
    data: Vec<u8>,
}

#[wasm_bindgen]
impl EncodingPacket {
    #[wasm_bindgen]
    pub fn deserialize(data: &[u8]) -> EncodingPacket {
        let payload_data = [data[0], data[1], data[2], data[3]];
        let payload_id = PayloadId::deserialize(&payload_data);
        EncodingPacket {
            source_block_number: payload_id.source_block_number(),
            encoding_symbol_id: payload_id.encoding_symbol_id(),
            data: Vec::from(&data[4..]),
        }
    }

    #[wasm_bindgen]
    pub fn source_block_number(&self) -> u8 {
        self.source_block_number
    }

    #[wasm_bindgen]
    pub fn encoding_symbol_id(&self) -> u32 {
        self.encoding_symbol_id
    }

    #[wasm_bindgen]
    pub fn data(&self) -> Uint8Array {
        Uint8Array::from(self.data.as_slice())
    }
}

#[wasm_bindgen]
pub struct RaptorqFrame {
    size: u32,
    payload: Vec<u8>,
}

#[wasm_bindgen]
impl RaptorqFrame {
    pub fn total(&self) -> u32 {
        // Decoding algorithm is probabilistic, see documentation of the `raptorq` crate
        // Rephrasing from there, the probability to decode message with h
        // additional packets is 1 - 1/256^(h+1).
        //
        // Thus, if there are no additional packets, probability is ~ 0.99609.
        // If one additional packet is added, it is ~ 0.99998.
        // It was decided to add one additional packet in the printed estimate, so that
        // the user expectations are lower.
        self.size / (self.payload.len() as u32) + 1
    }

    #[wasm_bindgen]
    pub fn size(&self) -> u32 {
        self.size
    }

    #[wasm_bindgen]
    pub fn payload(&self) -> Uint8Array {
        Uint8Array::from(self.payload.as_slice())
    }

    #[wasm_bindgen]
    pub fn try_from(i: &[u8]) -> RaptorqFrame {
        let (_, (size, payload)) = parse_raptor_frame(i).unwrap();

        Self {
            size,
            payload: payload.to_vec(),
        }
    }
}

fn parse_raptor_frame(i: &[u8]) -> IResult<&[u8], (u32, &[u8])> {
    tuple((
        bits(preceded(raptorq_tag, raptor_payload_size)),
        verify(rest, |a: &[u8]| !a.is_empty()),
    ))(i)
}

fn raptorq_tag(i: (&[u8], usize)) -> IResult<(&[u8], usize), u8> {
    bit_tag(0x1, 1usize)(i)
}

fn raptor_payload_size(i: (&[u8], usize)) -> IResult<(&[u8], usize), u32> {
    bit_take(31usize)(i)
}
