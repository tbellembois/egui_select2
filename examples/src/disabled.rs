use egui_select2::select2::{EguiSelect2, SelectItem};

struct MyApp {
    my_select: EguiSelect2,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut my_select = EguiSelect2::default();
        my_select.disabled = true;
        my_select.selected = vec![
            SelectItem {
                id: Some(1),
                label: "one".to_string(),
            },
            SelectItem {
                id: Some(2),
                label: "two".to_string(),
            },
            SelectItem {
                id: Some(3),
                label: "three".to_string(),
            },
        ];
        Self { my_select }
    }
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
            ui.separator();
            ui.label("Selected:");
            self.my_select.selected.iter().for_each(|item| {
                ui.label(format!("{:?} {}", item.id, item.label.clone()));
            });
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
