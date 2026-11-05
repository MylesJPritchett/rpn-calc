mod tui;

use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = tui::App::new().run(terminal);
    ratatui::restore();
    app_result
}
