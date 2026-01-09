mod app;

use app::EmulatorApp;

fn main() -> Result<(), slint::PlatformError> {
    let app = EmulatorApp::new()?;
    app.run()
}
