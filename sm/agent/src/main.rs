use crate::component::TEdgeComponent;

mod component;
mod error;
mod sm_agent;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    initialise_logging();

    let component = sm_agent::SmAgent::new("abc");
    component.start().await
    // Ok(())
}

fn initialise_logging() {
    tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::ChronoUtc::with_format(
            "%Y-%m-%dT%H:%M:%S%.3f%:z".into(),
        ))
        .init();
}
