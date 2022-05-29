use std::mem;

use super::{Encoding, StrOrBytes};

enum InputKind {
    Ascii,
    Utf8,
    Binary,
}

impl<'a> StrOrBytes<'a> {
    fn kind(&self) -> InputKind {
        if self.is_ascii() {
            InputKind::Ascii
        } else {
            match self {
                Self::Str(_) => InputKind::Utf8,
                Self::Bytes(_) => InputKind::Binary,
            }
        }
    }
}

impl Encoding {
    pub fn choose<'a>(input: impl Into<StrOrBytes<'a>>, supports_utf8: bool) -> Self {
        let input = input.into();
        Self::choose_impl(input, supports_utf8)
    }

    fn choose_impl(input: StrOrBytes<'_>, supports_utf8: bool) -> Self {
        let line_too_long = line_too_long(&input);

        match (input.kind(), line_too_long, supports_utf8) {
            (InputKind::Ascii, false, _) => {
                // Input is ascii and fits the maximum line length
                Self::SevenBit
            }
            (InputKind::Ascii, true, _) => {
                // Input is ascii but doesn't fix the maximum line length
                quoted_printable_or_base64(&input)
            }
            (InputKind::Utf8, false, true) => {
                // Input is utf-8, line fits, the server supports it
                Self::EightBit
            }
            (InputKind::Utf8, true, true) => {
                // Input is utf-8, line doesn't fit, the server supports it
                quoted_printable_or_base64(&input)
            }
            (InputKind::Utf8, _, false) => {
                // Input is utf-8, the server doesn't support it
                quoted_printable_or_base64(&input)
            }
            (InputKind::Binary, _, _) => {
                // Input is binary
                Self::Base64
            }
        }
    }
}

fn line_too_long(b: &[u8]) -> bool {
    let mut last = 0;
    memchr::memchr_iter(b'\n', b).any(|i| {
        let last_ = mem::replace(&mut last, i);
        (i - last_) >= 76
    }) || (b.len() - last) >= 76
}

fn quoted_printable_or_base64(b: &[u8]) -> Encoding {
    if quoted_printable_efficient(b) {
        Encoding::QuotedPrintable
    } else {
        Encoding::Base64
    }
}

fn quoted_printable_efficient(b: &[u8]) -> bool {
    let requiring_escaping = b
        .iter()
        .filter(|&b| !matches!(b, b'\t' | b' '..=b'~'))
        .count();
    requiring_escaping <= (b.len() / 3) // 33.33% or less
}

#[cfg(test)]
mod tests {
    use super::{line_too_long, Encoding};

    #[test]
    fn ascii_short_str() {
        let input = "0123";

        assert_eq!(Encoding::choose(input, false), Encoding::SevenBit);
    }

    #[test]
    fn ascii_long_str() {
        let input = concat!(
            "0123\n",
            "01234567899876543210012345678998765432100123456789987654321001234567899876543210\n",
            "4567"
        );

        assert_eq!(Encoding::choose(input, false), Encoding::QuotedPrintable);
    }

    #[test]
    fn ascii_short_binary() {
        let input = b"0123";

        assert_eq!(Encoding::choose(input, false), Encoding::SevenBit);
    }

    #[test]
    fn ascii_long_binary() {
        let input = concat!(
            "0123\n",
            "01234567899876543210012345678998765432100123456789987654321001234567899876543210\n",
            "4567"
        )
        .as_bytes();

        assert_eq!(Encoding::choose(input, false), Encoding::QuotedPrintable);
    }

    #[test]
    fn utf8_short_str_supported() {
        let input = "0123 📬";

        assert_eq!(Encoding::choose(input, true), Encoding::EightBit);
    }

    #[test]
    fn utf8_short_str_unsupported_efficient() {
        let input = "01234567899876543210 📬";

        assert_eq!(Encoding::choose(input, false), Encoding::QuotedPrintable);
    }

    #[test]
    fn utf8_short_str_unsupported_inefficient() {
        let input = "0123 📬";

        assert_eq!(Encoding::choose(input, false), Encoding::Base64);
    }

    #[test]
    fn utf8_long_str_efficient() {
        let input =
            "01234567899876543210012345678998765432100123456789987654321001234567899876543210";

        assert_eq!(Encoding::choose(input, true), Encoding::QuotedPrintable);
    }

    #[test]
    fn utf8_long_str_inefficient() {
        let input = "0123 📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬📬";

        assert_eq!(Encoding::choose(input, true), Encoding::Base64);
    }

    #[test]
    fn binary() {
        let input = &[255, 234, b'A', b'C', 210];

        assert_eq!(Encoding::choose(input, false), Encoding::Base64);
    }

    #[test]
    fn not_too_long_oneline() {
        let input = b"0123";

        assert!(!line_too_long(input));
    }

    #[test]
    fn not_too_long_multiline() {
        let input = concat!("0123\n", "4567").as_bytes();

        assert!(!line_too_long(input));
    }

    #[test]
    fn too_long_oneline() {
        let input =
            b"01234567899876543210012345678998765432100123456789987654321001234567899876543210";

        assert!(line_too_long(input));
    }

    #[test]
    fn too_long_multiline() {
        let input = concat!(
            "0123\n",
            "01234567899876543210012345678998765432100123456789987654321001234567899876543210\n",
            "4567"
        )
        .as_bytes();

        assert!(line_too_long(input));
    }
}
