use errgonomic::handle;
use std::io;
use std::io::Write;
use thiserror::Error;

#[derive(clap::ValueEnum, Copy, Clone, Debug, Default)]
#[clap(rename_all = "kebab")]
pub enum OutputKind {
    Key,
    Value,
    #[default]
    KeyValue,
}

impl OutputKind {
    pub fn write(&self, writer: &mut impl Write, key: &[u8], value: &[u8], key_value_separator: &str) -> Result<(), OutputKindWriteError> {
        use OutputKind::*;
        use OutputKindWriteError::*;
        match self {
            Key => {
                handle!(writer.write_all(key), WriteAllFailed);
            }
            Value => {
                handle!(writer.write_all(value), WriteAllFailed);
            }
            KeyValue => {
                handle!(writer.write_all(key), WriteAllFailed);
                handle!(writer.write_all(key_value_separator.as_bytes()), WriteAllFailed);
                handle!(writer.write_all(value), WriteAllFailed);
            }
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum OutputKindWriteError {
    #[error("failed to write output")]
    WriteAllFailed { source: io::Error },
}
