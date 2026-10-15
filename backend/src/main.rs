#[tokio::main]
async fn main() -> anyhow::Result<()> {
    opd_billing_backend::run().await
}
