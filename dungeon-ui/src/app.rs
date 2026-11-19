//! The window of the floor generator, like the `TkinterGenerator` of the Python UI.

use std::time::Duration;

use dungeon_generator::{Floor, Generator};
use eframe::egui::{
    self, Align, Button, CentralPanel, Context, DragValue, Id, Key, Layout, Modifiers, Panel,
    RichText, TextEdit, Theme, Ui, ViewportCommand,
};

use crate::dialog::{Answer, Confirm};
use crate::floor_view::floor_view;
use crate::geometry::View;
use crate::history::History;
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
    ToggleTheme,
    ToggleLegend,
    Quit,
}

/// Shortcuts that work everywhere.
const GLOBAL_SHORTCUTS: [(Modifiers, Key, Action); 3] = [
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

pub struct DungeonApp {
    history: History,
    /// Step by step drawing of the newest floor, `None` when it is fully drawn
    reveal: Option<Reveal>,
    next_seed: String,
    next_floor_id: u32,
    dark: bool,
    show_legend: bool,
    confirm_quit: bool,
    quitting: bool,
    /// Where the floor was drawn in the last frame
    view: Option<View>,
}

impl DungeonApp {
    /// Opens the window with a floor for the seed and floor id.
    pub fn new(ctx: &Context, seed: &str, floor_id: u32) -> Self {
        egui_extras::install_image_loaders(ctx);
        let app = Self {
            history: History::new(generate_floor(seed, floor_id)),
            reveal: None,
            next_seed: random_seed(),
            next_floor_id: floor_id.min(MAX_FLOOR_ID),
            dark: false,
            show_legend: true,
            confirm_quit: false,
            quitting: false,
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

    fn apply(&mut self, action: Action, ctx: &Context, now: f64) {
        match action {
            Action::Generate => {
                let floor = generate_floor(&self.next_seed, self.next_floor_id);
                self.reveal = Some(Reveal::new(&floor, now));
                self.history.push(floor);
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
            Action::ToggleTheme => {
                self.dark = !self.dark;
                self.apply_theme(ctx);
            }
            Action::ToggleLegend => self.show_legend = !self.show_legend,
            Action::Quit => self.confirm_quit = true,
        }
        ctx.request_repaint();
    }

    fn shortcuts(&self, ctx: &Context) -> Vec<Action> {
        let mut actions = Vec::new();
        if self.confirm_quit {
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

    /// Seed, floor id and history position of the shown floor and the controls.
    fn status_bar(&mut self, ui: &mut Ui, actions: &mut Vec<Action>) {
        let muted = self.palette().muted_text;
        ui.horizontal(|ui| {
            let floor = self.history.current();
            let floor_id = floor.floor_id().map_or("?".to_owned(), |id| id.to_string());
            ui.label(RichText::new(format!("Seed {}", floor.seed())).color(muted));
            ui.label(RichText::new(format!("·  Floor {floor_id}  ·")).color(muted));
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

    /// Asks before closing the window, like the Python UI.
    fn quit_dialog(&mut self, ctx: &Context) {
        let dialog = Confirm {
            title: "Quit",
            message: "Do you really want to quit?",
            confirm: "Quit",
            cancel: "Cancel",
            destructive: true,
        };
        match dialog.show(ctx, Id::unique("quit_dialog"), self.palette(), self.dark) {
            Some(Answer::Confirm) => {
                self.confirm_quit = false;
                self.quitting = true;
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
            Some(Answer::Cancel) => self.confirm_quit = false,
            None => {}
        }
    }
}

impl eframe::App for DungeonApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let now = ctx.input(|i| i.time);
        if ctx.input(|i| i.viewport().close_requested()) && !self.quitting {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.confirm_quit = true;
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
                    legend(ui, self.history.current(), palette);
                });
        }
        CentralPanel::default().show(ui, |ui| {
            let visible = self.reveal.as_ref().map(|reveal| reveal.visible_rooms(now));
            let (_, view) = floor_view(ui, self.history.current(), visible.as_ref(), palette);
            self.view = Some(view);
        });
        if self.confirm_quit {
            self.quit_dialog(&ctx);
        }

        for action in actions {
            self.apply(action, &ctx, now);
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

    fn harness() -> Harness<'static, DungeonApp> {
        let mut harness = Harness::builder()
            .with_size(vec2(1400.0, 620.0))
            .build_eframe(|cc| DungeonApp::new(&cc.egui_ctx, "abc", 2));
        harness.step();
        harness
    }

    fn press(harness: &mut Harness<'_, DungeonApp>, key: Key) {
        harness.key_press(key);
        harness.step();
    }

    #[test]
    fn starts_with_the_floor_of_the_seed() {
        let harness = harness();
        let app = harness.state();
        assert_eq!(app.history.len(), 1);
        assert_eq!(
            app.history.current().rooms(),
            generate_floor("abc", 2).rooms()
        );
        assert!(app.reveal.is_none());
        assert_eq!(app.next_seed.len(), 32);
    }

    #[test]
    fn space_generates_a_floor_with_the_next_seed() {
        let mut harness = harness();
        harness.state_mut().next_seed = "xyz".to_owned();
        harness.state_mut().next_floor_id = 4;
        press(&mut harness, Key::Space);

        let app = harness.state();
        assert_eq!(app.history.len(), 2);
        assert_eq!(app.history.current().seed(), "xyz");
        assert_eq!(app.history.current().floor_id(), Some(4));
        assert_ne!(app.next_seed, "xyz");
        assert!(app.reveal.is_some());
    }

    #[test]
    fn the_reveal_ends_after_all_rings() {
        let mut harness = harness();
        press(&mut harness, Key::Space);
        let rings = harness.state().history.current().rooms_by_distance().len();
        // Each step advances the time by a quarter second
        harness.run_steps(rings * 2 + 2);
        assert!(harness.state().reveal.is_none());
    }

    #[test]
    fn arrows_go_through_the_history() {
        let mut harness = harness();
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
        let mut harness = harness();
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
        let mut harness = harness();
        harness.key_press_modifiers(Modifiers::COMMAND, Key::T);
        harness.step();
        assert!(harness.state().dark);
        assert_eq!(harness.ctx.theme(), Theme::Dark);
        press(&mut harness, Key::F1);
        assert!(!harness.state().show_legend);
    }

    #[test]
    fn typing_a_seed_does_not_trigger_shortcuts() {
        let mut harness = harness();
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
        let mut harness = harness();
        harness.key_press_modifiers(Modifiers::COMMAND, Key::Q);
        harness.step();
        assert!(harness.state().confirm_quit);
        harness.step();
        harness.get_by_label("Cancel").click();
        harness.step();
        assert!(!harness.state().confirm_quit);
        assert!(!harness.state().quitting);
    }

    #[test]
    fn the_quit_dialog_works_with_the_keyboard() {
        let mut harness = harness();
        harness.key_press_modifiers(Modifiers::COMMAND, Key::Q);
        harness.step();
        press(&mut harness, Key::Escape);
        assert!(!harness.state().confirm_quit);
        assert!(!harness.state().quitting);

        harness.key_press_modifiers(Modifiers::COMMAND, Key::Q);
        harness.step();
        press(&mut harness, Key::Enter);
        assert!(harness.state().quitting);
    }

    #[test]
    fn menus_open_and_run_their_actions() {
        let mut harness = harness();
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
        let mut harness = harness();
        let view = harness.state().view.expect("the floor was drawn");
        let start = harness
            .state()
            .history
            .current()
            .rooms()
            .iter()
            .find(|room| room.room_type() == RoomType::Start)
            .map(|room| room.position());
        assert_eq!(start, Some(START_ROOM_POSITION));
        harness.hover_at(view.pos(cell_rect(START_ROOM_POSITION).center()));
        harness.run_steps(4);
        assert!(harness.query_by_label_contains("Start Room · #0").is_some());
    }
}
