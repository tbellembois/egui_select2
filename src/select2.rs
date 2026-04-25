use eframe::egui;
use egui::Ui;
use serde::{Deserialize, Serialize};
use std::{
    iter::FromIterator,
    sync::{Arc, Mutex},
};

const DEFAULT_SCROLL_MAX_HEIGHT: f32 = 150.0;
const DEFAULT_MINIMUM_INPUT_LENGTH: usize = 1;
const DEFAULT_MAXIMUM_SUGGESTIONS_NUMBER: usize = 10;
const DEFAULT_LOADING_TEXT: &str = "Loading";
const DEFAULT_NO_RESULTS_TEXT: &str = "No results";
const DEFAULT_ADD_TEXT: &str = "Add";
const DEFAULT_CLEAR_ALL_TEXT: &str = "Clear all";
const DEFAULT_HINT_TEXT: &str = "Search";

pub type SharedSelect2Items = Arc<Mutex<Option<SelectItems>>>;
pub type LoadSuggestionsFn = Arc<dyn Fn(SharedSelect2Items, usize, usize, String) + Send + Sync>;
pub type FormatSuggestionFn =
    Box<dyn Fn(&mut Ui, bool, &SelectItem) -> egui::Response + Send + Sync>;

// Widget translations.
#[derive(Default, Clone)]
pub struct Translations {
    pub loading: String,
    pub no_results: String,
    pub add: String,
    pub clear_all: String,
    pub hint: String,
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

/// The behavior of the select2 widget.
pub struct WidgetBehavior {
    close_on_select: bool,
    disabled: bool,
    maximum_suggestions_number: usize,
    minimum_input_length: usize,
    multiple: bool,
    read_only: bool,
    translations: Translations,
}

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
/// fn my_load_suggestions(suggestions: SharedSelect2Items, limit: usize, offset: usize, query: &str) {
///     // Do not unwrap() in your code, handle the error instead.
///     let mut locked_suggestions = suggestions.lock().unwrap();
///
///     // Perform your request (local or remote) using `offset`, `limit`, and `query`.
///     // Turn the results into a `SelectItems` struct and assign it to `locked_suggestions`.
///     ...
///     *locked_suggestions = Some(SelectItems { items, total });
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
    pub load_suggestions: LoadSuggestionsFn,
    /// The function to format a suggestion in the dropdown.
    pub format_suggestion: FormatSuggestionFn,
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
    pub suggestions: SharedSelect2Items,
    /// The new suggestions to display.
    pub new_suggestions: SharedSelect2Items,
    /// The scroll max height.
    pub scroll_max_height: f32,
    /// Whether the widget has more suggestions to load.
    pub has_more: bool,
    /// Whether the widget is loading suggestions.
    pub loading: bool,
    /// A thread has been spawned to load suggestions.
    pub load: bool,
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
    /// The unique id of the widget.
    id: String,
}

impl Default for EguiSelect2 {
    /// Creates a new `EguiSelect2` with default values.
    /// You need to provide at least a `load_suggestions` function to load suggestions.
    fn default() -> Self {
        let rng: u64 = rand::random();
        let rng_string = rng.to_string();

        Self {
            // Customizable parameters.
            load_suggestions: Arc::new(|_, _, _, _| ()),
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
                loading: DEFAULT_LOADING_TEXT.to_string(),
                no_results: DEFAULT_NO_RESULTS_TEXT.to_string(),
                add: DEFAULT_ADD_TEXT.to_string(),
                clear_all: DEFAULT_CLEAR_ALL_TEXT.to_string(),
                hint: DEFAULT_HINT_TEXT.to_string(),
            },

            // Internal attributes.
            id: rng_string,
            offset: 0,
            input: String::default(),
            selected: Vec::new(),
            suggestions: SharedSelect2Items::default(),
            new_suggestions: SharedSelect2Items::default(),
            has_more: true,
            loading: false,
            load: false,
            highlighted: None,
            open: false,
            last_edit_time: 0.0,
            autocomplete_triggered_for: String::default(),
        }
    }
}

impl EguiSelect2 {
    /// Creates a new `EguiSelect2` with the given parameters.
    #[must_use]
    pub fn new(
        load_suggestions: LoadSuggestionsFn,
        format_suggestion: FormatSuggestionFn,
        widget_behavior: WidgetBehavior,
    ) -> Self {
        EguiSelect2 {
            load_suggestions,
            format_suggestion: Box::new(format_suggestion),
            maximum_suggestions_number: widget_behavior.maximum_suggestions_number,
            read_only: widget_behavior.read_only,
            minimum_input_length: widget_behavior.minimum_input_length,
            close_on_select: widget_behavior.close_on_select,
            disabled: widget_behavior.disabled,
            multiple: widget_behavior.multiple,
            translations: widget_behavior.translations,
            ..Default::default()
        }
    }

    /// Checks if the widget is loading suggestions and loads them if necessary.
    /// This method must be called outside the `ui` method of the widget.
    /// See examples for usage.
    pub fn check_loading(&mut self) {
        if self.loading && !self.load {
            let cloned_load_fn = Arc::clone(&self.load_suggestions);
            let cloned_new_suggestions = Arc::clone(&self.new_suggestions);
            let limit = self.maximum_suggestions_number;
            let offset = self.offset;
            let query = self.input.clone();

            log::debug!("spawning load_suggestions");

            // Trigger the load.
            crate::spawn::spawn(move || {
                (cloned_load_fn)(cloned_new_suggestions, limit, offset, query);
            });

            self.load = true;
        }

        if self.loading {
            // Acquire the locks on (new) suggestions.
            let mut locked_suggestions = match self.suggestions.lock() {
                Ok(locked_suggestions) => locked_suggestions,
                Err(e) => {
                    log::error!("locked_suggestions lock error: {e}");
                    return;
                }
            };
            let mut locked_new_suggestions = match self.new_suggestions.lock() {
                Ok(locked_new_suggestions) => locked_new_suggestions,
                Err(e) => {
                    log::error!("locked_new_suggestions lock error: {e}");
                    return;
                }
            };

            // If there are new suggestions, append or replace them in the existing suggestions.
            if let Some(new_suggestions) = locked_new_suggestions.as_ref() {
                log::debug!("load_suggestions finished");

                // If the offset is 0, replace the existing suggestions with the new ones.
                if self.offset == 0 {
                    *locked_suggestions = Some(new_suggestions.clone());
                    self.offset = new_suggestions.items.len();
                } else if let Some(suggestions) = locked_suggestions.as_mut() {
                    suggestions.items.extend(new_suggestions.items.clone());
                    self.offset = suggestions.items.len();
                } else {
                    *locked_suggestions = Some(new_suggestions.clone());
                    self.offset = new_suggestions.items.len();
                }

                // Increase the offset for the next query and check if there are more suggestions to load.
                self.has_more = self.offset < new_suggestions.total;

                // Loading complete.
                self.load = false;
                self.loading = false;
                *locked_new_suggestions = None;
            }
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
        let input_widget =
            egui::TextEdit::singleline(&mut self.input).hint_text(&self.translations.hint);

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
        let delay = 0.4;

        // Check if enough time passed since last edit.
        let now = ui.input(|i| i.time);
        let debounce_delay_passed =
            (now - self.last_edit_time) > delay && self.input != self.autocomplete_triggered_for;

        // Trigger autocomplete after delay.
        if debounce_delay_passed && !self.loading && (self.minimum_input_length <= self.input.len())
        {
            self.close_suggestions();
            self.autocomplete_triggered_for.clone_from(&self.input);
            self.offset = 0;
            self.loading = true;
        }

        // Trigger autocomplete on first click when input is empty.
        if input_resp.clicked() && self.last_edit_time == 0.0 && self.input.is_empty() {
            self.close_suggestions();
            self.autocomplete_triggered_for.clone_from(&self.input);
            self.offset = 0;
            self.loading = true;
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
                    let cloned_suggestions = Arc::clone(&self.suggestions);
                    let Ok(locked_suggestions) = cloned_suggestions.lock() else {
                        log::error!("locked_suggestions lock error");
                        return;
                    };

                    if let Some(suggestions) = locked_suggestions.as_ref() {
                        self.select_highlighted(suggestions);
                    }
                }
                if i.key_pressed(egui::Key::Escape) {
                    self.close_suggestions();
                }
                if i.key_pressed(egui::Key::Backspace) && self.input.is_empty() {
                    self.selected.pop();
                }
            });
        }
    }

    // Render the dropdown of suggestions.
    fn render_dropdown(&mut self, ui: &mut egui::Ui, suggestions: &SharedSelect2Items) {
        if self.loading {
            ui.label(&self.translations.loading);
        }

        if self.open {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                let Ok(locked_suggestions) = suggestions.lock() else {
                    log::error!("locked_suggestions lock error");
                    return;
                };

                if let Some(suggestions) = locked_suggestions.as_ref() {
                    let mut clicked_index = None;

                    egui::ScrollArea::vertical()
                        .id_salt(ui.id().with(format!("scroll_{}", self.id)))
                        .max_height(self.scroll_max_height)
                        .show(ui, |ui| {
                            for (i, item) in suggestions.items.iter().enumerate() {
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
                        self.select_index(i, suggestions);
                    }
                } else {
                    // There is no suggestions to display.
                    if self.input.is_empty() || self.read_only {
                        ui.label(&self.translations.no_results);
                    } else if !self.read_only
                        && ui
                            .button(format!("{} \"{}\"", self.translations.add, self.input))
                            .clicked()
                    {
                        self.add_new();
                    }
                }
            });
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let response = ui.vertical(|ui| {
            let cloned_suggestions = Arc::clone(&self.suggestions);

            self.render_selected_items(ui);
            self.render_input(ui);
            self.render_keyboard_actions(ui);
            self.render_dropdown(ui, &cloned_suggestions);
        });

        response.response
    }

    // Close the suggestions
    fn close_suggestions(&mut self) {
        let Ok(mut locked_suggestions) = self.suggestions.lock() else {
            log::error!("locked_suggestions lock error");
            return;
        };
        *locked_suggestions = None;
    }

    // Move the highlighted suggestion down.
    fn move_down(&mut self) {
        let Ok(locked_suggestions) = self.suggestions.lock() else {
            log::error!("locked_suggestions lock error");
            return;
        };

        let suggestions_length = match *locked_suggestions {
            Some(ref suggestions) => suggestions.items.len(),
            None => 0,
        };

        if let Some(i) = self.highlighted {
            // An item is highlighted, so move the highlight down.
            // Increment the highlighted index if it's not at the end of the suggestions.
            if i + 1 < suggestions_length {
                self.highlighted = Some(i + 1);
            }
        } else if suggestions_length > 0 {
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
    fn select_highlighted(&mut self, suggestions: &SelectItems) {
        if let Some(i) = self.highlighted {
            // An item is highlighted, select it.
            self.select_index(i, suggestions);
        } else if !self.input.is_empty() {
            self.add_new();
        }
    }

    // Add select_item at index i to the selected items if not already selected.
    fn select_index(&mut self, i: usize, suggestions: &SelectItems) {
        if let Some(select_item) = suggestions.items.get(i).cloned()
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
            self.close_suggestions();
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
            self.close_suggestions();
        }
    }
}
