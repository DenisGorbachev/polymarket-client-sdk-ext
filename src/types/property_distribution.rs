use crate::PropertyStats;
use derive_more::{From, Into};
use derive_new::new;

#[derive(new, From, Into, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Clone, Debug)]
pub struct PropertyDistribution<const LIMIT: usize, T> {
    pub name: String,
    pub success: PropertyStats<LIMIT, T>,
    pub failure: PropertyStats<LIMIT, T>,
}

impl<const LIMIT: usize, T> PropertyDistribution<LIMIT, T> {
    pub fn as_simple(&self) -> (&str, usize, usize, usize) {
        let successes = self.success.count;
        let failures = self.failure.count;
        (self.name.as_str(), successes, failures, successes.abs_diff(failures))
    }
}
mod property_distribution_simple_view;

pub use property_distribution_simple_view::*;

mod property_distribution_view;

pub use property_distribution_view::*;
