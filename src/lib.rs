mod arraymap;
mod base;
mod constraint_matrix;
mod decoder;
mod encoder;
mod matrix;
mod octet;
mod octets;
mod pi_solver;
mod rng;
mod symbol;
mod systematic_constants;
mod util;

pub use crate::base::EncodingPacket;
pub use crate::base::ObjectTransmissionInformation;
pub use crate::base::PayloadId;
pub use crate::decoder::Decoder;
pub use crate::decoder::SourceBlockDecoder;
pub use crate::encoder::Encoder;
pub use crate::encoder::SourceBlockEncoder;

#[cfg(feature = "benchmarking")]
pub use crate::constraint_matrix::generate_constraint_matrix;
#[cfg(feature = "benchmarking")]
pub use crate::octet::Octet;
#[cfg(feature = "benchmarking")]
pub use crate::pi_solver::IntermediateSymbolDecoder;
#[cfg(feature = "benchmarking")]
pub use crate::symbol::Symbol;
#[cfg(feature = "benchmarking")]
pub use crate::systematic_constants::extended_source_block_symbols;
