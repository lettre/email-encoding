use std::fmt::{self, Write};
use std::mem;

use super::{hex_encoding, utils, MAX_LINE_LEN};

pub fn encode(
    key: &str,
    mut value: &str,
    w: &mut dyn Write,
    line_len: &mut usize,
) -> fmt::Result {
    assert!(
        utils::str_is_ascii_alphanumeric(key),
        "`key` must only be composed of ascii alphanumeric chars"
    );
    assert!(
        key.len() + "*12*=utf-8'';".len() < MAX_LINE_LEN,
        "`key` must not be too long to cause the encoder to overflow the max line length"
    );

    let plain_combined_len = key.len() + "=".len() + value.len() + "\r\n".len();
    let quoted_plain_combined_len = key.len() + "=\"".len() + value.len() + "\"\r\n".len();
    if *line_len + plain_combined_len <= MAX_LINE_LEN {
        if utils::str_is_ascii_printable(value)
            && *line_len + quoted_plain_combined_len <= MAX_LINE_LEN
        {
            // Fits line an can be escaped and put into double quotes

            w.write_str(key)?;

            w.write_char('=')?;

            w.write_char('"')?;
            utils::write_escaped(value, w, line_len)?;
            w.write_char('"')?;

            *line_len += key.len() + "=\"".len() + "\"".len();

            return Ok(());
        }
    }

    w.write_str("\r\n")?;
    *line_len = 0;

    let mut i = 0_usize;
    let mut entered_encoding = false;
    loop {
        write!(w, " {}*{}*=", key, i)?;
        *line_len += " ".len() + key.len() + "*12*=".len();

        let remaining_len = MAX_LINE_LEN - *line_len - "\r\n".len();
        let value_ = utils::truncate_to_char_boundary(value, remaining_len.min(value.len()));

        if utils::str_is_ascii_alphanumeric_plus(value) {
            // No need for encoding

            w.write_str(value_)?;
            *line_len += value_.len();

            value = &value[value_.len()..];
        } else {
            // Encode

            if !mem::replace(&mut entered_encoding, true) {
                w.write_str("utf-8''")?;
                *line_len += "utf-8''".len();
            }

            while *line_len < MAX_LINE_LEN - "=xx=xx=xx=xx;\r\n".len() {
                match value.chars().next() {
                    Some(c) => {
                        hex_encoding::percent_encode_char(w, c, line_len)?;
                        value = &value[c.len_utf8()..];
                    }
                    None => {
                        break;
                    }
                }
            }
        }

        if !value.is_empty() {
            // End of line
            w.write_str(";\r\n")?;
            *line_len = 0;
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
        let mut s = "Content-Disposition: attachment;\r\n ".to_string();
        let mut line_len = 1;

        encode("filename", "", &mut s, &mut line_len).unwrap();

        assert_eq!(
            s,
            concat!("Content-Disposition: attachment;\r\n", " filename=\"\"")
        );
    }

    #[test]
    fn parameter() {
        let mut s = "Content-Disposition: attachment;\r\n ".to_string();
        let mut line_len = 1;

        encode("filename", "duck.txt", &mut s, &mut line_len).unwrap();

        assert_eq!(
            s,
            concat!("Content-Disposition: attachment;\r\n", " filename=\"duck.txt\"")
        );
    }

    #[test]
    fn parameter_to_escape() {
        let mut s = "Content-Disposition: attachment;\r\n ".to_string();
        let mut line_len = 1;

        encode("filename", "du\"ck\\.txt", &mut s, &mut line_len).unwrap();

        assert_eq!(
            s,
            concat!(
                "Content-Disposition: attachment;\r\n",
                " filename=\"du\\\"ck\\\\.txt\""
            )
        );
    }

    #[test]
    fn parameter_long() {
        let mut s = "Content-Disposition: attachment;".to_string();
        let mut line_len = s.len();

        encode(
            "filename",
            "a-fairly-long-filename-just-to-see-what-happens-when-we-encode-it-will-the-client-be-able-to-handle-it.txt",
            &mut s,
            &mut line_len,
        )
        .unwrap();

        assert_eq!(
            s,
            concat!(
                "Content-Disposition: attachment;\r\n",
                " filename*0*=a-fairly-long-filename-just-to-see-what-happens-when-we-enco;\r\n",
                " filename*1*=de-it-will-the-client-be-able-to-handle-it.txt"
            )
        );
    }

    #[test]
    fn parameter_special() {
        let mut s = "Content-Disposition: attachment;".to_string();
        let mut line_len = s.len();

        encode("filename", "caffè.txt", &mut s, &mut line_len).unwrap();

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
        let mut line_len = s.len();

        encode(
            "filename",
            "testing-to-see-what-happens-when-📕📕📕📕📕📕📕📕📕📕📕-are-placed-on-the-boundary.txt",
            &mut s,
            &mut line_len,
        )
        .unwrap();

        assert_eq!(
            s,
            concat!(
                "Content-Disposition: attachment;\r\n",
                " filename*0*=utf-8''testing-to-see-what-happens-when-%F0%9F%93%95;\r\n",
                " filename*1*=%F0%9F%93%95%F0%9F%93%95%F0%9F%93%95%F0%9F%93%95;\r\n",
                " filename*2*=%F0%9F%93%95%F0%9F%93%95%F0%9F%93%95%F0%9F%93%95;\r\n",
                " filename*3*=%F0%9F%93%95%F0%9F%93%95-are-placed-on-the-boun;\r\n",
                " filename*4*=dary.txt"
            )
        );
    }
}
