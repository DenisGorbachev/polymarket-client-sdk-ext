use subtype::subtype;

subtype!(
    #[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Default, Debug)]
    #[derive(derive_more::Display)]
    #[derive(derive_more::Add, derive_more::Sub)]
    #[derive(derive_more::AddAssign, derive_more::SubAssign)]
    #[derive(serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct DurationPositiveSeconds(u64);
);

impl DurationPositiveSeconds {}
