use std::{sync::LazyLock, time::Duration};

use surrealdb::{
    Surreal,
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
};
use tracing::{error, info};
use tracing_unwrap::{OptionExt, ResultExt};
use url::Url;

use crate::config::get_shared_config;

pub mod fetch_profiles;
pub mod types;

pub static DB: LazyLock<Surreal<Client>> = LazyLock::new(Surreal::init);

pub async fn init() -> surrealdb::Result<()> {
    info!("Connecting to surrealdb");
    connect_using_url(&DB, &get_shared_config().await.surreal_db_url).await;
    let _ = DB
        .query(include_str!("sql/init_fetch_profiles_function.surrealql"))
        .await;

    Ok(())
}

async fn connect_using_url(db: &Surreal<Client>, url: &Url) {
    while let Err(_) = db.health().await {
        match db
            .connect::<Ws>(
                format!(
                    "{}:{}",
                    url.host_str().unwrap_or_log(),
                    url.port().unwrap_or_log()
                ), // "localhost:4102",
            )
            .await
        {
            Ok(_) => info!("Connected to surrealdb"),
            Err(err) => {
                error!("Failed to connect to surrealdb: {:?}", err);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }

    db.signin(Root {
        username: url.username().into(),
        password: url.password().unwrap_or_log().into(),
    })
    .await
    .unwrap_or_log();
    let path = url.path_segments().unwrap_or_log().collect::<Vec<&str>>();
    db.use_ns(path[0]).use_db(path[1]).await.unwrap_or_log();
}
