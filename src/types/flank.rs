#[allow(unused_imports)]
use Flank::*;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Display, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum Flank {
    Left,
    Right,
}

impl Flank {}
