use std::sync::Arc;

use egui_select2::select2::{EguiSelect2, SelectItem, SelectItems, SharedSelect2Items};

struct MyApp {
    my_select: EguiSelect2,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut my_select = EguiSelect2::default();
        my_select.read_only = true;
        my_select.minimum_input_length = 1;
        my_select.maximum_suggestions_number = 15;
        my_select.load_suggestions = Arc::new(my_load_suggestions);

        Self { my_select }
    }
}

fn my_load_suggestions(
    suggestions: SharedSelect2Items,
    _limit: usize,
    _offset: usize,
    query: String,
) {
    let request = ehttp::Request::get(format!("https://swapi.dev/api/people/?search={}", query))
        .with_headers(ehttp::Headers::new(&[(
            "Content-Type",
            "application/json; charset=UTF-8;",
        )]));

    #[derive(Debug, serde::Deserialize)]
    struct SamplePerson {
        pub name: String,
        pub homeworld: String,
    }

    #[derive(Debug, serde::Deserialize)]
    struct SampleResponse {
        pub count: usize,
        pub results: Vec<SamplePerson>,
    }

    let response = ehttp::fetch_blocking(&request);

    let response = response.unwrap();
    let sample_response: SampleResponse = serde_json::from_str(response.text().unwrap()).unwrap();
    let items: Vec<SelectItem> = sample_response
        .results
        .into_iter()
        .map(|p| SelectItem {
            id: Some(1),
            label: format!("{} ({})", p.name, p.homeworld),
        })
        .collect();
    let total = sample_response.count;

    let mut locked_suggestions = suggestions.lock().unwrap();
    *locked_suggestions = Some(SelectItems { items, total });
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
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

    eframe::run_native(
        "Select2-like MultiSelect",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}
