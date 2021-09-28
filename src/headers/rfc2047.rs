use std::fmt::{self, Write};

use super::{utils, MAX_LINE_LEN};

const ENCODING_START_PREFIX: &str = "=?utf-8?b?";
const ENCODING_END_SUFFIX: &str = "?=";

pub(super) fn encode(mut s: &str, w: &mut dyn Write, line_len: &mut usize) -> fmt::Result {
    while !s.is_empty() {
        let remaining_line_len = MAX_LINE_LEN.saturating_sub(
            ENCODING_START_PREFIX.len() + ENCODING_END_SUFFIX.len() + *line_len + "\r\n".len(),
        );
        let unencoded_remaining_line_len = remaining_line_len / 4 * 3;

        let word = utils::truncate_to_char_boundary(s, unencoded_remaining_line_len.min(s.len()));

        if word.is_empty() {
            // No space remaining on this line, go to a new one
            w.write_str("\r\n ")?;
            *line_len = 1;
            continue;
        }

        // Write the prefix
        w.write_str(ENCODING_START_PREFIX)?;
        *line_len += ENCODING_START_PREFIX.len();

        // Encode `word`
        let encoder =
            base64::display::Base64Display::with_config(word.as_bytes(), base64::STANDARD);
        write!(w, "{}", encoder)?;
        *line_len += word.len() / 3 * 4 + if word.len() % 3 == 0 { 0 } else { 4 };

        // Write the suffix
        w.write_str(ENCODING_END_SUFFIX)?;
        *line_len += ENCODING_END_SUFFIX.len();

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
        let mut line_len = s.len();

        encode("", &mut s, &mut line_len).unwrap();

        assert_eq!(s, "");
    }

    #[test]
    fn basic() {
        let mut s = String::new();
        let mut line_len = s.len();

        encode("abcd", &mut s, &mut line_len).unwrap();

        assert_eq!(s, "=?utf-8?b?YWJjZA==?=");
    }

    #[test]
    fn basic_nopad() {
        let mut s = String::new();
        let mut line_len = s.len();

        encode("abcdef", &mut s, &mut line_len).unwrap();

        assert_eq!(s, "=?utf-8?b?YWJjZGVm?=");
    }

    #[test]
    fn long() {
        let mut s = String::new();
        let mut line_len = s.len();

        encode(&"lettre".repeat(20), &mut s, &mut line_len).unwrap();

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
        let mut line_len = s.len();

        encode(&"hétérogénéité".repeat(16), &mut s, &mut line_len).unwrap();

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
