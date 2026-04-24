use std::thread::sleep;

use egui_select2::select2::{EguiSelect2, SelectItem, SelectItems, SharedSelect2Items};

struct MyApp {
    my_select: EguiSelect2,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut my_select = EguiSelect2::default();

        my_select.read_only = false;
        my_select.multiple = true;
        my_select.minimum_input_length = 1;
        my_select.maximum_suggestions_number = 15;
        my_select.close_on_select = false;
        my_select.load_suggestions = Box::new(my_load_suggestions);

        Self { my_select }
    }
}

fn my_load_suggestions(suggestions: SharedSelect2Items, limit: usize, offset: usize, query: &str) {
    sleep(std::time::Duration::from_secs(1));

    let database: Vec<(u64, String)> = (0..500)
        .map(|i| (i, format!("This is item {}", i)))
        .collect();

    let filtered = database
        .into_iter()
        .filter(|(_, label)| label.to_lowercase().contains(query));

    let total = filtered.clone().count();

    let items: Vec<SelectItem> = filtered
        .skip(offset)
        .take(limit)
        .map(|(id, label)| SelectItem {
            id: Some(id),
            label,
        })
        .collect();

    let mut locked_suggestions = suggestions.lock().unwrap();
    *locked_suggestions = Some(SelectItems { items, total });
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Window::new("Log").show(ui, |ui| {
            egui_logger::logger_ui().show(ui);
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            self.my_select.check_loading();
            self.my_select.ui(ui);

            let locked_suggestions = self.my_select.suggestions.lock().unwrap();

            if let Some(suggestions) = locked_suggestions.as_ref() {
                ui.separator();
                ui.label(format!("Loaded: {}", suggestions.items.len()));
            } else {
                ui.separator();
                ui.label("Loaded: 0");
            }

            ui.separator();
            self.my_select.selected.iter().for_each(|item| {
                ui.label(format!("Selected: {:?} {}", item.id, item.label.clone()));
            });
        });

        let ctx = ui.ctx();
        ctx.request_repaint();
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();

    egui_logger::builder().init().unwrap();

    eframe::run_native(
        "Select2-like MultiSelect",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}
