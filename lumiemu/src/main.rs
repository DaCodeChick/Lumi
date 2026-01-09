mod app;

use app::EmulatorApp;

fn main() -> Result<(), slint::PlatformError> {
    // Initialize tracing subscriber for logging
    // Set RUST_LOG environment variable to control log level
    // Examples: RUST_LOG=trace, RUST_LOG=debug, RUST_LOG=info
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    let app = EmulatorApp::new()?;
    app.run()
}
