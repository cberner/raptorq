#![allow(clippy::needless_return, clippy::unreadable_literal)]
#![no_std]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

#[cfg(not(feature = "std"))]
extern crate core;

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

mod arraymap;
mod base;
mod constraint_matrix;
mod decoder;
mod encoder;
mod gf2;
mod graph;
mod iterators;
mod matrix;
mod octet;
mod octet_matrix;
mod octets;
mod operation_vector;
mod pi_solver;
#[cfg(feature = "python")]
mod python;
mod rng;
mod sparse_matrix;
mod sparse_vec;
mod symbol;
mod systematic_constants;
mod util;
#[cfg(feature = "wasm")]
mod wasm;

pub use crate::base::partition;
pub use crate::base::EncodingPacket;
pub use crate::base::ObjectTransmissionInformation;
pub use crate::base::PayloadId;
#[cfg(not(any(feature = "python", feature = "wasm")))]
pub use crate::decoder::Decoder;
pub use crate::decoder::SourceBlockDecoder;
pub use crate::encoder::calculate_block_offsets;
#[cfg(not(any(feature = "python", feature = "wasm")))]
pub use crate::encoder::Encoder;
pub use crate::encoder::EncoderBuilder;
pub use crate::encoder::SourceBlockEncoder;
pub use crate::encoder::SourceBlockEncodingPlan;
#[cfg(feature = "python")]
pub use crate::python::raptorq;
#[cfg(feature = "python")]
pub use crate::python::Decoder;
#[cfg(feature = "python")]
pub use crate::python::Encoder;
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
