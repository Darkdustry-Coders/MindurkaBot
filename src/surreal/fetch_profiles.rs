use crate::surreal::{
    DB,
    types::{NeededProfiles, ProfileType, Profiles},
};

pub async fn fetch_profiles(
    profile_type: ProfileType,
    value: String,
    needed_profiles: NeededProfiles,
) -> Result<Option<Profiles>, surrealdb::Error> {
    let mut results = DB
        .query(include_str!("sql/fetch_profiles.surrealql"))
        .bind(("type", profile_type.value()))
        .bind(("value", value))
        .bind(("needed_profiles", needed_profiles))
        .await?;

    Ok(results.take(0)?)
}
