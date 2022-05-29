use std::fmt::{self, Write};
use std::str;

const LINE_LEN: usize = 76;
const CRLF: &str = "\r\n";

pub fn encode(b: &[u8], w: &mut dyn Write) -> fmt::Result {
    let mut buf = [0; LINE_LEN];

    const CHUNK_LEN: usize = LINE_LEN / 4 * 3;
    for chunk in b.chunks(CHUNK_LEN) {
        let len = ::base64::encode_config_slice(chunk, ::base64::STANDARD, &mut buf);

        w.write_str(str::from_utf8(&buf[..len]).expect("base64 produced an invalid encode"))?;
        if chunk.len() == CHUNK_LEN {
            w.write_str(CRLF)?;
        }
    }

    Ok(())
}

pub fn encoded_len(input_len: usize) -> usize {
    let mut base64_len = input_len / 3 * 4;
    if input_len % 3 != 0 {
        base64_len += 4 - base64_len % 4;
    }
    let crlf_len = base64_len / LINE_LEN * CRLF.len();
    base64_len + crlf_len
}

#[cfg(test)]
mod tests {
    use super::{encode, encoded_len};

    #[test]
    fn empty() {
        let input = b"";
        let mut output = String::new();

        encode(input, &mut output).unwrap();

        assert_eq!(output, "");
        assert_eq!(output.len(), encoded_len(input.len()));
    }

    #[test]
    fn oneline() {
        let input = b"012";
        let mut output = String::new();

        encode(input, &mut output).unwrap();

        assert_eq!(output, "MDEy");
        assert_eq!(output.len(), encoded_len(input.len()));
    }

    #[test]
    fn oneline_padded() {
        let input = b"0123";
        let mut output = String::new();

        encode(input, &mut output).unwrap();

        assert_eq!(output, "MDEyMw==");
        assert_eq!(output.len(), encoded_len(input.len()));
    }

    #[test]
    fn multiline() {
        let input =
            b"012345678998765432100123456789987654321001234567899876543210012345678998765432100";
        let mut output = String::new();

        encode(input, &mut output).unwrap();

        assert_eq!(
            output,
            concat!(
                "MDEyMzQ1Njc4OTk4NzY1NDMyMTAwMTIzNDU2Nzg5OTg3NjU0MzIxMDAxMjM0NTY3ODk5ODc2NTQz\r\n",
                "MjEwMDEyMzQ1Njc4OTk4NzY1NDMyMTAw"
            )
        );
        assert_eq!(output.len(), encoded_len(input.len()));
    }

    #[test]
    fn multiline_padded() {
        let input =
            b"01234567899876543210012345678998765432100123456789987654321001234567899876543210";
        let mut output = String::new();

        encode(input, &mut output).unwrap();

        assert_eq!(
            output,
            concat!(
                "MDEyMzQ1Njc4OTk4NzY1NDMyMTAwMTIzNDU2Nzg5OTg3NjU0MzIxMDAxMjM0NTY3ODk5ODc2NTQz\r\n",
                "MjEwMDEyMzQ1Njc4OTk4NzY1NDMyMTA="
            )
        );
        assert_eq!(output.len(), encoded_len(input.len()));
    }
}
