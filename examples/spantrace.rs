use thiserror::Error;
use tracing_error::SpanTrace;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_error::ErrorLayer::default())
        .init();

    let result = boom();
    match result {
        Err(err) => {
            eprintln!("error: {}", err);
            match err {
                Error::MyError(source, span_trace) => {
                    eprintln!("source: {}", source);
                    eprintln!("span trace: {:#?}", span_trace);
                }
            }
        }
        _ => unreachable!(),
    }
}

#[tracing::instrument]
fn boom() -> Result<(), Error> {
    inner_boom()?;
    Ok(())
}

#[tracing::instrument]
fn inner_boom() -> Result<(), Error> {
    non_span_trace()?;
    Ok(())
}

#[tracing::instrument]
fn non_span_trace() -> std::io::Result<()> {
    std::fs::read_to_string("nonexistent-file")?;
    Ok(())
}

#[derive(Error, Debug)]
enum Error {
    #[error("I/O error: {0}")]
    MyError(#[from] std::io::Error, SpanTrace),
}
