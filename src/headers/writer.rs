//! Utilities for writing email headers to a [`Write`]r.
//!
//! [`Write`]: core::fmt::Write

use core::fmt::{self, Write};

use super::MAX_LINE_LEN;

/// Wrapper around [`Write`] that remembers the length of the
/// last line written to it.
///
/// [`Write`]: core::fmt::Write
pub struct EmailWriter<'a> {
    writer: &'a mut dyn Write,
    line_len: usize,
    spaces: usize,
    can_go_to_new_line_now: bool,
    /// Whether the current line holds nothing but white space so far
    line_is_white_space: bool,
}

impl<'a> EmailWriter<'a> {
    /// Construct a new `EmailWriter`.
    ///
    /// * `line_len` is the length of the last line in `writer`. Unless
    ///   it's `0` the line is taken to hold something other than white
    ///   space, like the name of the header.
    /// * `spaces` the number of spaces that must be written before
    ///   the next write.
    /// * `can_go_to_new_line_now` is whether the current line can
    ///   be wrapped now or not.
    pub fn new(
        writer: &'a mut dyn Write,
        line_len: usize,
        spaces: usize,
        can_go_to_new_line_now: bool,
    ) -> Self {
        Self {
            writer,
            line_len,
            spaces,
            can_go_to_new_line_now,
            line_is_white_space: line_len == 0,
        }
    }

    /// Go to a new line and reset the `line_len` to `0`.
    pub fn new_line(&mut self) -> fmt::Result {
        self.writer.write_str("\r\n")?;
        self.line_len = 0;
        self.can_go_to_new_line_now = false;
        self.line_is_white_space = true;

        Ok(())
    }

    /// Write a space which _might_ get wrapped to a new line on the next write.
    pub fn space(&mut self) {
        self.spaces += 1;
    }

    /// Forget all buffered spaces
    pub(super) fn forget_spaces(&mut self) {
        self.spaces = 0;
    }

    pub(super) fn has_spaces(&mut self) -> bool {
        self.spaces >= 1
    }

    /// Get the length in bytes of the last line written to the inner writer.
    pub fn line_len(&self) -> usize {
        self.line_len
    }

    /// Get the length in bytes of the last line written to the inner writer
    /// plus the spaces which might be written to in on the next write call.
    pub fn projected_line_len(&self) -> usize {
        self.line_len + self.spaces
    }

    /// Get a [`Write`]r which automatically line folds text written to it.
    ///
    /// [`Write`]: core::fmt::Write
    pub fn folding<'b>(&'b mut self) -> FoldingEmailWriter<'a, 'b> {
        FoldingEmailWriter { writer: self }
    }

    /// Take note of something other than spaces having been written
    fn wrote(&mut self, white_space: bool) {
        if !white_space {
            self.line_is_white_space = false;
        }

        // Going to a new line after nothing but white space
        // would leave a line that is made of white space only
        if !self.line_is_white_space {
            self.can_go_to_new_line_now = true;
        }
    }

    fn write_spaces(&mut self) -> fmt::Result {
        while self.spaces > 0 {
            self.writer.write_char(' ')?;
            self.line_len += 1;
            self.spaces -= 1;
        }

        Ok(())
    }
}

impl Write for EmailWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_spaces()?;

        let s_after = s.trim_end_matches(' ');
        self.spaces += s.len() - s_after.len();

        if !s_after.is_empty() {
            self.writer.write_str(s_after)?;
            self.line_len += s_after.len();
            self.wrote(is_white_space(s_after));
        }

        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        if c == ' ' {
            self.spaces += 1;
        } else {
            self.write_spaces()?;
            self.wrote(c == '\t');

            self.writer.write_char(c)?;
            self.line_len += c.len_utf8();
        }

        Ok(())
    }
}

/// Whether `s` is made of nothing but spaces and tabs
fn is_white_space(s: &str) -> bool {
    s.bytes().all(|b| matches!(b, b' ' | b'\t'))
}

impl Drop for EmailWriter<'_> {
    fn drop(&mut self) {
        let _ = self.write_spaces();
    }
}

/// Wrapper around [`Write`] that remembers the length of the
/// last line and automatically line folds text written to it.
///
/// [`Write`]: core::fmt::Write
pub struct FoldingEmailWriter<'a, 'b> {
    writer: &'b mut EmailWriter<'a>,
}

impl Write for FoldingEmailWriter<'_, '_> {
    fn write_str(&mut self, mut s: &str) -> fmt::Result {
        while !s.is_empty() {
            if s.starts_with(' ') {
                self.writer.space();
                s = &s[1..];
                continue;
            }

            let (start, end) = s.find(' ').map_or((s, ""), |i| s.split_at(i));

            if self.writer.can_go_to_new_line_now
                && self.writer.spaces >= 1
                && (self.writer.projected_line_len() + start.len()) > MAX_LINE_LEN
                // A new line must not start with nothing but white space,
                // as it could then end without anything else on it
                && !is_white_space(start)
            {
                self.writer.new_line()?;
            }

            self.writer.write_str(start)?;
            s = end;
        }

        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        if c == ' ' {
            self.writer.spaces += 1;
        } else {
            self.write_str(c.encode_utf8(&mut [0u8; 4]))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloc::borrow::ToOwned;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn wrap_immediate() {
        let mut s =
            "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 0, true);
            for _ in 0..16 {
                w.folding().write_str("0123456789").unwrap();
            }
        }

        assert_eq!(
            s,
            "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789",
        );
    }

    #[test]
    fn wrap_keeping_final_whitespace() {
        let mut s = "Subject: AAAAAAAAAAAAAA".to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 1, true);
            w.folding().write_str("12345 ").unwrap();
            w.new_line().unwrap();
            w.folding().write_str("12345").unwrap();
        }

        assert_eq!(s, concat!("Subject: AAAAAAAAAAAAAA 12345\r\n", " 12345"));
    }

    #[test]
    fn catch_space() {
        let mut s = "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 1, true);
            w.folding().write_str("BBB ").unwrap();
            w.folding().write_str("CCCCCCCCCCCCC").unwrap();
        }

        assert_eq!(
            s,
            concat!(
                "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA BBB\r\n",
                " CCCCCCCCCCCCC"
            )
        );
    }

    #[test]
    fn catch_spaces() {
        let mut s = "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 1, true);
            w.folding().write_str("BBB   ").unwrap();
            w.folding().write_str("CCCCCCCCCCCCC").unwrap();
        }

        assert_eq!(
            s,
            concat!(
                "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA BBB\r\n",
                "   CCCCCCCCCCCCC"
            )
        );
    }

    #[test]
    fn explicit_space() {
        let mut s = "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 1, true);
            w.folding().write_str("BBB").unwrap();
            w.space();
            w.folding().write_str("CCCCCCCCCCCCC").unwrap();
        }

        assert_eq!(
            s,
            concat!(
                "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA BBB\r\n",
                " CCCCCCCCCCCCC"
            )
        );
    }

    #[test]
    fn explicit_spaces() {
        let mut s = "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 1, true);
            w.folding().write_str("BBB").unwrap();
            w.space();
            w.write_char(' ').unwrap();
            w.space();
            w.folding().write_str("CCCCCCCCCCCCC").unwrap();
        }

        assert_eq!(
            s,
            concat!(
                "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA BBB\r\n",
                "   CCCCCCCCCCCCC"
            )
        );
    }

    #[test]
    fn optional_breakpoint() {
        let mut s = "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
            .to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 0, true);
            w.space();
            w.folding().write_str("BBBBBBBBBB").unwrap();
            w.space();
            w.folding().write_str("CCCCCCCCCC").unwrap();
        }

        assert_eq!(
            s,
            concat!(
                "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\r\n",
                " BBBBBBBBBB CCCCCCCCCC",
            )
        );
    }

    #[test]
    fn tab_between_spaces() {
        let mut s = "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
            .to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 0, true);
            w.folding()
                .write_str(
                    " \t BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
                )
                .unwrap();
            w.folding().write_str(" \t").unwrap();
        }

        // No line is made of white space only
        assert_eq!(
            s,
            concat!(
                "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA \t\r\n",
                " BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB \t",
            )
        );
    }

    #[test]
    fn tab_after_new_line() {
        let mut s = "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
            .to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 0, true);
            w.new_line().unwrap();
            w.space();
            w.write_char('\t').unwrap();
            w.folding()
                .write_str(
                    " BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
                )
                .unwrap();
        }

        // The white space written after going to a new line isn't left alone on it
        assert_eq!(
            s,
            concat!(
                "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\r\n",
                " \t BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
            )
        );
    }

    #[test]
    fn tab_after_header_name() {
        let mut s = "Subject: ".to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 0, false);
            w.folding()
                .write_str(
                    "\t BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
                )
                .unwrap();
        }

        // The line of the header name isn't made of white space only
        assert_eq!(
            s,
            concat!(
                "Subject: \t\r\n",
                " BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
            )
        );
    }

    #[test]
    fn double_spaces_issue_949() {
        let mut s = "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA ".to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 0, true);
            w.folding().write_str("BBBBBBBBBBBBB ").unwrap();
            crate::headers::rfc2047::encode("sélection", &mut w).unwrap();
        }

        assert_eq!(
            s,
            concat!(
                "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA BBBBBBBBBBBBB\r\n",
                " =?utf-8?b?c8OpbGVjdGlvbg==?=",
            )
        );
    }

    #[test]
    fn double_spaces_issue_949_no_space() {
        let mut s = "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA ".to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 0, true);
            w.folding().write_str("BBBBBBBBBBBBBBB").unwrap();
            crate::headers::rfc2047::encode("sélection", &mut w).unwrap();
        }

        assert_eq!(
            s,
            concat!(
                "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA BBBBBBBBBBBBBBB=?utf-8?b?cw==?=\r\n",
                " =?utf-8?b?w6lsZWN0aW9u?=",
            )
        );
    }
}
