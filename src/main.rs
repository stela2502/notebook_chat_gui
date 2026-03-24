mod ai;
mod app;
mod logging;
mod state;

use app::NotebookChatApp;
use eframe::NativeOptions;
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let mut args = std::env::args().skip(1);

    let startup_notebook = args.next().map(PathBuf::from);

    let options = NativeOptions::default();

    eframe::run_native(
        "Notebook Chat GUI",
        options,
        Box::new(move |_cc| Ok(Box::new(NotebookChatApp::new(startup_notebook.clone())))),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_builds() {
        assert_eq!(2 + 2, 4);
    }
}
