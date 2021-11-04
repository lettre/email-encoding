use std::fmt::{self, Write};
use std::mem;

pub struct EmailWriter<'a> {
    writer: &'a mut dyn Write,
    line_len: usize,
    write_space_on_next_write: bool,
}

impl<'a> EmailWriter<'a> {
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

    pub(crate) fn new_line_no_initial_space(&mut self) -> fmt::Result {
        self.writer.write_str("\r\n")?;
        self.line_len = 0;
        self.write_space_on_next_write = false;

        Ok(())
    }

    pub fn space(&mut self) {
        debug_assert!(!self.write_space_on_next_write);
        self.write_space_on_next_write = true;
    }

    pub(crate) fn line_len(&self) -> usize {
        self.line_len
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

pub(super) fn str_is_ascii_alphanumeric(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_alphanumeric())
}

// TODO: function seems unused for now. Remove?
#[allow(dead_code)]
pub(super) fn str_is_ascii_alphanumeric_plus(s: &str) -> bool {
    s.chars().all(char_is_ascii_alphanumeric_plus)
}

pub(super) const fn char_is_ascii_alphanumeric_plus(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')
}

pub(super) fn str_is_ascii_printable(s: &str) -> bool {
    s.chars().all(char_is_ascii_printable)
}

const fn char_is_ascii_printable(c: char) -> bool {
    matches!(c, ' '..='~')
}

pub(super) fn write_escaped(s: &str, w: &mut EmailWriter) -> fmt::Result {
    debug_assert!(s.is_ascii());

    for b in s.bytes() {
        match b {
            b'\\' => {
                w.write_str("\\\\")?;
            }
            b'"' => {
                w.write_str("\\\"")?;
            }
            b => {
                w.write_char(char::from(b))?;
            }
        }
    }

    Ok(())
}

pub(super) fn truncate_to_char_boundary(s: &str, mut max: usize) -> &str {
    assert!(max <= s.len());

    while !s.is_char_boundary(max) {
        max -= 1;
    }
    &s[..max]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_ascii() {
        assert_eq!(truncate_to_char_boundary("12345678", 4), "1234");
    }

    #[test]
    fn truncate0_ascii() {
        assert_eq!(truncate_to_char_boundary("12345678", 0), "");
    }

    #[test]
    fn truncate_utf8() {
        assert_eq!(truncate_to_char_boundary("📬📬📬📬📬📬", 8), "📬📬");
    }

    #[test]
    fn truncate0_utf8() {
        assert_eq!(truncate_to_char_boundary("📬📬📬📬📬📬", 0), "");
    }

    #[test]
    fn truncate_boundary_utf8() {
        assert_eq!(truncate_to_char_boundary("📬📬📬📬📬📬", 9), "📬📬");
    }

    #[test]
    #[should_panic]
    fn truncate_out_of_bounds() {
        let _ = truncate_to_char_boundary("12345678", 16);
    }
}
