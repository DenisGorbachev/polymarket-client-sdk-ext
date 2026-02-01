use std::io::{self, Write};

#[derive(clap::ValueEnum, Copy, Clone, Debug)]
#[clap(rename_all = "kebab")]
pub enum PrefixKind {
    LenU64Le,
    LenU64Be,
}

impl PrefixKind {
    pub fn write<I: AsRef<[u8]>, W: Write>(self, item: &I, writer: &mut W) -> io::Result<()> {
        use PrefixKind::*;
        let item = item.as_ref();
        let len = item.len() as u64;
        let bytes = match self {
            LenU64Le => len.to_le_bytes(),
            LenU64Be => len.to_be_bytes(),
        };
        writer.write_all(&bytes)
    }
}
