use std::thread::sleep;

use egui::{Response, Ui, Vec2};
use egui_select2::select2::{EguiSelect2, SelectItem, SelectItems};

struct MyApp {
    my_select: EguiSelect2,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut my_select = EguiSelect2::default();
        my_select.read_only = true;
        my_select.minimum_input_length = 1;
        my_select.maximum_suggestions_number = 10;
        my_select.load_suggestions = Box::new(my_load_suggestions);
        my_select.format_suggestion = Box::new(my_format_suggestion);
        my_select.scroll_max_height = 400.0;

        Self { my_select }
    }
}

fn my_format_suggestion(ui: &mut Ui, selected: bool, select_item: &SelectItem) -> Response {
    let image_name = format!("{}.png", select_item.label);
    let image_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("examples/assets/{}", image_name));
    let image =
        egui::Image::new(format!("file://{}", image_path.to_string_lossy())).corner_radius(5.0);

    let image = image.fit_to_exact_size(Vec2::new(20.0, 20.0));

    ui.add(egui::Button::image_and_text(image, select_item.label.clone()).selected(selected))
}

fn my_load_suggestions(limit: usize, offset: usize, query: &str) -> Result<SelectItems, String> {
    sleep(std::time::Duration::from_secs(1));

    let database: Vec<(String, String)> = (1..9)
        .map(|i| (i.to_string(), format!("GHS0{}", i)))
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

    Ok(SelectItems { items, total })
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
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(MyApp::default()))
        }),
    )
}
