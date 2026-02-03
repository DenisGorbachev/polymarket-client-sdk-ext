use crate::{ClobMarket as MarketType, PrefixKind, TranscodeFormat, TranscodeFormatDecodeError, TranscodeFormatEncodeError, TranscodeTyp};
use core::num::TryFromIntError;
use errgonomic::handle;
use std::io::{self, Read, Write, stdin, stdout};
use std::process::ExitCode;
use thiserror::Error;

#[derive(clap::Parser, Clone, Debug)]
pub struct TranscodeCommand {
    #[arg(long, short = 'i', value_enum)]
    pub input: TranscodeFormat,
    #[arg(long, short = 'o', value_enum)]
    pub output: TranscodeFormat,
    #[arg(long, short = 'p', value_enum)]
    pub prefix: Option<PrefixKind>,
    #[arg(long, short = 's')]
    pub suffix: Option<String>,
    #[arg(long = "type", value_enum)]
    pub typ: TranscodeTyp,
}

impl TranscodeCommand {
    pub async fn run(self) -> Result<ExitCode, TranscodeCommandRunError> {
        let Self {
            input,
            output,
            prefix,
            suffix,
            typ,
        } = self;
        let suffix_bytes = suffix.map(String::into_bytes);
        let mut reader = stdin().lock();
        let mut writer = stdout().lock();
        let mut iter = read_items(&mut reader);

        iter.try_for_each(|item_result| {
            use TranscodeCommandRunError::*;
            let item = handle!(item_result, ReadItemFailed);
            handle!(transcode_item(input, output, typ, prefix, suffix_bytes.as_deref(), item, &mut writer), TranscodeItemFailed);
            Ok(())
        })
        .map(|()| ExitCode::SUCCESS)
    }
}

#[derive(Error, Debug)]
pub enum TranscodeCommandRunError {
    #[error("failed to read input item")]
    ReadItemFailed { source: ReadItemError },
    #[error("failed to transcode input item")]
    TranscodeItemFailed { source: TranscodeItemError },
}

pub fn read_items(reader: &mut impl Read) -> impl Iterator<Item = Result<Vec<u8>, ReadItemError>> + '_ {
    std::iter::from_fn(move || match read_item(reader) {
        Ok(Some(item)) => Some(Ok(item)),
        Ok(None) => None,
        Err(error) => Some(Err(error)),
    })
}

pub fn read_item(reader: &mut impl Read) -> Result<Option<Vec<u8>>, ReadItemError> {
    use ReadItemError::*;
    let len_opt = handle!(read_len_prefix(reader), ReadLenPrefixFailed);
    let Some(len) = len_opt else {
        return Ok(None);
    };
    if len == 0 {
        return Ok(Some(Vec::new()));
    }
    let len_usize = handle!(usize::try_from(len), TryFromFailed, len);
    let bytes = handle!(read_item_bytes(reader, len_usize), ReadItemBytesFailed, len);
    Ok(Some(bytes))
}

#[derive(Error, Debug)]
pub enum ReadItemError {
    #[error("failed to read length prefix")]
    ReadLenPrefixFailed { source: ReadLenPrefixError },
    #[error("failed to convert item length '{len}' to usize")]
    TryFromFailed { source: TryFromIntError, len: u64 },
    #[error("failed to read item bytes for length '{len}'")]
    ReadItemBytesFailed { source: ReadItemBytesError, len: u64 },
}

pub fn read_len_prefix(reader: &mut impl Read) -> Result<Option<u64>, ReadLenPrefixError> {
    use ReadLenPrefixError::*;
    let mut len_bytes = [0_u8; 8];
    let mut offset = 0usize;
    while offset < len_bytes.len() {
        let read = handle!(reader.read(&mut len_bytes[offset..]), ReadFailed, offset);
        if read == 0 {
            if offset == 0 {
                return Ok(None);
            }
            return Err(UnexpectedEofFailed {
                offset,
            });
        }
        offset += read;
    }
    Ok(Some(u64::from_le_bytes(len_bytes)))
}

#[derive(Error, Debug)]
pub enum ReadLenPrefixError {
    #[error("failed to read length prefix at offset '{offset}'")]
    ReadFailed { source: io::Error, offset: usize },
    #[error("unexpected EOF while reading length prefix after '{offset}' bytes")]
    UnexpectedEofFailed { offset: usize },
}

pub fn read_item_bytes(reader: &mut impl Read, len: usize) -> Result<Vec<u8>, ReadItemBytesError> {
    use ReadItemBytesError::*;
    let mut buf = vec![0_u8; len];
    handle!(reader.read_exact(&mut buf), ReadExactFailed);
    Ok(buf)
}

#[derive(Error, Debug)]
pub enum ReadItemBytesError {
    #[error("failed to read item bytes")]
    ReadExactFailed { source: io::Error },
}

pub fn transcode_item(input_format: TranscodeFormat, output_format: TranscodeFormat, typ: TranscodeTyp, prefix: Option<PrefixKind>, suffix: Option<&[u8]>, item_bytes: Vec<u8>, writer: &mut impl Write) -> Result<(), TranscodeItemError> {
    use TranscodeItemError::*;
    use TranscodeTyp::*;
    if item_bytes.is_empty() {
        return Ok(());
    }
    match typ {
        Market => {
            let market = handle!(input_format.decode::<MarketType>(item_bytes), DecodeFailed);
            let output_bytes = handle!(output_format.encode(market), EncodeFailed);
            if let Some(prefix) = prefix {
                handle!(prefix.write(&output_bytes, writer), WriteFailed);
            }
            handle!(writer.write_all(&output_bytes), WriteAllFailed);
            if let Some(suffix) = suffix {
                handle!(writer.write_all(suffix), WriteAllFailed);
            }
        }
    }
    Ok(())
}

#[derive(Error, Debug)]
pub enum TranscodeItemError {
    #[error("failed to decode input item")]
    DecodeFailed { source: TranscodeFormatDecodeError },
    #[error("failed to encode output item")]
    EncodeFailed { source: TranscodeFormatEncodeError<MarketType> },
    #[error("failed to write output prefix")]
    WriteFailed { source: io::Error },
    #[error("failed to write output bytes")]
    WriteAllFailed { source: io::Error },
}
