use crate::{Amount, Price, RkyvDecimal, RkyvRefWrapper};
use core::marker::PhantomData;
use indexmap::IndexMap;
use rkyv::Place;
use rkyv::collections::swiss_table::{ArchivedIndexMap, IndexMapResolver};
use rkyv::rancor::{Fallible, Source};
use rkyv::ser::{Allocator, Writer};
use rkyv::with::{ArchiveWith, DeserializeWith, SerializeWith};
use rustc_hash::FxBuildHasher;

#[derive(Debug)]
pub struct RkyvIndexMapDecimal;

impl ArchiveWith<IndexMap<Price, Amount, FxBuildHasher>> for RkyvIndexMapDecimal {
    type Archived = ArchivedIndexMap<<RkyvDecimal as ArchiveWith<Price>>::Archived, <RkyvDecimal as ArchiveWith<Amount>>::Archived>;
    type Resolver = IndexMapResolver;

    fn resolve_with(field: &IndexMap<Price, Amount, FxBuildHasher>, resolver: Self::Resolver, out: Place<Self::Archived>) {
        ArchivedIndexMap::resolve_from_len(field.len(), (7, 8), resolver, out)
    }
}

impl<S> SerializeWith<IndexMap<Price, Amount, FxBuildHasher>, S> for RkyvIndexMapDecimal
where
    S: Fallible + Allocator + Writer + ?Sized,
    S::Error: Source,
{
    fn serialize_with(field: &IndexMap<Price, Amount, FxBuildHasher>, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        let iter = field
            .iter()
            .map(|(price, amount)| (RkyvRefWrapper(price, PhantomData::<RkyvDecimal>), RkyvRefWrapper(amount, PhantomData::<RkyvDecimal>)));
        let result = ArchivedIndexMap::<<RkyvDecimal as ArchiveWith<Price>>::Archived, <RkyvDecimal as ArchiveWith<Amount>>::Archived>::serialize_from_iter(iter, (7, 8), serializer);
        match result {
            Ok(resolver) => Ok(resolver),
            Err(error) => Err(error),
        }
    }
}

impl<D> DeserializeWith<ArchivedIndexMap<<RkyvDecimal as ArchiveWith<Price>>::Archived, <RkyvDecimal as ArchiveWith<Amount>>::Archived>, IndexMap<Price, Amount, FxBuildHasher>, D> for RkyvIndexMapDecimal
where
    D: Fallible + ?Sized,
{
    fn deserialize_with(field: &ArchivedIndexMap<<RkyvDecimal as ArchiveWith<Price>>::Archived, <RkyvDecimal as ArchiveWith<Amount>>::Archived>, deserializer: &mut D) -> Result<IndexMap<Price, Amount, FxBuildHasher>, D::Error> {
        field
            .iter()
            .try_fold(IndexMap::with_capacity_and_hasher(field.len(), FxBuildHasher), |mut map, (price, amount)| {
                let price = match RkyvDecimal::deserialize_with(price, deserializer) {
                    Ok(value) => value,
                    Err(error) => return Err(error),
                };
                let amount = match RkyvDecimal::deserialize_with(amount, deserializer) {
                    Ok(value) => value,
                    Err(error) => return Err(error),
                };
                map.insert(price, amount);
                Ok(map)
            })
    }
}
