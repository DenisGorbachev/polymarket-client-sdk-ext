use errgonomic::handle;
use rkyv::api::high::{HighSerializer, HighValidator};
use rkyv::bytecheck::CheckBytes;
use rkyv::de::Pool;
use rkyv::rancor::Error as RkyvError;
use rkyv::rancor::Strategy;
use rkyv::ser::allocator::ArenaHandle;
use rkyv::util::AlignedVec;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(clap::ValueEnum, Copy, Clone, Debug)]
pub enum TranscodeFormat {
    #[value(name = "rkyv")]
    Rkyv,
    #[value(name = "serde_json")]
    SerdeJson,
}

impl TranscodeFormat {
    pub fn decode<T>(self, input: Vec<u8>) -> Result<T, TranscodeFormatDecodeError>
    where
        T: rkyv::Archive + for<'de> Deserialize<'de>,
        T::Archived: for<'a> CheckBytes<HighValidator<'a, RkyvError>> + rkyv::Deserialize<T, Strategy<Pool, RkyvError>>,
    {
        use TranscodeFormat::*;
        use TranscodeFormatDecodeError::*;
        match self {
            Rkyv => {
                let value = handle!(rkyv::from_bytes::<T, RkyvError>(&input), FromBytesFailed, input);
                Ok(value)
            }
            SerdeJson => {
                let value = handle!(serde_json::from_slice::<T>(&input), FromSliceFailed, input);
                Ok(value)
            }
        }
    }

    pub fn encode<T>(self, value: T) -> Result<Vec<u8>, TranscodeFormatEncodeError<T>>
    where
        T: Serialize + for<'a> rkyv::Serialize<HighSerializer<AlignedVec, ArenaHandle<'a>, RkyvError>>,
    {
        use TranscodeFormat::*;
        use TranscodeFormatEncodeError::*;
        match self {
            Rkyv => {
                let bytes = handle!(
                    rkyv::to_bytes::<RkyvError>(&value),
                    ToBytesFailed,
                    value: Box::new(value)
                );
                Ok(bytes.into_vec())
            }
            SerdeJson => {
                let bytes = handle!(
                    serde_json::to_vec(&value),
                    ToVecFailed,
                    value: Box::new(value)
                );
                Ok(bytes)
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum TranscodeFormatDecodeError {
    #[error("failed to deserialize rkyv payload")]
    FromBytesFailed { source: rkyv::rancor::Error, input: Vec<u8> },
    #[error("failed to deserialize serde_json payload")]
    FromSliceFailed { source: serde_json::Error, input: Vec<u8> },
}

#[derive(Error, Debug)]
pub enum TranscodeFormatEncodeError<T> {
    #[error("failed to serialize payload to rkyv")]
    ToBytesFailed { source: rkyv::rancor::Error, value: Box<T> },
    #[error("failed to serialize payload to serde_json")]
    ToVecFailed { source: serde_json::Error, value: Box<T> },
}
