use std::env;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const REFRESH_TEST_CACHE_ENV: &str = "REFRESH_TEST_CACHE";

pub fn parse_boolish_env_var(var: &str) -> Result<Option<bool>, ParseBoolishEnvVarError> {
    use ParseBoolishEnvVarError::*;
    let value = match env::var(var) {
        Ok(value) => value,
        Err(env::VarError::NotPresent) => return Ok(None),
        Err(source) => {
            return Err(ReadEnvVarFailed {
                source,
                var: var.to_string(),
            });
        }
    };
    let parsed = match parse_boolish(&value) {
        Ok(value) => value,
        Err(source) => {
            return Err(InvalidBoolishValue {
                source,
                var: var.to_string(),
                value,
            });
        }
    };
    Ok(Some(parsed))
}

pub fn parse_boolish(value: &str) -> Result<bool, ParseBoolishError> {
    use ParseBoolishError::*;
    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "1" | "true" | "t" | "yes" | "y" | "on" => Ok(true),
        "0" | "false" | "f" | "no" | "n" | "off" => Ok(false),
        _ => Err(InvalidValue {
            value: value.to_string(),
        }),
    }
}

pub fn progress_report_line(action: &str, count: u64, total: Option<u64>) -> String {
    let counter = match total {
        None => format!("{count} so far"),
        Some(total) => format!("{count} / {total}"),
    };
    format!("{action}: {counter}")
}

pub fn to_tmp_path(cache_path: &Path) -> PathBuf {
    cache_path.with_extension("tmp")
}

#[derive(Error, Debug)]
pub enum ParseBoolishEnvVarError {
    #[error("failed to read env var '{var}'")]
    ReadEnvVarFailed { source: env::VarError, var: String },
    #[error("invalid boolish value '{value}' for env var '{var}'")]
    InvalidBoolishValue { source: ParseBoolishError, var: String, value: String },
}

#[derive(Error, Debug)]
pub enum ParseBoolishError {
    #[error("invalid boolish value '{value}'")]
    InvalidValue { value: String },
}
