use crate::PropertyDistribution;
use derive_more::{From, Into};
use derive_new::new;
use serde::Serialize;

#[derive(new, From, Into, Serialize, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Clone, Debug)]
pub struct PropertyDistributionSimpleView<'a> {
    pub name: &'a str,
    pub successes: usize,
    pub failures: usize,
    pub abs_diff: usize,
}

impl<'a, const LIMIT: usize, T> From<&'a PropertyDistribution<LIMIT, T>> for PropertyDistributionSimpleView<'a> {
    fn from(distribution: &'a PropertyDistribution<LIMIT, T>) -> Self {
        let PropertyDistribution {
            name,
            success,
            failure,
        } = distribution;
        let name = name.as_str();
        let successes = success.count;
        let failures = failure.count;
        let abs_diff = successes.abs_diff(failures);
        Self {
            name,
            successes,
            failures,
            abs_diff,
        }
    }
}
