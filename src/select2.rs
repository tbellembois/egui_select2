use eframe::egui;
use egui::{Response, Ui};
use serde::{Deserialize, Serialize};
use std::iter::FromIterator;

// Widget translations.
#[derive(Default, Clone)]
pub struct Translations {
    pub loading: String,
    pub no_results: String,
    pub add: String,
    pub clear_all: String,
}

/// A select item.
/// The `id` is used to identify the item, and the `label` is the text to display.
/// The `id` is None for new items only (when `read_only` is false).
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SelectItem {
    pub id: Option<u64>,
    pub label: String,
}

/// A collection of select items retrieved by the `load_suggestions` function.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SelectItems {
    pub items: Vec<SelectItem>,
    pub total: usize,
}

impl FromIterator<SelectItem> for SelectItems {
    fn from_iter<I: IntoIterator<Item = SelectItem>>(iter: I) -> Self {
        let mut items = Vec::new();
        for item in iter {
            items.push(item);
        }
        SelectItems {
            items: items.clone(),
            total: items.len(),
        }
    }
}

const DEFAULT_SCROLL_MAX_HEIGHT: f32 = 150.0;
const DEFAULT_MINIMUM_INPUT_LENGTH: usize = 1;
const DEFAULT_MAXIMUM_SUGGESTIONS_NUMBER: usize = 10;

/// A select2 like widget.
/// Typical usage in egui:
/// ```
/// // Define your select in your app state.
/// struct MyApp {
///     my_select: EguiSelect2,
/// ...
/// }
///
/// // Define your load_suggestions function.
/// fn my_load_suggestions(limit: usize, offset: usize, query: &str) -> SelectItems {
/// ...
/// SelectItems { items, total }
/// }
///
/// // Initialize your select with default values.
/// // And attach your load_suggestions function.
/// impl Default for MyApp {
///     fn default() -> Self {
///         let mut my_select = EguiSelect2::default();
///         // required
///         my_select.load_suggestions = Box::new(my_load_suggestions);
///
///         Self { my_select }
///     }
/// }
///
/// // Use the select in your UI.
/// // Don't forget to call `check_loading` before `ui`.
/// impl eframe::App for MyApp {
///     fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
///         // required
///         self.my_select.check_loading();
///         self.my_select.ui(ui);
///     }
/// }
/// ```
pub struct EguiSelect2 {
    /// The function to load suggestions.
    pub load_suggestions: Box<dyn Fn(usize, usize, &str) -> Result<SelectItems, String>>,
    /// The function to format a suggestion in the dropdown.
    pub format_suggestion: Box<dyn Fn(&mut Ui, bool, &SelectItem) -> Response>,
    /// The translations for the widget.
    pub translations: Translations,
    /// The maximum number of suggestions to load at once.
    pub maximum_suggestions_number: usize,
    /// The offset of the suggestions to load. Automatically managed by the widget.
    pub offset: usize,
    /// Whether to close the widget when a suggestion is selected.
    pub close_on_select: bool,
    /// Whether the widget is disabled.
    pub disabled: bool,
    /// Whether the widget allows multiple selections.
    pub multiple: bool,
    /// The minimum number of characters required to trigger a suggestion load.
    pub minimum_input_length: usize,
    /// The input text.
    pub input: String,
    /// The selected items.
    pub selected: Vec<SelectItem>,
    /// The suggestions to display.
    pub suggestions: SelectItems,
    /// The scroll max height.
    pub scroll_max_height: f32,
    /// Whether the widget has more suggestions to load.
    pub has_more: bool,
    /// Whether the widget is loading suggestions.
    pub loading: bool,
    /// Whether the widget is read-only. Setting this to `false` allows the user to enter new items.
    pub read_only: bool,
    /// The last time the input was edited. Automatically managed by the widget to debounce input events.
    last_edit_time: f64,
    /// The last input text that triggered a suggestion load. Automatically managed by the widget to debounce input events.
    autocomplete_triggered_for: String,
    /// The index of the highlighted suggestion.
    highlighted: Option<usize>,
    /// Whether the widget is open.
    open: bool,
}

impl Default for EguiSelect2 {
    /// Creates a new `EguiSelect2` with default values.
    /// You need to provide at least a `load_suggestions` function to load suggestions.
    fn default() -> Self {
        Self {
            // Customizable parameters.
            load_suggestions: Box::new(|_, _, _| Ok(SelectItems::default())),
            format_suggestion: Box::new(|ui, selected, select_item| {
                ui.add(egui::Button::new(&select_item.label).selected(selected))
            }),
            minimum_input_length: DEFAULT_MINIMUM_INPUT_LENGTH,
            maximum_suggestions_number: DEFAULT_MAXIMUM_SUGGESTIONS_NUMBER,
            scroll_max_height: DEFAULT_SCROLL_MAX_HEIGHT,
            multiple: false,
            read_only: true,
            close_on_select: true,
            disabled: false,
            translations: Translations {
                loading: "Loading...".to_string(),
                no_results: "No results".to_string(),
                add: "Add".to_string(),
                clear_all: "Clear all".to_string(),
            },

            // Internal attributes.
            offset: 0,
            input: String::default(),
            selected: Vec::new(),
            suggestions: SelectItems::default(),
            has_more: true,
            loading: false,
            highlighted: None,
            open: false,
            last_edit_time: 0.0,
            autocomplete_triggered_for: String::default(),
        }
    }
}

impl EguiSelect2 {
    /// Creates a new `EguiSelect2` with the given parameters.
    pub fn new(
        load_suggestions: impl Fn(usize, usize, &str) -> Result<SelectItems, String> + 'static,
        format_suggestion: impl Fn(&mut Ui, bool, &SelectItem) -> Response + 'static,
        close_on_select: bool,
        disabled: bool,
        maximum_suggestions_number: usize,
        minimum_input_length: usize,
        multiple: bool,
        read_only: bool,
        translations: Translations,
    ) -> Self {
        EguiSelect2 {
            load_suggestions: Box::new(load_suggestions),
            format_suggestion: Box::new(format_suggestion),
            maximum_suggestions_number,
            read_only,
            minimum_input_length,
            close_on_select,
            disabled,
            multiple,
            translations,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn execute_load(
        &self,
        limit: usize,
        offset: usize,
        query: &str,
    ) -> Result<SelectItems, String> {
        (self.load_suggestions)(limit, offset, query)
    }

    /// Checks if the widget is loading suggestions and loads them if necessary.
    /// This method must be called outside the `ui` method of the widget.
    /// See examples for usage.
    pub fn check_loading(&mut self) {
        if self.loading {
            // Trigger the load.
            let suggestions = match self.execute_load(
                self.maximum_suggestions_number,
                self.offset,
                &self.input,
            ) {
                Ok(suggestions) => suggestions,
                Err(e) => {
                    log::error!("failed to load suggestions: {e}");
                    SelectItems::default()
                }
            };

            // Append or replace the suggestions given the offset.
            if self.offset == 0 {
                self.suggestions = suggestions.clone();
            } else {
                self.suggestions.items.extend(suggestions.items);
            }

            // Increase the offset for the next query and check if there are more suggestions to load.
            self.offset = self.suggestions.items.len();
            self.has_more = self.offset < suggestions.total;

            // Loading complete.
            self.loading = false;
        }
    }

    // Render the selected items as clickable labels.
    fn render_selected_items(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            // Index of the item to remove.
            let mut remove_idx = None;

            for (i, item) in self.selected.iter().enumerate() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(&item.label);
                        // Add a "✕" button to remove the item on when the widget is not disabled.
                        if !self.disabled && ui.button("✕").clicked() {
                            remove_idx = Some(i);
                        }
                    });
                });
            }

            // Remove the selected item if the user clicks the "✕" button.
            if let Some(i) = remove_idx {
                self.selected.remove(i);
            }
        });
    }

    // Render the input text field.
    fn render_input(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let input_widget = egui::TextEdit::singleline(&mut self.input);

        // Manage input widget state based on disabled state.
        let input_resp = if self.disabled {
            ui.add_enabled(false, input_widget)
        } else {
            ui.add(input_widget)
        };

        // Set last edit time on input change.
        if input_resp.changed() {
            self.last_edit_time = ui.input(|i| i.time);
        }

        // Debounce delay (seconds).
        let delay = 0.3;

        // Check if enough time passed since last edit.
        let now = ui.input(|i| i.time);
        let debounce_delay_passed =
            (now - self.last_edit_time) > delay && self.input != self.autocomplete_triggered_for;

        // Trigger autocomplete after delay.
        if debounce_delay_passed && !self.loading && self.minimum_input_length <= self.input.len() {
            self.autocomplete_triggered_for.clone_from(&self.input);
            self.offset = 0;
            // self.has_more = true;
            self.loading = true;
            self.open = false;
        }

        // Open suggestions on focus.
        if input_resp.has_focus() && !self.loading {
            self.open = true;
        }

        input_resp
    }

    // Render keyboard actions for the widget.
    fn render_keyboard_actions(&mut self, ui: &mut egui::Ui) {
        if self.open {
            ui.input(|i| {
                if i.key_pressed(egui::Key::ArrowDown) {
                    self.move_down();
                }
                if i.key_pressed(egui::Key::ArrowUp) {
                    self.move_up();
                }
                if i.key_pressed(egui::Key::Enter) {
                    self.select_highlighted();
                }
                if i.key_pressed(egui::Key::Escape) {
                    self.open = false;
                }
                if i.key_pressed(egui::Key::Backspace) && self.input.is_empty() {
                    self.selected.pop();
                }
            });
        }
    }

    // Render the dropdown of suggestions.
    fn render_dropdown(&mut self, ui: &mut egui::Ui) {
        if self.loading {
            ui.label(&self.translations.loading);
        }

        if self.open {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                if self.suggestions.items.is_empty() {
                    if self.input.is_empty() || self.read_only {
                        ui.label(&self.translations.no_results);
                    } else if !self.read_only
                        && ui
                            .button(format!("{} \"{}\"", self.translations.add, self.input))
                            .clicked()
                    {
                        self.add_new();
                    }
                } else {
                    let mut clicked_index = None;

                    egui::ScrollArea::vertical()
                        .id_salt(ui.id().with("scroll"))
                        .max_height(self.scroll_max_height)
                        .show(ui, |ui| {
                            for (i, item) in self.suggestions.items.iter().enumerate() {
                                let selected = self.highlighted == Some(i);

                                let resp = (self.format_suggestion)(ui, selected, item);

                                if resp.clicked() {
                                    clicked_index = Some(i);
                                }

                                if selected
                                    && ui.input(|i| {
                                        i.key_pressed(egui::Key::ArrowDown)
                                            || i.key_pressed(egui::Key::ArrowUp)
                                    })
                                {
                                    resp.scroll_to_me(Some(egui::Align::Center));
                                }
                            }

                            // Request the next page of suggestions.
                            if self.has_more && !self.loading && ui.small_button("+").clicked() {
                                self.loading = true;
                            }
                        });

                    if let Some(i) = clicked_index {
                        self.select_index(i);
                    }
                }
            });
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let response = ui.vertical(|ui| {
            self.render_selected_items(ui);
            self.render_input(ui);
            self.render_keyboard_actions(ui);
            self.render_dropdown(ui);
        });

        response.response
    }

    // Move the highlighted suggestion down.
    fn move_down(&mut self) {
        if let Some(i) = self.highlighted {
            // An item is highlighted// An item is highlighted, so move the highlight down.
            // Increment the highlighted index if it's not at the end of the suggestions.
            if i + 1 < self.suggestions.items.len() {
                self.highlighted = Some(i + 1);
            }
        } else if !self.suggestions.items.is_empty() {
            // No item is highlighted, so move to the first suggestion.
            self.highlighted = Some(0);
        }
    }

    // Move the highlighted suggestion up.
    fn move_up(&mut self) {
        // Decrement the highlighted index if it's not at the beginning of the suggestions.
        if let Some(i) = self.highlighted
            && i > 0
        {
            self.highlighted = Some(i - 1);
        }
    }

    // Select the highlighted suggestion.
    fn select_highlighted(&mut self) {
        if let Some(i) = self.highlighted {
            // An item is highlighted, select it.
            self.select_index(i);
        } else if !self.input.is_empty() {
            self.add_new();
        }
    }

    // Add select_item at index i to the selected items if not already selected.
    fn select_index(&mut self, i: usize) {
        if let Some(select_item) = self.suggestions.items.get(i).cloned()
            && !self
                .selected
                .iter()
                .any(|s| s.id == select_item.id && s.label == select_item.label)
        {
            if self.multiple {
                self.selected.push(select_item);
            } else {
                self.selected.clear();
                self.selected.push(select_item);
            }
        }

        // Clear the input and close the suggestions if close_on_select is enabled.
        self.input.clear();
        if self.close_on_select {
            self.open = false;
        }
    }

    // Add a new custom item to the selected items.
    fn add_new(&mut self) {
        if !self.input.is_empty() {
            let new_item = SelectItem {
                id: None,
                label: self.input.clone(),
            };

            if !self.selected.iter().any(|x| x.label == new_item.label) {
                self.selected.push(new_item);
            }

            self.input.clear();
            self.open = false;
        }
    }
}
