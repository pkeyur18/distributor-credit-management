// Testing-only tool. GUI wrapper around test_data_shared's reset/import
// logic for a client with no dev toolchain — a double-click .exe, no
// terminal, no cargo/node required on their machine.
// See docs/superpowers/specs/2026-08-19-client-test-tool-design.md.
//
// Not registered anywhere (no `lib.rs`/`commands.rs` change beyond
// `test_data_shared`, shared with `import_test_data.rs`). Delete this
// file (and test_data_shared.rs, if import_test_data.rs is also gone)
// to remove the tool.
use std::path::PathBuf;

use bvconsole_lib::test_data_shared;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([420.0, 320.0]),
        ..Default::default()
    };
    eframe::run_native(
        "BV Console — Test Tool",
        options,
        Box::new(|_cc| Ok(Box::new(TestToolApp::default()))),
    )
}

#[derive(Default)]
struct TestToolApp {
    status: String,
    credential: String,
    closed_months_input: String,
    pending_csv: Option<PathBuf>,
}

impl eframe::App for TestToolApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("BV Console — Test Tool");
        ui.label("Close the main app before using this tool.");
        ui.separator();

        if ui.button("Reset Test Data").clicked() {
            let confirmed = rfd::MessageDialog::new()
                .set_title("Reset Test Data")
                .set_description(
                    "This deletes console.db and backups (keeps your PIN/password). \
                     Close the main app first. Continue?",
                )
                .set_buttons(rfd::MessageButtons::YesNo)
                .show();
            if confirmed == rfd::MessageDialogResult::Yes {
                let app_data_dir = test_data_shared::default_app_data_dir();
                self.status = match test_data_shared::reset_data(&app_data_dir) {
                    Ok(deleted) if deleted.is_empty() => {
                        "nothing to reset — no app data found".to_string()
                    }
                    Ok(deleted) => format!(
                        "reset done — {} item(s) removed. Log in with your PIN/password to start fresh.",
                        deleted.len()
                    ),
                    Err(e) => format!("reset failed: {e}"),
                };
            }
        }

        ui.separator();

        if ui.button("Import CSV...").clicked() {
            self.status = "import not wired yet".to_string();
        }

        ui.separator();
        ui.label(&self.status);
    }
}
