use std::ops::Deref;

pub mod base64;
mod chooser;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Encoding {
    SevenBit,
    EightBit,
    QuotedPrintable,
    Base64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum StrOrBytes<'a> {
    Str(&'a str),
    Bytes(&'a [u8]),
}

impl<'a> From<&'a str> for StrOrBytes<'a> {
    fn from(s: &'a str) -> Self {
        Self::Str(s)
    }
}

impl<'a> From<&'a [u8]> for StrOrBytes<'a> {
    fn from(s: &'a [u8]) -> Self {
        Self::Bytes(s)
    }
}

impl<'a, const N: usize> From<&'a [u8; N]> for StrOrBytes<'a> {
    fn from(s: &'a [u8; N]) -> Self {
        Self::Bytes(s)
    }
}

impl<'a> Deref for StrOrBytes<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Str(s) => s.as_bytes(),
            Self::Bytes(b) => b,
        }
    }
}
