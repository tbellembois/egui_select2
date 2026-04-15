use eframe::egui;
use egui::{Response, Ui};

/// A select item.
/// The `id` is used to identify the item, and the `label` is the text to display.
/// The `id` is None for new items only (when `read_only` is false).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct SelectItem {
    pub id: Option<String>,
    pub label: String,
}

/// A collection of select items retrieved by the `load_suggestions` function.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct SelectItems {
    pub items: Vec<SelectItem>,
    pub total: usize,
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
    pub load_suggestions: Box<dyn Fn(usize, usize, &str) -> SelectItems>,
    /// The function to format a suggestion in the dropdown.
    pub format_suggestion: Box<dyn Fn(&mut Ui, bool, &SelectItem) -> Response>,
    /// The maximum number of suggestions to load at once.
    pub limit: usize,
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
            load_suggestions: Box::new(|_, _, _| SelectItems::default()),
            format_suggestion: Box::new(|ui, selected, select_item| {
                ui.add(egui::Button::new(&select_item.label).selected(selected))
            }),
            limit: 10,
            offset: 0,
            scroll_max_height: 150.0,
            input: String::default(),
            selected: Vec::new(),
            suggestions: SelectItems::default(),
            has_more: true,
            loading: false,
            read_only: true,
            highlighted: None,
            open: false,
            minimum_input_length: 0,
            last_edit_time: 0.0,
            autocomplete_triggered_for: String::default(),
            close_on_select: true,
            disabled: false,
            multiple: false,
        }
    }
}

impl EguiSelect2 {
    /// Creates a new `EguiSelect2` with the given parameters.
    pub fn new(
        read_only: bool,
        min_input_length: usize,
        limit: usize,
        load_suggestions: impl Fn(usize, usize, &str) -> SelectItems + 'static,
        format_suggestion: impl Fn(&mut Ui, bool, &SelectItem) -> Response + 'static,
    ) -> Self {
        EguiSelect2 {
            load_suggestions: Box::new(load_suggestions),
            format_suggestion: Box::new(format_suggestion),
            limit,
            read_only,
            minimum_input_length: min_input_length,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn execute_load(&self, limit: usize, offset: usize, query: &str) -> SelectItems {
        (self.load_suggestions)(limit, offset, query)
    }

    /// Checks if the widget is loading suggestions and loads them if necessary.
    /// This method must be called outside the `ui` method of the widget.
    /// See examples for usage.
    pub fn check_loading(&mut self) {
        if self.loading {
            let suggestions = self.execute_load(self.limit, self.offset, &self.input);
            self.set_suggestions(suggestions);
            self.loading = false;
        }
    }

    pub fn request_next_page(&mut self) {
        if self.has_more && !self.loading {
            self.loading = true;
        }
    }

    pub fn reset_pagination(&mut self) {
        self.offset = 0;
        self.has_more = true;
        self.loading = false;
    }

    pub fn set_suggestions(&mut self, suggestion_items: SelectItems) {
        if self.offset == 0 {
            self.suggestions = suggestion_items.clone();
        } else {
            self.suggestions.items.extend(suggestion_items.items);
        }

        self.offset = self.suggestions.items.len();
        self.has_more = self.offset < suggestion_items.total;
        self.loading = false;
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let response = ui.vertical(|ui| {
            // --- Selected chips ---
            ui.horizontal_wrapped(|ui| {
                let mut remove_idx = None;

                for (i, item) in self.selected.iter().enumerate() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(&item.label);
                            if !self.disabled && ui.button("✕").clicked() {
                                remove_idx = Some(i);
                            }
                        });
                    });
                }

                if let Some(i) = remove_idx {
                    self.selected.remove(i);
                }
            });

            // --- Input ---
            let input_widget = egui::TextEdit::singleline(&mut self.input);
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
            let should_autocomplete = (now - self.last_edit_time) > delay
                && self.input != self.autocomplete_triggered_for;

            // Trigger autocomplete after delay.
            if should_autocomplete && !self.loading && self.minimum_input_length <= self.input.len()
            {
                self.autocomplete_triggered_for.clone_from(&self.input);
                self.reset_pagination();
                self.loading = true;
            }

            // Open suggestions on focus.
            if input_resp.has_focus() && !self.loading {
                self.open = true;
            }

            // --- Loading ---
            if self.loading {
                ui.label("Loading...");
            }

            // --- Keyboard ---
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

            // --- Dropdown ---
            if self.open {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    if self.suggestions.items.is_empty() {
                        if self.input.is_empty() || self.read_only {
                            ui.label("No results");
                        } else if !self.read_only
                            && ui.button(format!("Add \"{}\"", self.input)).clicked()
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

                                if self.has_more && !self.loading && ui.small_button("+").clicked()
                                {
                                    self.request_next_page();
                                }
                            });

                        if let Some(i) = clicked_index {
                            self.select_index(i);
                        }
                    }
                });
            }
        });

        response.response
    }

    fn move_down(&mut self) {
        if let Some(i) = self.highlighted {
            if i + 1 < self.suggestions.items.len() {
                self.highlighted = Some(i + 1);
            }
        } else if !self.suggestions.items.is_empty() {
            self.highlighted = Some(0);
        }
    }

    fn move_up(&mut self) {
        if let Some(i) = self.highlighted
            && i > 0
        {
            self.highlighted = Some(i - 1);
        }
    }

    fn select_highlighted(&mut self) {
        if let Some(i) = self.highlighted {
            self.select_index(i);
        } else if !self.input.is_empty() {
            self.add_new();
        }
    }

    fn select_index(&mut self, i: usize) {
        if let Some(item) = self.suggestions.items.get(i).cloned()
            && !self
                .selected
                .iter()
                .any(|x| x.id == item.id && x.label == item.label)
        {
            if self.multiple {
                self.selected.push(item);
            } else {
                self.selected.clear();
                self.selected.push(item);
            }
        }

        self.input.clear();
        if self.close_on_select {
            self.open = false;
        }
    }

    fn add_new(&mut self) {
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
