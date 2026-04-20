# A select2 like widget for [egui](https://github.com/emilk/egui)

- WASM compatible.
- Support local or remote data fetching.
- Possible custom rendering of the drop down items.
- Multiple or unique selection.
- Can be disabled.
- Add new entries (when `read_only` is false)
- Suggestions auto close on select or not.

There is space for improvements. Pull requests are welcome.

![basic](screenshots/basic.png)
![add](screenshots/add.png)
![pictures](screenshots/pictures.png)

## Inspiration

<https://select2.org/>

## Usage

Typical usage in egui:

Define your select in your app state.

```
struct MyApp {
    my_select: EguiSelect2,
...
}
```

Define your load_suggestions function.

```
fn my_load_suggestions(limit: usize, offset: usize, query: &str) -> SelectItems {
...
SelectItems { items, total }
}
```

Initialize your select with default values.
And attach your load_suggestions function.

```
impl Default for MyApp {
    fn default() -> Self {
        let mut my_select = EguiSelect2::default();
        // required
        my_select.load_suggestions = Box::new(my_load_suggestions);

        Self { my_select }
    }
}
```

Use the select in your UI.
Don't forget to call `check_loading` before `ui`.

```
impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // required
        self.my_select.check_loading();
        self.my_select.ui(ui);
    }
}
```

## Parameters

- `load_suggestions: Box<dyn Fn(usize, usize, &str) -> SelectItems>` The function to load suggestions. REQUIRED

- `format_suggestion: Box<dyn Fn(&mut Ui, bool, &SelectItem) -> Response>` The function to format a suggestion in the dropdown.

- `maximum_suggestions_number: usize` The maximum number of suggestions to load at once.

- `minimum_input_length: usize` The minimum number of characters required to trigger a suggestion load.

- `scroll_max_height: f32` The scroll max height.

- `read_only: bool` Whether the widget is read-only. Setting this to `false` allows the user to enter new items.

- `close_on_select` Whether to close the widget when a suggestion is selected.

- `disabled` Whether the widget is disabled.

- `multiple` Whether the widget allows multiple selections.

## Data

The suggestions are represented as a struct:

```bash
pub struct SelectItems {
    pub items: Vec<SelectItem>,
    pub total: usize,
}
```

with `SelectItem`:

```bash
pub struct SelectItem {
    pub id: Option<String>,
    pub label: String,
}
```

`id` is expected to be set for existing suggestions and `None` for newly entered items.

## Selected values

Selected values can be retrieved with the `selected` parameters as a `Vec<SelectItem>`.

```bash
self.my_select.selected.iter().for_each(|item| {
    ui.label(item.label.clone());
});
```

## Run examples

```bash
cargo run --example basic|remote|pictures|disabled
```
