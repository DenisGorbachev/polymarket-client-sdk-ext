use crate::{PropertyName, property_name};
use fjall::Snapshot;

/// A trait for checking whether the property holds for a value
/// [`&Snapshot`](Snapshot) is passed to facilitate checking cross-keyspace properties (e.g. foreign key fields)
pub trait Property<T> {
    fn name(&self) -> PropertyName {
        property_name::<Self>()
    }

    fn holds(&mut self, value: &T, snapshot: &Snapshot) -> bool;
}
