use surrealdb::types::SurrealValue;
use surrealdb_types::RecordId;

#[derive(Debug, Clone, SurrealValue)]
pub struct MindustryUser {
    id: RecordId,
    profile: RecordId,
}

#[derive(Debug, Clone, SurrealValue)]
pub struct MindustryProfile {}

#[derive(Clone, Debug)]
pub enum ProfileType {
    Mindustry,
    Discord,
    Telegram,
}

impl ProfileType {
    pub const fn value(&self) -> &'static str {
        match self {
            Self::Mindustry => "mindustry",
            Self::Discord => "discord",
            Self::Telegram => "telegram",
        }
    }
}

#[derive(Clone, Debug, Default, SurrealValue)]
pub struct NeededProfiles {
    pub mindustry: bool,
    pub discord: bool,
    pub telegram: bool,
}

#[derive(Clone, Debug, SurrealValue)]
pub struct Profiles {
    pub mindustry: Option<MindustryProfile>,
}
