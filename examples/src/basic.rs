use std::thread::sleep;

use egui_select2::select2::{EguiSelect2, SelectItem, SelectItems};

struct MyApp {
    my_select: EguiSelect2,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut my_select = EguiSelect2::default();
        my_select.read_only = false;
        my_select.min_input_length = 1;
        my_select.limit = 15;
        my_select.load_suggestions = Box::new(my_load_suggestions);

        Self { my_select }
    }
}

fn my_load_suggestions(limit: usize, offset: usize, query: &str) -> SelectItems {
    sleep(std::time::Duration::from_secs(1));

    let database: Vec<(String, String)> = (0..500)
        .map(|i| (i.to_string(), format!("This is item {}", i)))
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

    SelectItems { items, total }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            self.my_select.check_loading();
            self.my_select.ui(ui);

            ui.separator();
            ui.label(format!(
                "Loaded: {}",
                self.my_select.suggestions.items.len()
            ));
        });

        let ctx = ui.ctx();
        ctx.request_repaint();
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Select2-like MultiSelect",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}
