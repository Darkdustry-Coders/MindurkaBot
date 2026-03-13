use async_trait::async_trait;

#[async_trait]
pub trait Bot: Send + Sync {
    fn service_name(&self) -> &'static str {
        "Undefined"
    }

    async fn start(&self);
}
