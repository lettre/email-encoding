pub use self::writer::EmailWriter;

mod hex;
mod hex_encoding;
pub mod quoted_string;
pub mod rfc2047;
pub mod rfc2231;
mod utils;
mod writer;

pub(super) const MAX_LINE_LEN: usize = 76;
