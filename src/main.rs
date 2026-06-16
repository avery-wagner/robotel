use opentelemetry::global;
use opentelemetry::trace::{Tracer, TracerProvider as _};
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_stdout::SpanExporter;

fn init_tracer() -> TracerProvider {
    let exporter = SpanExporter::default();
    let provider = TracerProvider::builder()
        .with_simple_exporter(exporter)
        .build();
    global::set_tracer_provider(provider.clone());
    provider
}

fn main() -> anyhow::Result<()> {
    let provider = init_tracer();
    let tracer = global::tracer("robotel");

    // Outer span: a fake "episode" of the robot doing things
    tracer.in_span("robot.episode", |_cx| {
        tracer.in_span("perception.scan", |_cx| {
            std::thread::sleep(std::time::Duration::from_millis(20));
        });

        tracer.in_span("planning.compute_path", |_cx| {
            std::thread::sleep(std::time::Duration::from_millis(40));
        });

        tracer.in_span("control.execute", |_cx| {
            std::thread::sleep(std::time::Duration::from_millis(10));
        });
    });

    // Make sure pending spans are flushed before exit
    provider.shutdown()?;
    Ok(())
}
