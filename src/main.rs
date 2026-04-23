use clap::Parser;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let app = bms_resource_toolbox::cli::App::parse();
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    rt.block_on(async {
        if let Err(e) = bms_resource_toolbox::commands::dispatch(app).await {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    });
}
