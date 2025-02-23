//! Low-level crate implementing various RFCs for encoding emails.
//! Used internally by [lettre].
//!
//! [lettre]: https://crates.io/crates/lettre

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, missing_docs, rustdoc::broken_intra_doc_links)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
extern crate alloc;

pub mod body;
pub mod headers;
