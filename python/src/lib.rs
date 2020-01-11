use pyo3::prelude::*;
use pyo3::types::*;
use raptorq::{
    Encoder as EncoderNative,
    Decoder as DecoderNative,
    ObjectTransmissionInformation,
    EncodingPacket,
};


#[pyclass]
struct Encoder {
    encoder: EncoderNative
}

#[pymethods]
impl Encoder {
    #[staticmethod]
    pub fn with_defaults(data: &PyBytes, maximum_transmission_unit: u16) -> PyResult<Encoder> {
        let encoder = EncoderNative::with_defaults(data.as_bytes(), maximum_transmission_unit);
        Ok(Encoder { encoder })
    }

    pub fn get_encoded_packets<'p>(&self,
        py: Python<'p>,
        repair_packets_per_block: u32,
    ) -> PyResult<Vec<&'p PyBytes>> {
        let packets: Vec<&PyBytes> = self.encoder.get_encoded_packets(repair_packets_per_block)
            .iter()
            .map(|packet| PyBytes::new(py, &packet.serialize()))
            .collect();

        Ok(packets)
    }
}

#[pyclass]
struct Decoder {
    decoder: DecoderNative
}

#[pymethods]
impl Decoder {
    #[staticmethod]
    pub fn with_defaults(transfer_length: u64, maximum_transmission_unit: u16) -> PyResult<Decoder> {
        let config = ObjectTransmissionInformation::with_defaults(
            transfer_length,
            maximum_transmission_unit,
        );
        let decoder = DecoderNative::new(config);
        Ok(Decoder { decoder })
    }

    pub fn decode<'p>(&mut self, py: Python<'p>, packet: &PyBytes) -> PyResult<Option<&'p PyBytes>> {
        let result = self.decoder.decode(EncodingPacket::deserialize(packet.as_bytes()));
        Ok(result.map(|data| PyBytes::new(py, &data)))
    }
}

#[pymodule]
fn raptorq(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Encoder>()?;
    m.add_class::<Decoder>()?;
    Ok(())
}