#[allow(unused_imports)]
use Flank::*;
use strum::Display;

#[derive(Display, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum Flank {
    Left,
    Right,
}

impl Flank {}
