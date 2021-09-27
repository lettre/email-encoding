use std::fmt::{self, Write};

use super::{hex, utils};

pub(super) fn percent_encode_char(
    w: &mut dyn Write,
    to_append: char,
    line_len: &mut usize,
) -> fmt::Result {
    encode_char(w, '%', to_append, line_len)
}

fn encode_char(
    w: &mut dyn Write,
    prefix: char,
    to_append: char,
    line_len: &mut usize,
) -> fmt::Result {
    if utils::char_is_ascii_alphanumeric_plus(to_append) {
        w.write_char(to_append)?;
        *line_len += 1;
    } else {
        let mut dst = [0; 4];
        let written = to_append.encode_utf8(&mut dst).len();

        encode_byte(w, prefix, dst[0], line_len)?;

        // Manually unrolled loop over `dst`
        if written >= 2 {
            encode_byte(w, prefix, dst[1], line_len)?;

            if written >= 3 {
                encode_byte(w, prefix, dst[2], line_len)?;

                if written >= 4 {
                    encode_byte(w, prefix, dst[3], line_len)?;
                }
            }
        }
    }

    Ok(())
}

fn encode_byte(
    w: &mut dyn Write,
    prefix: char,
    to_append: u8,
    line_len: &mut usize,
) -> fmt::Result {
    let chars = hex::encode_byte(to_append);
    w.write_char(prefix)?;
    w.write_char(char::from(chars[0]))?;
    w.write_char(char::from(chars[1]))?;
    *line_len += 3;

    Ok(())
}
