use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(ValueEnum, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Debug, Default)]
#[clap(rename_all = "kebab")]
pub enum RelatedMarketsFormat {
    #[default]
    Json,
    Short,
}
