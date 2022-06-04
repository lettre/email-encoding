//! [RFC 2047] encoder
//!
//! [RFC 2047]: https://datatracker.ietf.org/doc/html/rfc2047

use std::fmt::{self, Write};

use super::{utils, EmailWriter, MAX_LINE_LEN};

const ENCODING_START_PREFIX: &str = "=?utf-8?b?";
const ENCODING_END_SUFFIX: &str = "?=";

/// Encode a string via RFC 2047
///
/// # Examples
///
/// ```rust
/// # use email_encoding::headers::writer::EmailWriter;
/// # fn main() -> std::fmt::Result {
/// let input = "Adrián";
///
/// let mut output = String::new();
/// let mut writer = EmailWriter::new(&mut output, 0, false);
/// email_encoding::headers::rfc2047::encode(input, &mut writer)?;
/// assert_eq!(output, "=?utf-8?b?QWRyacOhbg==?=");
/// # Ok(())
/// # }
/// ```
pub fn encode(mut s: &str, w: &mut EmailWriter<'_>) -> fmt::Result {
    while !s.is_empty() {
        let remaining_line_len = MAX_LINE_LEN.saturating_sub(
            ENCODING_START_PREFIX.len() + ENCODING_END_SUFFIX.len() + w.line_len() + "\r\n".len(),
        );
        let unencoded_remaining_line_len = remaining_line_len / 4 * 3;

        let word = utils::truncate_to_char_boundary(s, unencoded_remaining_line_len.min(s.len()));

        if word.is_empty() {
            // No space remaining on this line, go to a new one
            w.new_line()?;
            continue;
        }

        // Write the prefix
        w.write_str(ENCODING_START_PREFIX)?;

        // Encode `word`
        let encoder =
            base64::display::Base64Display::with_config(word.as_bytes(), base64::STANDARD);
        write!(w, "{}", encoder)?;

        // Write the suffix
        w.write_str(ENCODING_END_SUFFIX)?;

        // Advance `s`
        s = &s[word.len()..];
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn empty() {
        let mut s = String::new();
        let line_len = s.len();

        let mut w = EmailWriter::new(&mut s, line_len, false);
        encode("", &mut w).unwrap();

        assert_eq!(s, "");
    }

    #[test]
    fn basic() {
        let mut s = String::new();
        let line_len = s.len();

        let mut w = EmailWriter::new(&mut s, line_len, false);
        encode("abcd", &mut w).unwrap();

        assert_eq!(s, "=?utf-8?b?YWJjZA==?=");
    }

    #[test]
    fn basic_nopad() {
        let mut s = String::new();
        let line_len = s.len();

        let mut w = EmailWriter::new(&mut s, line_len, false);
        encode("abcdef", &mut w).unwrap();

        assert_eq!(s, "=?utf-8?b?YWJjZGVm?=");
    }

    #[test]
    fn long() {
        let mut s = String::new();
        let line_len = s.len();

        let mut w = EmailWriter::new(&mut s, line_len, false);
        encode(&"lettre".repeat(20), &mut w).unwrap();

        assert_eq!(
            s,
            concat!(
                "=?utf-8?b?bGV0dHJlbGV0dHJlbGV0dHJlbGV0dHJlbGV0dHJlbGV0dHJlbGV0dHJlbGV0?=\r\n",
                " =?utf-8?b?dHJlbGV0dHJlbGV0dHJlbGV0dHJlbGV0dHJlbGV0dHJlbGV0dHJlbGV0dHJl?=\r\n",
                " =?utf-8?b?bGV0dHJlbGV0dHJlbGV0dHJlbGV0dHJlbGV0dHJl?="
            )
        );
    }

    #[test]
    fn long_encoded() {
        let mut s = String::new();
        let line_len = s.len();

        let mut w = EmailWriter::new(&mut s, line_len, false);
        encode(&"hétérogénéité".repeat(16), &mut w).unwrap();

        assert_eq!(
            s,
            concat!(
                "=?utf-8?b?aMOpdMOpcm9nw6luw6lpdMOpaMOpdMOpcm9nw6luw6lpdMOpaMOpdMOpcm9n?=\r\n",
                " =?utf-8?b?w6luw6lpdMOpaMOpdMOpcm9nw6luw6lpdMOpaMOpdMOpcm9nw6luw6lpdMOp?=\r\n",
                " =?utf-8?b?aMOpdMOpcm9nw6luw6lpdMOpaMOpdMOpcm9nw6luw6lpdMOpaMOpdMOpcm9n?=\r\n",
                " =?utf-8?b?w6luw6lpdMOpaMOpdMOpcm9nw6luw6lpdMOpaMOpdMOpcm9nw6luw6lpdMOp?=\r\n",
                " =?utf-8?b?aMOpdMOpcm9nw6luw6lpdMOpaMOpdMOpcm9nw6luw6lpdMOpaMOpdMOpcm9n?=\r\n",
                " =?utf-8?b?w6luw6lpdMOpaMOpdMOpcm9nw6luw6lpdMOpaMOpdMOpcm9nw6luw6lpdMOp?=\r\n",
                " =?utf-8?b?aMOpdMOpcm9nw6luw6lpdMOp?=",
            )
        );
    }
}
