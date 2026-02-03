use subtype::subtype_u64;

subtype_u64!(
    #[derive(serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct DurationPositiveSeconds(u64);
);

impl DurationPositiveSeconds {}
