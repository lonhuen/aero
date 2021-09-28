use opentelemetry::sdk::export::trace::stdout;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::{fmt::format::FmtSpan, prelude::*};

pub fn init_tracing(
    service_name: &str,
    agent_endpoint: &str,
    filter: LevelFilter,
) -> anyhow::Result<()> {
    //let tracer = opentelemetry_jaeger::new_pipeline()
    //    .with_agent_endpoint(agent_endpoint)
    //    .with_service_name(service_name)
    //    .install_batch(opentelemetry::runtime::Tokio)?;
    let tracer = stdout::new_pipeline().install_simple();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env().add_directive(filter.into()))
        .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::NEW | FmtSpan::CLOSE))
        //.with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::NONE))
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .try_init()?;

    Ok(())
}
