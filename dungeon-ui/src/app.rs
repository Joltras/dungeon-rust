//! The window of the floor generator, like the `TkinterGenerator` of the Python UI.

use std::path::{Path, PathBuf};
use std::time::Duration;

use dungeon_generator::{Floor, Generator};
use eframe::egui::{
    self, Align, Button, CentralPanel, Context, DragValue, Id, Key, Layout, Modifiers, Panel,
    RichText, TextEdit, Theme, Ui, ViewportCommand,
};

use crate::dialog::{Answer, Confirm};
use crate::files::{
    FilePicker, default_file_name, display_name, read_floor, with_json_suffix, write_floor,
};
use crate::floor_view::floor_view;
use crate::geometry::View;
use crate::history::{Entry, History};
use crate::legend::{self, legend};
use crate::palette::Palette;
use crate::reveal::Reveal;

pub const TITLE: &str = "Floor Generator";
/// Highest floor id that can be entered in the floor field
pub const MAX_FLOOR_ID: u32 = 99;
const NEXT_SEED_ID: &str = "next_seed";

/// Creates a random seed of 32 hex characters, like `secrets.token_hex(16)` in Python.
pub fn random_seed() -> String {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("the operating system provides random numbers");
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn generate_floor(seed: &str, floor_id: u32) -> Floor {
    let mut generator = Generator::new(seed, floor_id);
    generator.generate();
    generator.into_floor()
}

/// Something the user wants to do, from a shortcut, the menu or a button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Generate,
    Back,
    Forward,
    HigherFloorId,
    LowerFloorId,
    Open,
    Save,
    SaveAs,
    ToggleTheme,
    ToggleLegend,
    Quit,
}

/// Shortcuts that work everywhere. "Save As" comes before "Save",
/// because the first matching shortcut takes the key press.
const GLOBAL_SHORTCUTS: [(Modifiers, Key, Action); 6] = [
    (
        Modifiers::COMMAND.plus(Modifiers::SHIFT),
        Key::S,
        Action::SaveAs,
    ),
    (Modifiers::COMMAND, Key::S, Action::Save),
    (Modifiers::COMMAND, Key::O, Action::Open),
    (Modifiers::COMMAND, Key::Q, Action::Quit),
    (Modifiers::COMMAND, Key::T, Action::ToggleTheme),
    (Modifiers::NONE, Key::F1, Action::ToggleLegend),
];

/// Single key shortcuts, ignored while a field or button has the keyboard focus,
/// so typing a seed or pressing a button with the keyboard does not trigger them.
const KEY_SHORTCUTS: [(Key, Action); 5] = [
    (Key::Space, Action::Generate),
    (Key::ArrowLeft, Action::Back),
    (Key::ArrowRight, Action::Forward),
    (Key::PageUp, Action::HigherFloorId),
    (Key::PageDown, Action::LowerFloorId),
];

/// The dialog that is shown above the window.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Dialog {
    Quit,
    /// Asks before saving the shown floor over an existing file
    Overwrite(PathBuf),
    Message {
        title: String,
        message: String,
    },
}

impl Dialog {
    fn message(title: &str, message: impl Into<String>) -> Self {
        Self::Message {
            title: title.to_owned(),
            message: message.into(),
        }
    }
}

pub struct DungeonApp {
    history: History,
    /// Step by step drawing of the newest floor, `None` when it is fully drawn
    reveal: Option<Reveal>,
    next_seed: String,
    next_floor_id: u32,
    dark: bool,
    show_legend: bool,
    dialog: Option<Dialog>,
    quitting: bool,
    /// Folder for generated floors until they are saved somewhere else
    output_folder: PathBuf,
    files: Box<dyn FilePicker>,
    /// Where the floor was drawn in the last frame
    view: Option<View>,
}

impl DungeonApp {
    /// Opens the window with a floor for the seed and floor id.
    /// Generated floors are saved in the output folder by default.
    pub fn new(
        ctx: &Context,
        seed: &str,
        floor_id: u32,
        output_folder: PathBuf,
        files: Box<dyn FilePicker>,
    ) -> Self {
        egui_extras::install_image_loaders(ctx);
        let first = Entry::generated(
            generate_floor(seed, floor_id),
            output_folder.join(default_file_name()),
        );
        let app = Self {
            history: History::new(first),
            reveal: None,
            next_seed: random_seed(),
            next_floor_id: floor_id.min(MAX_FLOOR_ID),
            dark: false,
            show_legend: true,
            dialog: None,
            quitting: false,
            output_folder,
            files,
            view: None,
        };
        app.apply_theme(ctx);
        app
    }

    fn palette(&self) -> &'static Palette {
        Palette::get(self.dark)
    }

    fn apply_theme(&self, ctx: &Context) {
        let theme = if self.dark { Theme::Dark } else { Theme::Light };
        ctx.set_theme(theme);
        ctx.set_visuals_of(theme, self.palette().visuals(self.dark));
    }

    /// Folder of the shown floor's file, or the working directory if it does not exist yet.
    fn current_folder(&self) -> PathBuf {
        self.history
            .current()
            .file
            .parent()
            .filter(|folder| folder.is_dir())
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
    }

    fn apply(&mut self, action: Action, ctx: &Context, window: &eframe::Frame, now: f64) {
        match action {
            Action::Generate => {
                let floor = generate_floor(&self.next_seed, self.next_floor_id);
                self.reveal = Some(Reveal::new(&floor, now));
                let file = self.output_folder.join(default_file_name());
                self.history.push(Entry::generated(floor, file));
                self.next_seed = random_seed();
            }
            Action::Back => {
                if self.history.back() {
                    self.reveal = None;
                }
            }
            Action::Forward => {
                if self.history.forward() {
                    self.reveal = None;
                }
            }
            Action::HigherFloorId => {
                self.next_floor_id = (self.next_floor_id + 1).min(MAX_FLOOR_ID);
            }
            Action::LowerFloorId => self.next_floor_id = self.next_floor_id.saturating_sub(1),
            Action::Open => {
                if let Some(path) = self.files.pick_open(window, &self.current_folder()) {
                    self.open(path);
                }
            }
            Action::Save => {
                let entry = self.history.current();
                if !entry.has_file && entry.file.exists() {
                    self.dialog = Some(Dialog::Overwrite(entry.file.clone()));
                } else {
                    self.save(entry.file.clone());
                }
            }
            Action::SaveAs => self.save_as(window),
            Action::ToggleTheme => {
                self.dark = !self.dark;
                self.apply_theme(ctx);
            }
            Action::ToggleLegend => self.show_legend = !self.show_legend,
            Action::Quit => self.dialog = Some(Dialog::Quit),
        }
        ctx.request_repaint();
    }

    /// Shows the floor of the file as the newest floor of the history.
    fn open(&mut self, path: PathBuf) {
        match read_floor(&path) {
            Ok(floor) => {
                self.history.push(Entry::opened(floor, path));
                self.reveal = None;
            }
            Err(error) => {
                let message = format!("{} could not be opened.\n{error}", display_name(&path));
                self.dialog = Some(Dialog::message("Could not open file", message));
            }
        }
    }

    fn save_as(&mut self, window: &eframe::Frame) {
        let name = display_name(&self.history.current().file);
        let Some(picked) = self.files.pick_save(window, &self.current_folder(), &name) else {
            return;
        };
        let path = with_json_suffix(picked.clone());
        // The file dialog only asked about the file without the added suffix
        if path != picked && path.exists() {
            self.dialog = Some(Dialog::Overwrite(path));
        } else {
            self.save(path);
        }
    }

    /// Saves the shown floor to the file, which becomes the file of the floor.
    fn save(&mut self, path: PathBuf) {
        let entry = self.history.current_mut();
        self.dialog = Some(match write_floor(&entry.floor, &path) {
            Ok(()) => {
                entry.file = path;
                entry.has_file = true;
                Dialog::message("Save", "File saved successfully!")
            }
            Err(error) => {
                let message = format!("{} could not be saved.\n{error}", display_name(&path));
                Dialog::message("Could not save file", message)
            }
        });
    }

    fn shortcuts(&self, ctx: &Context) -> Vec<Action> {
        let mut actions = Vec::new();
        if self.dialog.is_some() {
            return actions;
        }
        let focused = ctx.egui_wants_keyboard_input();
        ctx.input_mut(|input| {
            for (modifiers, key, action) in GLOBAL_SHORTCUTS {
                if input.consume_key(modifiers, key) {
                    actions.push(action);
                }
            }
            if !focused {
                for (key, action) in KEY_SHORTCUTS {
                    if input.consume_key(Modifiers::NONE, key) {
                        actions.push(action);
                    }
                }
            }
        });
        actions
    }

    fn menu_bar(&mut self, ui: &mut Ui, actions: &mut Vec<Action>) {
        let mut item = |ui: &mut Ui, button: Button, action: Action| {
            if ui.add(button).clicked() {
                actions.push(action);
            }
        };
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                item(
                    ui,
                    Button::new("Open…").shortcut_text("Ctrl+O"),
                    Action::Open,
                );
                item(
                    ui,
                    Button::new("Save").shortcut_text("Ctrl+S"),
                    Action::Save,
                );
                let save_as = Button::new("Save As…").shortcut_text("Ctrl+Shift+S");
                item(ui, save_as, Action::SaveAs);
                ui.separator();
                item(
                    ui,
                    Button::new("Exit").shortcut_text("Ctrl+Q"),
                    Action::Quit,
                );
            });
            ui.menu_button("View", |ui| {
                let dark = Button::selectable(self.dark, "Dark Theme").shortcut_text("Ctrl+T");
                item(ui, dark, Action::ToggleTheme);
                let legend = Button::selectable(self.show_legend, "Legend").shortcut_text("F1");
                item(ui, legend, Action::ToggleLegend);
            });
            ui.menu_button("Floor", |ui| {
                item(
                    ui,
                    Button::new("Generate").shortcut_text("Space"),
                    Action::Generate,
                );
                ui.separator();
                let higher = Button::new("Higher Floor ID").shortcut_text("Page Up");
                item(ui, higher, Action::HigherFloorId);
                let lower = Button::new("Lower Floor ID").shortcut_text("Page Down");
                item(ui, lower, Action::LowerFloorId);
                ui.separator();
                item(ui, Button::new("Back").shortcut_text("⬅"), Action::Back);
                item(
                    ui,
                    Button::new("Forward").shortcut_text("➡"),
                    Action::Forward,
                );
            });
        });
    }

    /// File, seed, floor id and history position of the shown floor and the controls.
    fn status_bar(&mut self, ui: &mut Ui, actions: &mut Vec<Action>) {
        let muted = self.palette().muted_text;
        ui.horizontal(|ui| {
            let entry = self.history.current();
            let floor_id = entry
                .floor
                .floor_id()
                .map_or("?".to_owned(), |id| id.to_string());
            let path = std::path::absolute(&entry.file).unwrap_or_else(|_| entry.file.clone());
            ui.label(RichText::new(display_name(&entry.file)).strong())
                .on_hover_text(path.display().to_string());
            let seed = entry.floor.seed();
            ui.label(RichText::new(format!("·  Seed {seed}  ·  Floor {floor_id}  ·")).color(muted));
            ui.label(RichText::new(self.history.position_text()).color(muted))
                .on_hover_text("Position in the history of generated floors");

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let enter = |ui: &Ui| ui.input(|i| i.key_pressed(Key::Enter));
                let seed = ui
                    .add(
                        TextEdit::singleline(&mut self.next_seed)
                            .id(Id::unique(NEXT_SEED_ID))
                            .desired_width(250.0),
                    )
                    .on_hover_text("Seed for the next floor, press Enter to generate it");
                if seed.lost_focus() && enter(ui) {
                    actions.push(Action::Generate);
                }
                ui.label(RichText::new("Next seed").color(muted));
                ui.add_space(10.0);
                let floor_id = ui
                    .add(DragValue::new(&mut self.next_floor_id).range(0..=MAX_FLOOR_ID))
                    .on_hover_text(
                        "Floor id for the next floor, higher ids get more rooms\n\
                         (Page Up / Page Down)",
                    );
                if floor_id.lost_focus() && enter(ui) {
                    actions.push(Action::Generate);
                }
                ui.label(RichText::new("Floor").color(muted));
                ui.add_space(14.0);
                let buttons = [
                    ("▶", "Forward to the next floor (➡)", Action::Forward),
                    ("Generate", "Generate a new floor (Space)", Action::Generate),
                    ("◀", "Back to the previous floor (⬅)", Action::Back),
                ];
                for (text, hint, action) in buttons {
                    if ui.button(text).on_hover_text(hint).clicked() {
                        actions.push(action);
                    }
                }
            });
        });
    }

    /// Shows the open dialog and handles the answer.
    fn show_dialog(&mut self, ctx: &Context) {
        let Some(dialog) = self.dialog.clone() else {
            return;
        };
        let (palette, dark) = (self.palette(), self.dark);
        let id = Id::unique("dialog");
        match dialog {
            Dialog::Quit => {
                let answer = Confirm {
                    title: "Quit",
                    message: "Do you really want to quit?",
                    confirm: "Quit",
                    cancel: Some("Cancel"),
                    destructive: true,
                }
                .show(ctx, id, palette, dark);
                match answer {
                    Some(Answer::Confirm) => {
                        self.dialog = None;
                        self.quitting = true;
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                    }
                    Some(Answer::Cancel) => self.dialog = None,
                    None => {}
                }
            }
            Dialog::Overwrite(path) => {
                let message = format!(
                    "{} already exists. Do you want to overwrite it?",
                    display_name(&path)
                );
                let answer = Confirm {
                    title: "File already exists",
                    message: &message,
                    confirm: "Overwrite",
                    cancel: Some("Cancel"),
                    destructive: true,
                }
                .show(ctx, id, palette, dark);
                match answer {
                    Some(Answer::Confirm) => self.save(path),
                    Some(Answer::Cancel) => self.dialog = None,
                    None => {}
                }
            }
            Dialog::Message { title, message } => {
                let dialog = Confirm {
                    title: &title,
                    message: &message,
                    confirm: "OK",
                    cancel: None,
                    destructive: false,
                };
                if dialog.show(ctx, id, palette, dark).is_some() {
                    self.dialog = None;
                }
            }
        }
    }
}

impl eframe::App for DungeonApp {
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let now = ctx.input(|i| i.time);
        if ctx.input(|i| i.viewport().close_requested()) && !self.quitting {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.dialog = Some(Dialog::Quit);
        }
        if self
            .reveal
            .as_ref()
            .is_some_and(|reveal| reveal.is_finished(now))
        {
            self.reveal = None;
        }

        let mut actions = self.shortcuts(&ctx);
        Panel::top("menu_bar").show(ui, |ui| self.menu_bar(ui, &mut actions));
        Panel::bottom("status_bar").show(ui, |ui| {
            ui.add_space(4.0);
            self.status_bar(ui, &mut actions);
            ui.add_space(4.0);
        });
        let palette = self.palette();
        if self.show_legend {
            Panel::right("legend")
                .resizable(false)
                .exact_size(legend::WIDTH + 24.0)
                .show(ui, |ui| {
                    ui.add_space(8.0);
                    legend(ui, &self.history.current().floor, palette);
                });
        }
        CentralPanel::default().show(ui, |ui| {
            let visible = self.reveal.as_ref().map(|reveal| reveal.visible_rooms(now));
            let floor = &self.history.current().floor;
            let (_, view) = floor_view(ui, floor, visible.as_ref(), palette);
            self.view = Some(view);
        });
        self.show_dialog(&ctx);

        for action in actions {
            self.apply(action, &ctx, frame, now);
        }
        if let Some(reveal) = &self.reveal {
            ctx.request_repaint_after(Duration::from_secs_f64(reveal.time_to_next_step(now)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::cell_rect;
    use dungeon_generator::RoomType;
    use dungeon_generator::utils::constants::START_ROOM_POSITION;
    use egui::vec2;
    use egui_kittest::Harness;
    use egui_kittest::kittest::{self, Queryable};
    use std::cell::RefCell;
    use std::fs;
    use std::rc::Rc;
    use tempfile::TempDir;

    /// File dialog that returns the paths the test chose, in a temporary folder.
    #[derive(Clone)]
    struct FakeFiles {
        dir: Rc<TempDir>,
        open: Rc<RefCell<Option<PathBuf>>>,
        save: Rc<RefCell<Option<PathBuf>>>,
    }

    impl FakeFiles {
        fn path(&self, name: &str) -> PathBuf {
            self.dir.path().join(name)
        }

        fn will_open(&self, path: PathBuf) {
            *self.open.borrow_mut() = Some(path);
        }

        fn will_save(&self, path: PathBuf) {
            *self.save.borrow_mut() = Some(path);
        }
    }

    impl FilePicker for FakeFiles {
        fn pick_open(&self, _: &eframe::Frame, _: &Path) -> Option<PathBuf> {
            self.open.borrow_mut().take()
        }

        fn pick_save(&self, _: &eframe::Frame, _: &Path, _: &str) -> Option<PathBuf> {
            self.save.borrow_mut().take()
        }
    }

    fn harness() -> (Harness<'static, DungeonApp>, FakeFiles) {
        let files = FakeFiles {
            dir: Rc::new(tempfile::tempdir().unwrap()),
            open: Rc::default(),
            save: Rc::default(),
        };
        let output = files.path("generation");
        let picker = Box::new(files.clone());
        let mut harness = Harness::builder()
            .with_size(vec2(1400.0, 620.0))
            .build_eframe(move |cc| DungeonApp::new(&cc.egui_ctx, "abc", 2, output, picker));
        harness.step();
        (harness, files)
    }

    fn shortcut(harness: &mut Harness<'_, DungeonApp>, modifiers: Modifiers, key: Key) {
        harness.key_press_modifiers(modifiers, key);
        harness.step();
    }

    fn dialog(harness: &Harness<'_, DungeonApp>) -> Option<Dialog> {
        harness.state().dialog.clone()
    }

    fn press(harness: &mut Harness<'_, DungeonApp>, key: Key) {
        harness.key_press(key);
        harness.step();
    }

    #[test]
    fn starts_with_the_floor_of_the_seed() {
        let (harness, _) = harness();
        let app = harness.state();
        assert_eq!(app.history.len(), 1);
        assert_eq!(
            app.history.current().floor.rooms(),
            generate_floor("abc", 2).rooms()
        );
        assert!(app.reveal.is_none());
        assert_eq!(app.next_seed.len(), 32);
    }

    #[test]
    fn space_generates_a_floor_with_the_next_seed() {
        let (mut harness, _) = harness();
        harness.state_mut().next_seed = "xyz".to_owned();
        harness.state_mut().next_floor_id = 4;
        press(&mut harness, Key::Space);

        let app = harness.state();
        assert_eq!(app.history.len(), 2);
        assert_eq!(app.history.current().floor.seed(), "xyz");
        assert_eq!(app.history.current().floor.floor_id(), Some(4));
        assert_ne!(app.next_seed, "xyz");
        assert!(app.reveal.is_some());
    }

    #[test]
    fn the_reveal_ends_after_all_rings() {
        let (mut harness, _) = harness();
        press(&mut harness, Key::Space);
        let rings = harness
            .state()
            .history
            .current()
            .floor
            .rooms_by_distance()
            .len();
        // Each step advances the time by a quarter second
        harness.run_steps(rings * 2 + 2);
        assert!(harness.state().reveal.is_none());
    }

    #[test]
    fn arrows_go_through_the_history() {
        let (mut harness, _) = harness();
        press(&mut harness, Key::Space);
        press(&mut harness, Key::ArrowLeft);
        assert_eq!(harness.state().history.position_text(), "1 / 2");
        assert!(harness.state().reveal.is_none());
        press(&mut harness, Key::ArrowLeft);
        assert_eq!(harness.state().history.position_text(), "1 / 2");
        press(&mut harness, Key::ArrowRight);
        assert_eq!(harness.state().history.position_text(), "2 / 2");
    }

    #[test]
    fn page_keys_change_the_floor_id_within_its_range() {
        let (mut harness, _) = harness();
        press(&mut harness, Key::PageUp);
        assert_eq!(harness.state().next_floor_id, 3);
        harness.state_mut().next_floor_id = MAX_FLOOR_ID;
        press(&mut harness, Key::PageUp);
        assert_eq!(harness.state().next_floor_id, MAX_FLOOR_ID);
        harness.state_mut().next_floor_id = 0;
        press(&mut harness, Key::PageDown);
        assert_eq!(harness.state().next_floor_id, 0);
    }

    #[test]
    fn theme_and_legend_can_be_switched() {
        let (mut harness, _) = harness();
        harness.key_press_modifiers(Modifiers::COMMAND, Key::T);
        harness.step();
        assert!(harness.state().dark);
        assert_eq!(harness.ctx.theme(), Theme::Dark);
        press(&mut harness, Key::F1);
        assert!(!harness.state().show_legend);
    }

    #[test]
    fn typing_a_seed_does_not_trigger_shortcuts() {
        let (mut harness, _) = harness();
        harness
            .ctx
            .memory_mut(|m| m.request_focus(Id::unique(NEXT_SEED_ID)));
        harness.step();
        press(&mut harness, Key::Space);
        press(&mut harness, Key::ArrowLeft);
        assert_eq!(harness.state().history.len(), 1);
    }

    #[test]
    fn quitting_asks_first() {
        let (mut harness, _) = harness();
        harness.key_press_modifiers(Modifiers::COMMAND, Key::Q);
        harness.step();
        assert_eq!(dialog(&harness), Some(Dialog::Quit));
        harness.step();
        harness.get_by_label("Cancel").click();
        harness.step();
        assert_eq!(dialog(&harness), None);
        assert!(!harness.state().quitting);
    }

    #[test]
    fn the_quit_dialog_works_with_the_keyboard() {
        let (mut harness, _) = harness();
        harness.key_press_modifiers(Modifiers::COMMAND, Key::Q);
        harness.step();
        press(&mut harness, Key::Escape);
        assert_eq!(dialog(&harness), None);
        assert!(!harness.state().quitting);

        harness.key_press_modifiers(Modifiers::COMMAND, Key::Q);
        harness.step();
        press(&mut harness, Key::Enter);
        assert!(harness.state().quitting);
    }

    #[test]
    fn menus_open_and_run_their_actions() {
        let (mut harness, _) = harness();
        let menu = |label| kittest::by().role(egui::Role::Button).label(label);
        for label in ["File", "View"] {
            harness.get(menu(label)).click();
            harness.step();
            harness.key_press(Key::Escape);
            harness.step();
        }
        harness.get(menu("Floor")).click();
        harness.run_steps(2);
        harness.get_by_label_contains("Higher Floor ID").click();
        harness.step();
        assert_eq!(harness.state().next_floor_id, 3);
    }

    #[test]
    fn hovering_a_room_shows_its_details() {
        let (mut harness, _) = harness();
        let view = harness.state().view.expect("the floor was drawn");
        let start = harness
            .state()
            .history
            .current()
            .floor
            .rooms()
            .iter()
            .find(|room| room.room_type() == RoomType::Start)
            .map(|room| room.position());
        assert_eq!(start, Some(START_ROOM_POSITION));
        harness.hover_at(view.pos(cell_rect(START_ROOM_POSITION).center()));
        harness.run_steps(4);
        assert!(harness.query_by_label_contains("Start Room · #0").is_some());
    }

    #[test]
    fn opens_a_floor_file() {
        let (mut harness, files) = harness();
        let path = files.path("my-floor.json");
        let floor = generate_floor("opened", 5);
        write_floor(&floor, &path).unwrap();

        files.will_open(path.clone());
        shortcut(&mut harness, Modifiers::COMMAND, Key::O);
        let app = harness.state();
        assert_eq!(app.history.len(), 2);
        assert_eq!(app.history.current().floor.rooms(), floor.rooms());
        assert_eq!(app.history.current().file, path);
        assert!(app.reveal.is_none());
        assert_eq!(dialog(&harness), None);
    }

    #[test]
    fn reports_files_that_cannot_be_opened() {
        let (mut harness, files) = harness();
        let path = files.path("broken.json");
        fs::write(&path, "no floor").unwrap();

        files.will_open(path);
        shortcut(&mut harness, Modifiers::COMMAND, Key::O);
        assert_eq!(harness.state().history.len(), 1);
        let Some(Dialog::Message { title, message }) = dialog(&harness) else {
            panic!("expected an error message");
        };
        assert_eq!(title, "Could not open file");
        assert!(
            message.starts_with("broken.json could not be opened."),
            "{message}"
        );
    }

    #[test]
    fn cancelling_the_file_dialog_does_nothing() {
        let (mut harness, _) = harness();
        shortcut(&mut harness, Modifiers::COMMAND, Key::O);
        shortcut(
            &mut harness,
            Modifiers::COMMAND.plus(Modifiers::SHIFT),
            Key::S,
        );
        assert_eq!(harness.state().history.len(), 1);
        assert_eq!(dialog(&harness), None);
    }

    #[test]
    fn saves_a_generated_floor_and_asks_before_overwriting() {
        let (mut harness, files) = harness();
        let path = files.path("generation/floor.json");
        shortcut(&mut harness, Modifiers::COMMAND, Key::S);
        assert_eq!(
            dialog(&harness),
            Some(Dialog::message("Save", "File saved successfully!"))
        );
        press(&mut harness, Key::Enter);
        assert_eq!(dialog(&harness), None);
        let first = read_floor(&path).unwrap();
        assert_eq!(first.seed(), "abc");

        // A new floor has the same default file, so saving it asks first
        press(&mut harness, Key::Space);
        shortcut(&mut harness, Modifiers::COMMAND, Key::S);
        assert_eq!(dialog(&harness), Some(Dialog::Overwrite(path.clone())));
        press(&mut harness, Key::Escape);
        assert_eq!(read_floor(&path).unwrap().seed(), "abc");

        shortcut(&mut harness, Modifiers::COMMAND, Key::S);
        press(&mut harness, Key::Enter);
        let seed = harness.state().history.current().floor.seed().to_owned();
        assert_eq!(read_floor(&path).unwrap().seed(), seed);

        // Once saved, the file belongs to the floor and is saved without asking
        press(&mut harness, Key::Enter);
        shortcut(&mut harness, Modifiers::COMMAND, Key::S);
        assert_eq!(
            dialog(&harness),
            Some(Dialog::message("Save", "File saved successfully!"))
        );
    }

    #[test]
    fn save_as_adds_the_json_suffix_and_keeps_the_new_file() {
        let (mut harness, files) = harness();
        files.will_save(files.path("my-floor"));
        shortcut(
            &mut harness,
            Modifiers::COMMAND.plus(Modifiers::SHIFT),
            Key::S,
        );

        let path = files.path("my-floor.json");
        assert_eq!(read_floor(&path).unwrap().seed(), "abc");
        assert_eq!(harness.state().history.current().file, path);
        assert!(!files.path("generation").exists());
    }

    #[test]
    fn save_as_asks_if_the_added_suffix_hits_an_existing_file() {
        let (mut harness, files) = harness();
        fs::write(files.path("taken.json"), "old").unwrap();
        files.will_save(files.path("taken"));
        shortcut(
            &mut harness,
            Modifiers::COMMAND.plus(Modifiers::SHIFT),
            Key::S,
        );
        assert_eq!(
            dialog(&harness),
            Some(Dialog::Overwrite(files.path("taken.json")))
        );
        assert_eq!(fs::read_to_string(files.path("taken.json")).unwrap(), "old");
    }
}
