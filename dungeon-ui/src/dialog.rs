//! Modal dialogs in the style of the application, like `dialog.py` of the Python UI.

use dungeon_generator::RoomType;
use eframe::egui::{
    Align, Button, Color32, Context, Frame, Id, Key, Layout, Margin, Modal, Modifiers, RichText,
    Stroke, vec2,
};

use crate::palette::Palette;

const WIDTH: f32 = 340.0;
const BUTTON_SIZE: [f32; 2] = [88.0, 30.0];

/// A question with a confirmation and a cancel button, or a message with one button.
pub struct Confirm<'a> {
    pub title: &'a str,
    pub message: &'a str,
    pub confirm: &'a str,
    /// Text of the cancel button, `None` for a message with only the confirm button
    pub cancel: Option<&'a str>,
    /// Shows the confirm button in red, for actions like quitting or overwriting
    pub destructive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    Confirm,
    Cancel,
}

impl Confirm<'_> {
    /// Shows the dialog centered over the window. Returns the answer once the user
    /// chose one: Enter confirms, Escape or a click next to the dialog cancels.
    pub fn show(&self, ctx: &Context, id: Id, palette: &Palette, dark: bool) -> Option<Answer> {
        let frame = Frame::new()
            .fill(palette.window)
            .stroke(Stroke::new(1.0, palette.muted_text.gamma_multiply(0.35)))
            .corner_radius(10.0)
            .inner_margin(Margin {
                left: 22,
                right: 22,
                top: 18,
                bottom: 16,
            })
            .shadow(ctx.global_style().visuals.popup_shadow);
        let backdrop = Color32::from_black_alpha(if dark { 140 } else { 90 });

        let modal = Modal::new(id)
            .frame(frame)
            .backdrop_color(backdrop)
            .show(ctx, |ui| {
                ui.set_width(WIDTH);
                ui.label(
                    RichText::new(self.title)
                        .size(17.0)
                        .strong()
                        .color(palette.text),
                );
                ui.add_space(6.0);
                ui.label(RichText::new(self.message).color(palette.muted_text));
                ui.add_space(18.0);
                let buttons = vec2(ui.available_width(), BUTTON_SIZE[1]);
                ui.allocate_ui_with_layout(buttons, Layout::right_to_left(Align::Center), |ui| {
                    let mut confirm =
                        Button::new(self.confirm_text(palette)).min_size(BUTTON_SIZE.into());
                    if self.destructive {
                        confirm = confirm.fill(palette.room(RoomType::Boss));
                    }
                    let confirmed = ui.add(confirm).clicked();
                    let cancelled = self.cancel.is_some_and(|cancel| {
                        ui.add_space(2.0);
                        ui.add(Button::new(cancel).min_size(BUTTON_SIZE.into()))
                            .clicked()
                    });
                    (confirmed, cancelled)
                })
                .inner
            });

        let (confirmed, cancelled) = modal.inner;
        let enter = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter));
        if confirmed || (enter && modal.is_top_modal) {
            Some(Answer::Confirm)
        } else if cancelled || modal.should_close() {
            Some(Answer::Cancel)
        } else {
            None
        }
    }

    fn confirm_text(&self, palette: &Palette) -> RichText {
        let text = RichText::new(self.confirm).strong();
        if self.destructive {
            text.color(Color32::WHITE)
        } else {
            text.color(palette.text)
        }
    }
}
