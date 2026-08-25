//! An implementation of RaptorQ, the fountain code defined in
//! [RFC 6330](https://tools.ietf.org/html/rfc6330).
//!
//! # Erasure correction, not error detection
//!
//! RaptorQ recovers *erasures*: packets that were lost in transit. Given
//! enough of the packets that were sent, it reconstructs the original object
//! regardless of which ones went missing.
//!
//! It does **not** detect *errors*: packets that arrived but were altered.
//! Nothing in the encoded stream identifies a packet as corrupt, so the
//! decoder cannot distinguish a modified packet from a legitimate one.
//!
//! **Callers are responsible for ensuring that data passed to the decode
//! functions is free of corruption.** Verify integrity before decoding, with
//! whatever the surrounding protocol provides — a transport that authenticates
//! its payloads, a checksum, or a signature over each packet.
//!
//! Passing corrupted data in has no single defined outcome. It may:
//!
//! - **return incorrect data**, with no error and no indication that anything
//!   is wrong, when the corruption is in a packet's payload;
//! - **panic**, when the corruption is in the 4-byte FEC Payload ID that
//!   prefixes each packet, since the block and symbol numbers it carries are
//!   used to index the decoder's state;
//! - fail to decode and return [`None`].
//!
//! None of these should be relied on as a signal. A corruption check that
//! happens *before* decoding is the only thing that makes the outcome
//! well-defined.

#![allow(
    clippy::needless_return,
    clippy::unreadable_literal,
    clippy::needless_range_loop
)]
#![cfg_attr(not(feature = "std"), no_std)]

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
#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), feature = "std"))]
mod octets_gfni;
mod operation_vector;
mod pi_solver;
#[cfg(feature = "python")]
mod python;
mod rng;
mod sparse_matrix;
mod sparse_vec;
mod symbol;
mod symbol_slab;
mod systematic_constants;
mod util;

pub use crate::base::EncodingPacket;
pub use crate::base::ObjectTransmissionInformation;
pub use crate::base::PayloadId;
pub use crate::base::partition;
#[cfg(not(feature = "python"))]
pub use crate::decoder::Decoder;
pub use crate::decoder::SourceBlockDecoder;
#[cfg(not(feature = "python"))]
pub use crate::encoder::Encoder;
pub use crate::encoder::EncoderBuilder;
pub use crate::encoder::SourceBlockEncoder;
pub use crate::encoder::SourceBlockEncodingPlan;
pub use crate::encoder::calculate_block_offsets;
#[cfg(feature = "python")]
pub use crate::python::Decoder;
#[cfg(feature = "python")]
pub use crate::python::Encoder;
#[cfg(feature = "python")]
pub use crate::python::raptorq;
pub use crate::systematic_constants::extended_source_block_symbols;

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
#[cfg(feature = "benchmarking")]
pub use crate::symbol_slab::SymbolSlab;
