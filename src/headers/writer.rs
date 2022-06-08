//! Utilities for writing email headers to a [`Write`]r.
//!
//! [`Write`]: std::fmt::Write

use std::fmt::{self, Write};
use std::mem;

use super::MAX_LINE_LEN;

/// Wrapper around [`Write`] that remembers the length of the
/// last line written to it.
///
/// [`Write`]: std::fmt::Write
pub struct EmailWriter<'a> {
    writer: &'a mut dyn Write,
    line_len: usize,
    write_space_on_next_write: bool,
}

impl<'a> EmailWriter<'a> {
    /// Construct a new `EmailWriter`.
    ///
    /// * `line_len` is the length of the last line in `writer`.
    /// * `write_space_on_next_write` is whether the writer must
    ///   go to a new line when writing it's first character
    pub fn new(
        writer: &'a mut dyn Write,
        line_len: usize,
        write_space_on_next_write: bool,
    ) -> Self {
        Self {
            writer,
            line_len,
            write_space_on_next_write,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn new_line(&mut self) -> fmt::Result {
        self.writer.write_str("\r\n ")?;
        self.line_len = 1;
        self.write_space_on_next_write = false;

        Ok(())
    }

    /// Go to a new line and reset the `line_len` to `0`.
    pub fn new_line_no_initial_space(&mut self) -> fmt::Result {
        self.writer.write_str("\r\n")?;
        self.line_len = 0;
        self.write_space_on_next_write = false;

        Ok(())
    }

    /// Write a space which _might_ get wrapped to a new line on the next write.
    ///
    /// This method shouldn't be called multiple times consecutively,
    /// and will panic if debug assertions are on.
    pub fn space(&mut self) {
        debug_assert!(!self.write_space_on_next_write);
        self.write_space_on_next_write = true;
    }

    /// Get the length in bytes of the last line written to the inner writer.
    pub fn line_len(&self) -> usize {
        self.line_len
    }

    /// Get a [`Write`]r which automatically line folds text written to it.
    ///
    /// [`Write`]: std::fmt::Write
    pub fn folding<'b>(&'b mut self) -> FoldingEmailWriter<'a, 'b> {
        FoldingEmailWriter { writer: self }
    }
}

impl<'a> Write for EmailWriter<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if mem::take(&mut self.write_space_on_next_write) {
            self.writer.write_char(' ')?;
            self.line_len += 1;
        }

        self.writer.write_str(s)?;
        self.line_len += s.len();

        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        if mem::take(&mut self.write_space_on_next_write) {
            self.writer.write_char(' ')?;
            self.line_len += 1;
        }

        self.writer.write_char(c)?;
        self.line_len += c.len_utf8();

        Ok(())
    }
}

/// Wrapper around [`Write`] that remembers the length of the
/// last line and automatically line folds text written to it.
///
/// [`Write`]: std::fmt::Write
pub struct FoldingEmailWriter<'a, 'b> {
    writer: &'b mut EmailWriter<'a>,
}

impl<'a, 'b> Write for FoldingEmailWriter<'a, 'b> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut first = true;

        for word in s.split(' ') {
            if !mem::take(&mut first) {
                self.writer.space();

                if (self.writer.line_len + word.len()) > MAX_LINE_LEN {
                    self.writer.new_line()?;
                }
            }

            self.writer.write_str(word)?;
        }

        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        self.write_str(c.encode_utf8(&mut [0u8; 4]))
    }
}
