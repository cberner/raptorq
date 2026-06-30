use std::sync::Mutex;
use std::vec::Vec;

use crate::base::{EncodingPacket, ObjectTransmissionInformation};
use crate::decoder::Decoder as DecoderNative;
use crate::encoder::Encoder as EncoderNative;
use pyo3::prelude::*;
use pyo3::types::*;

#[pyclass(frozen)]
struct Encoder {
    encoder: EncoderNative,
}

#[pymethods]
impl Encoder {
    #[staticmethod]
    pub fn with_defaults(
        data: Bound<'_, PyBytes>,
        maximum_transmission_unit: u16,
    ) -> PyResult<Encoder> {
        let encoder = EncoderNative::with_defaults(data.as_bytes(), maximum_transmission_unit);
        Ok(Encoder { encoder })
    }
    
    fn get_encoded_packets(
        &self,
        py: Python<'_>,
        repair_packets_per_block: u32,
    ) -> PyResult<Vec<Py<PyBytes>>> {
        let raw: Vec<Vec<u8>> = py.detach(|| {
            self.encoder
                .get_encoded_packets(repair_packets_per_block)
                .iter()
                .map(|packet| packet.serialize())
                .collect()
        });
        Ok(raw.iter().map(|bytes| PyBytes::new(py, bytes).into()).collect())
    }
}

#[pyclass(frozen)]
struct Decoder {
    decoder: Mutex<DecoderNative>,
}

#[pymethods]
impl Decoder {
    #[staticmethod]
    fn with_defaults(
        transfer_length: u64,
        maximum_transmission_unit: u16,
    ) -> Decoder {
        let config = ObjectTransmissionInformation::with_defaults(
            transfer_length,
            maximum_transmission_unit,
        );
        Decoder { decoder: Mutex::new(DecoderNative::new(config)) }
    }

    fn decode(
        &self,
        py: Python<'_>,
        packet: Bound<'_, PyBytes>,
    ) -> PyResult<Option<Py<PyBytes>>> {
        let packet_bytes = packet.as_bytes().to_vec();
        let result: Option<Vec<u8>> = py.detach(|| {
            self.decoder
                .lock()
                .unwrap()
                .decode(EncodingPacket::deserialize(&packet_bytes))
        });
        Ok(result.map(|data| PyBytes::new(py, &data).into()))
    }
}

#[pymodule]
pub fn raptorq(_py: Python<'_>, m: Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Encoder>()?;
    m.add_class::<Decoder>()?;
    Ok(())
}
