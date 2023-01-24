#![allow(clippy::needless_return, clippy::unreadable_literal)]
#![no_std]

#[cfg(feature = "metal")]
#[macro_use]
extern crate alloc;

#[cfg(feature = "metal")]
extern crate core;

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

#[cfg(any(feature = "std", feature = "metal"))]
mod arraymap;
#[cfg(any(feature = "std", feature = "metal"))]
mod base;
#[cfg(any(feature = "std", feature = "metal"))]
mod constraint_matrix;
#[cfg(any(feature = "std", feature = "metal"))]
mod decoder;
#[cfg(any(feature = "std", feature = "metal"))]
mod encoder;
mod features_check;
#[cfg(any(feature = "std", feature = "metal"))]
mod gf2;
#[cfg(any(feature = "std", feature = "metal"))]
mod graph;
#[cfg(any(feature = "std", feature = "metal"))]
mod iterators;
#[cfg(any(feature = "std", feature = "metal"))]
mod matrix;
#[cfg(any(feature = "std", feature = "metal"))]
mod octet;
#[cfg(any(feature = "std", feature = "metal"))]
mod octet_matrix;
#[cfg(any(feature = "std", feature = "metal"))]
mod octets;
#[cfg(any(feature = "std", feature = "metal"))]
mod operation_vector;
#[cfg(any(feature = "std", feature = "metal"))]
mod pi_solver;
#[cfg(feature = "python")]
mod python;
#[cfg(any(feature = "std", feature = "metal"))]
mod rng;
#[cfg(any(feature = "std", feature = "metal"))]
mod sparse_matrix;
#[cfg(any(feature = "std", feature = "metal"))]
mod sparse_vec;
#[cfg(any(feature = "std", feature = "metal"))]
mod symbol;
#[cfg(any(feature = "std", feature = "metal"))]
mod systematic_constants;
#[cfg(any(feature = "std", feature = "metal"))]
mod util;
#[cfg(feature = "wasm")]
mod wasm;

#[cfg(any(feature = "std", feature = "metal"))]
pub use crate::base::partition;
#[cfg(any(feature = "std", feature = "metal"))]
pub use crate::base::EncodingPacket;
#[cfg(any(feature = "std", feature = "metal"))]
pub use crate::base::ObjectTransmissionInformation;
#[cfg(any(feature = "std", feature = "metal"))]
pub use crate::base::PayloadId;
#[cfg(all(
    any(feature = "std", feature = "metal"),
    not(feature = "python"),
    not(feature = "wasm")
))]
pub use crate::decoder::Decoder;
#[cfg(any(feature = "std", feature = "metal"))]
pub use crate::decoder::SourceBlockDecoder;
#[cfg(any(feature = "std", feature = "metal"))]
pub use crate::encoder::calculate_block_offsets;
#[cfg(all(
    any(feature = "std", feature = "metal"),
    not(feature = "python"),
    not(feature = "wasm")
))]
pub use crate::encoder::Encoder;
#[cfg(any(feature = "std", feature = "metal"))]
pub use crate::encoder::EncoderBuilder;
#[cfg(any(feature = "std", feature = "metal"))]
pub use crate::encoder::SourceBlockEncoder;
#[cfg(any(feature = "std", feature = "metal"))]
pub use crate::encoder::SourceBlockEncodingPlan;
#[cfg(feature = "python")]
pub use crate::python::raptorq;
#[cfg(feature = "python")]
pub use crate::python::Decoder;
#[cfg(feature = "python")]
pub use crate::python::Encoder;
#[cfg(any(feature = "std", feature = "metal"))]
pub use crate::systematic_constants::extended_source_block_symbols;
#[cfg(feature = "wasm")]
pub use crate::wasm::Decoder as WasmDecoder;
#[cfg(feature = "wasm")]
pub use crate::wasm::Encoder as WasmEncoder;

#[cfg(feature = "benchmarking")]
pub use crate::constraint_matrix::generate_constraint_matrix;
#[cfg(feature = "benchmarking")]
pub use crate::matrix::BinaryMatrix;
#[cfg(feature = "benchmarking")]
pub use crate::matrix::DenseBinaryMatrix;
#[cfg(feature = "benchmarking")]
pub use crate::octet::Octet;
#[cfg(feature = "benchmarking")]
pub use crate::pi_solver::IntermediateSymbolDecoder;
#[cfg(feature = "benchmarking")]
pub use crate::sparse_matrix::SparseBinaryMatrix;
#[cfg(feature = "benchmarking")]
pub use crate::symbol::Symbol;
