#[allow(unused_imports)]
use Flank::*;
use rkyv::Archive;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Archive, Display, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum Flank {
    Left,
    Right,
}

impl Flank {}
