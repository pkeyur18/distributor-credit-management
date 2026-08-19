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

        if ui.button("Choose CSV...").clicked() {
            if let Some(path) = rfd::FileDialog::new().add_filter("CSV", &["csv"]).pick_file() {
                self.pending_csv = Some(path);
            }
        }
        ui.label(match &self.pending_csv {
            Some(p) => format!("Selected: {}", p.display()),
            None => "No CSV selected".to_string(),
        });

        ui.label("PIN/password:");
        ui.add(egui::TextEdit::singleline(&mut self.credential).password(true));

        ui.label("Closed months (comma-separated YYYY-MM, optional):");
        ui.text_edit_singleline(&mut self.closed_months_input);

        if ui.button("Run Import").clicked() {
            self.status = run_import(&self.credential, &self.pending_csv, &self.closed_months_input);
        }

        ui.separator();
        ui.label(&self.status);
    }
}

fn run_import(credential: &str, csv_path: &Option<PathBuf>, closed_months_input: &str) -> String {
    let Some(csv_path) = csv_path else {
        return "pick a CSV file first".to_string();
    };
    let content = match std::fs::read_to_string(csv_path) {
        Ok(c) => c,
        Err(e) => return format!("reading {}: {e}", csv_path.display()),
    };
    let closed_months: std::collections::HashSet<String> = closed_months_input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let app_data_dir = test_data_shared::default_app_data_dir();
    let conn = match test_data_shared::unlock_db(&app_data_dir, credential) {
        Ok(c) => c,
        Err(e) => return format!("unlock failed: {e}"),
    };
    match test_data_shared::import_csv(&conn, &content, &closed_months) {
        Ok(summary) => format!(
            "imported {} member(s), {} closed-month entries across {} month(s), {} open-period entries",
            summary.members, summary.closed_entries, summary.closed_months, summary.open_entries
        ),
        Err(e) => format!("import failed: {e}"),
    }
}
