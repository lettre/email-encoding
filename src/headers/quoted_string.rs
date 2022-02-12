use std::fmt::{self, Write};

use super::{rfc2047, utils, EmailWriter};

pub fn encode(value: &str, w: &mut EmailWriter<'_>) -> fmt::Result {
    enum Strategy {
        Plain,
        Quoted,
        QuotedEscaped,
        Rfc2047,
    }

    let mut strategy = Strategy::Plain;

    let mut bytes = value.as_bytes();

    // Plain -> Quoted
    while !bytes.is_empty() {
        let byte = bytes[0];

        if !byte.is_ascii_alphanumeric() && !matches!(byte, b'-' | b'_' | b'.') {
            strategy = Strategy::Quoted;
            break;
        }

        bytes = &bytes[1..];
    }

    // Quoted -> QuotedEscaped
    while !bytes.is_empty() {
        let byte = bytes[0];

        if !byte.is_ascii_alphanumeric() && !matches!(byte, b' ' | b'-' | b'_' | b'.') {
            strategy = Strategy::QuotedEscaped;
            break;
        }

        bytes = &bytes[1..];
    }

    // QuotedEscaped -> Rfc2047
    while !bytes.is_empty() {
        let byte = bytes[0];

        if !byte.is_ascii_alphanumeric()
            && !matches!(byte, b'\\' | b'"' | b' ' | b'-' | b'_' | b'.')
        {
            strategy = Strategy::Rfc2047;
            break;
        }

        bytes = &bytes[1..];
    }

    match strategy {
        Strategy::Plain => {
            w.write_str(value)?;
        }
        Strategy::Quoted => {
            w.write_char('"')?;
            // TODO: line folding
            w.write_str(value)?;
            w.write_char('"')?;
        }
        Strategy::QuotedEscaped => {
            w.write_char('"')?;
            // TODO: line folding
            utils::write_escaped(value, w)?;
            w.write_char('"')?;
        }
        Strategy::Rfc2047 => {
            rfc2047::encode(value, w)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn plain() {
        let mut s = String::new();
        let line_len = s.len();

        let mut w = EmailWriter::new(&mut s, line_len, false);
        encode("1234567890abcd", &mut w).unwrap();

        assert_eq!(s, "1234567890abcd");
    }

    #[test]
    fn quoted() {
        let mut s = String::new();
        let line_len = s.len();

        let mut w = EmailWriter::new(&mut s, line_len, false);
        encode("1234567890 abcd", &mut w).unwrap();

        assert_eq!(s, "\"1234567890 abcd\"");
    }

    #[test]
    fn quoted_escaped() {
        let mut s = String::new();
        let line_len = s.len();

        let mut w = EmailWriter::new(&mut s, line_len, false);
        encode("12345\\67890 ab\"cd", &mut w).unwrap();

        assert_eq!(s, "\"12345\\\\67890 ab\\\"cd\"");
    }

    #[test]
    fn rfc2047() {
        let mut s = String::new();
        let line_len = s.len();

        let mut w = EmailWriter::new(&mut s, line_len, false);
        encode("12345\\67890 perché ab\"cd", &mut w).unwrap();

        assert_eq!(s, "=?utf-8?b?MTIzNDVcNjc4OTAgcGVyY2jDqSBhYiJjZA==?=");
    }
}
