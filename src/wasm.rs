use crate::base::EncodingPacket as EncodingPacketNative;
use crate::base::ObjectTransmissionInformation;
use crate::decoder::Decoder as DecoderNative;
use crate::encoder::Encoder as EncoderNative;

use crate::PayloadId;
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
