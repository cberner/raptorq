use std::vec::Vec;

use crate::base::{EncodingPacket, ObjectTransmissionInformation};
use crate::decoder::Decoder as DecoderNative;
use crate::encoder::Encoder as EncoderNative;
use js_sys::Uint8Array;
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
        self.decoder.decode(EncodingPacket::deserialize(packet))
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
