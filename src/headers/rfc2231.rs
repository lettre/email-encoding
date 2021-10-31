use std::fmt::{self, Write};
use std::mem;

use super::{hex_encoding, utils, EmailWriter, MAX_LINE_LEN};

pub fn encode(key: &str, mut value: &str, w: &mut EmailWriter) -> fmt::Result {
    assert!(
        utils::str_is_ascii_alphanumeric(key),
        "`key` must only be composed of ascii alphanumeric chars"
    );
    assert!(
        key.len() + "*12*=utf-8'';".len() < MAX_LINE_LEN,
        "`key` must not be too long to cause the encoder to overflow the max line length"
    );

    let quoted_plain_combined_len = key.len() + "=\"".len() + value.len() + "\"\r\n".len();
    if utils::str_is_ascii_printable(value)
        && w.line_len() + quoted_plain_combined_len <= MAX_LINE_LEN
    {
        // Fits line an can be escaped and put into double quotes

        w.write_str(key)?;

        w.write_char('=')?;

        w.write_char('"')?;
        utils::write_escaped(value, w)?;
        w.write_char('"')?;

        return Ok(());
    }

    w.new_line_no_initial_space()?;

    let mut i = 0_usize;
    let mut entered_encoding = false;
    loop {
        write!(w, " {}*{}*=", key, i)?;

        let remaining_len = MAX_LINE_LEN - w.line_len() - "\r\n".len();
        let value_ = utils::truncate_to_char_boundary(value, remaining_len.min(value.len()));

        if utils::str_is_ascii_alphanumeric_plus(value) {
            // No need for encoding

            w.write_str(value_)?;

            value = &value[value_.len()..];
        } else {
            // Encode

            if !mem::replace(&mut entered_encoding, true) {
                w.write_str("utf-8''")?;
            }

            let mut chars = value.chars();
            while w.line_len() < MAX_LINE_LEN - "=xx=xx=xx=xx;\r\n".len() {
                match chars.next() {
                    Some(c) => {
                        hex_encoding::percent_encode_char(w, c)?;
                        value = chars.as_str();
                    }
                    None => {
                        break;
                    }
                }
            }
        }

        if !value.is_empty() {
            // End of line
            w.write_char(';')?;
            w.new_line_no_initial_space()?;
        } else {
            // End of value
            break;
        }

        i += 1;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn empty() {
        let mut s = "Content-Disposition: attachment;".to_string();
        let line_len = 1;

        let mut w = EmailWriter::new(&mut s, line_len, true);
        encode("filename", "", &mut w).unwrap();

        assert_eq!(s, concat!("Content-Disposition: attachment; filename=\"\""));
    }

    #[test]
    fn parameter() {
        let mut s = "Content-Disposition: attachment;".to_string();
        let line_len = 1;

        let mut w = EmailWriter::new(&mut s, line_len, true);
        encode("filename", "duck.txt", &mut w).unwrap();

        assert_eq!(
            s,
            concat!("Content-Disposition: attachment; filename=\"duck.txt\"")
        );
    }

    #[test]
    fn parameter_to_escape() {
        let mut s = "Content-Disposition: attachment;".to_string();
        let line_len = 1;

        let mut w = EmailWriter::new(&mut s, line_len, true);
        encode("filename", "du\"ck\\.txt", &mut w).unwrap();

        assert_eq!(
            s,
            concat!("Content-Disposition: attachment; filename=\"du\\\"ck\\\\.txt\"")
        );
    }

    #[test]
    fn parameter_long() {
        let mut s = "Content-Disposition: attachment;".to_string();
        let line_len = s.len();

        let mut w = EmailWriter::new(&mut s, line_len, true);
        encode(
            "filename",
            "a-fairly-long-filename-just-to-see-what-happens-when-we-encode-it-will-the-client-be-able-to-handle-it.txt",
            &mut w,
        )
        .unwrap();

        assert_eq!(
            s,
            concat!(
                "Content-Disposition: attachment;\r\n",
                " filename*0*=a-fairly-long-filename-just-to-see-what-happens-when-we-encod;\r\n",
                " filename*1*=e-it-will-the-client-be-able-to-handle-it.txt"
            )
        );
    }

    #[test]
    fn parameter_special() {
        let mut s = "Content-Disposition: attachment;".to_string();
        let line_len = s.len();

        let mut w = EmailWriter::new(&mut s, line_len, true);
        encode("filename", "caffè.txt", &mut w).unwrap();

        assert_eq!(
            s,
            concat!(
                "Content-Disposition: attachment;\r\n",
                " filename*0*=utf-8''caff%C3%A8.txt"
            )
        );
    }

    #[test]
    fn parameter_special_long() {
        let mut s = "Content-Disposition: attachment;".to_string();
        let line_len = s.len();

        let mut w = EmailWriter::new(&mut s, line_len, true);
        encode(
            "filename",
            "testing-to-see-what-happens-when-📕📕📕📕📕📕📕📕📕📕📕-are-placed-on-the-boundary.txt",
            &mut w,
        )
        .unwrap();

        assert_eq!(
            s,
            concat!(
                "Content-Disposition: attachment;\r\n",
                " filename*0*=utf-8''testing-to-see-what-happens-when-%F0%9F%93%95;\r\n",
                " filename*1*=%F0%9F%93%95%F0%9F%93%95%F0%9F%93%95%F0%9F%93%95;\r\n",
                " filename*2*=%F0%9F%93%95%F0%9F%93%95%F0%9F%93%95%F0%9F%93%95;\r\n",
                " filename*3*=%F0%9F%93%95%F0%9F%93%95-are-placed-on-the-bound;\r\n",
                " filename*4*=ary.txt"
            )
        );
    }

    #[test]
    fn parameter_special_long_part2() {
        let mut s = "Content-Disposition: attachment;".to_string();
        let line_len = s.len();

        let mut w = EmailWriter::new(&mut s, line_len, true);
        encode(
            "filename",
            "testing-to-see-what-happens-when-books-are-placed-in-the-second-part-📕📕📕📕📕📕📕📕📕📕📕.txt",
            &mut w,
        )
        .unwrap();

        assert_eq!(
            s,
            concat!(
                "Content-Disposition: attachment;\r\n",
                " filename*0*=utf-8''testing-to-see-what-happens-when-books-ar;\r\n",
                " filename*1*=e-placed-in-the-second-part-%F0%9F%93%95%F0%9F%93%95;\r\n",
                " filename*2*=%F0%9F%93%95%F0%9F%93%95%F0%9F%93%95%F0%9F%93%95;\r\n",
                " filename*3*=%F0%9F%93%95%F0%9F%93%95%F0%9F%93%95%F0%9F%93%95;\r\n",
                " filename*4*=%F0%9F%93%95.txt"
            )
        );
    }
}
