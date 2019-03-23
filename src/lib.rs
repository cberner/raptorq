mod util;
mod arraymap;
mod systematic_constants;
mod rng;
mod octet;
mod octets;
mod symbol;
mod matrix;
mod constraint_matrix;
mod base;
mod pi_solver;
mod encoder;
mod decoder;

pub use crate::base::PayloadId;
pub use crate::base::EncodingPacket;
pub use crate::base::ObjectTransmissionInformation;
pub use crate::encoder::SourceBlockEncoder;
pub use crate::encoder::Encoder;
pub use crate::decoder::SourceBlockDecoder;
pub use crate::decoder::Decoder;

#[cfg(feature = "benchmarking")]
pub use crate::constraint_matrix::generate_constraint_matrix;
#[cfg(feature = "benchmarking")]
pub use crate::systematic_constants::extended_source_block_symbols;
#[cfg(feature = "benchmarking")]
pub use crate::pi_solver::IntermediateSymbolDecoder;
#[cfg(feature = "benchmarking")]
pub use crate::octet::Octet;
#[cfg(feature = "benchmarking")]
pub use crate::symbol::Symbol;
