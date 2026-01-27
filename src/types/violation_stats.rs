use derive_more::{From, Into};
use derive_new::new;
use serde::{Deserialize, Serialize};

#[derive(new, From, Into, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Clone, Debug)]
pub struct ViolationStats<const LIMIT: usize, T> {
    pub count: usize,
    pub examples: Vec<T>,
}

impl<const LIMIT: usize, T> ViolationStats<LIMIT, T> {
    pub fn witness(&mut self, example: T) {
        self.count = self.count.saturating_add(1);
        if self.examples.len() < LIMIT {
            self.examples.push(example)
        }
    }
}
