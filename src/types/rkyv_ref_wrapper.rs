use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use rkyv::rancor::Fallible;
use rkyv::with::{ArchiveWith, SerializeWith};
use rkyv::{Archive, Place, Serialize};

#[derive(Debug)]
pub struct RkyvRefWrapper<'a, A, O>(pub &'a O, pub PhantomData<A>);

impl<A: ArchiveWith<O>, O> Archive for RkyvRefWrapper<'_, A, O> {
    type Archived = <A as ArchiveWith<O>>::Archived;
    type Resolver = <A as ArchiveWith<O>>::Resolver;

    fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
        A::resolve_with(self.0, resolver, out)
    }
}

impl<A, O, S> Serialize<S> for RkyvRefWrapper<'_, A, O>
where
    A: ArchiveWith<O> + SerializeWith<O, S>,
    S: Fallible + ?Sized,
{
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        A::serialize_with(self.0, serializer)
    }
}

impl<A, O: Hash> Hash for RkyvRefWrapper<'_, A, O> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<A, O: PartialEq> PartialEq for RkyvRefWrapper<'_, A, O> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<A, O: Eq> Eq for RkyvRefWrapper<'_, A, O> {}
