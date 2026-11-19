//! Legend next to the floor with the room types and how often they appear.

use dungeon_generator::{Floor, RoomType};
use eframe::egui::{Align2, FontId, Pos2, Rect, RichText, Sense, Ui, pos2, vec2};

use crate::floor_view::paint_room;
use crate::geometry::View;
use crate::palette::Palette;

const ROW_HEIGHT: f32 = 44.0;
pub const WIDTH: f32 = 260.0;
/// Width of the cell the sample room is drawn in
const SAMPLE_WIDTH: f32 = 72.0;

pub fn legend(ui: &mut Ui, floor: &Floor, palette: &Palette) {
    ui.label(RichText::new("Legend").strong().size(15.0));
    let counts = floor.count_room_types();
    ui.spacing_mut().item_spacing.y = 0.0;
    for room_type in RoomType::ALL {
        let (rect, _) = ui.allocate_exact_size(vec2(WIDTH, ROW_HEIGHT), Sense::hover());
        let painter = ui.painter_at(rect);
        let sample = Rect::from_min_size(Pos2::ZERO, vec2(SAMPLE_WIDTH, ROW_HEIGHT));
        paint_room(
            ui,
            &painter,
            &View::at(rect.min),
            sample,
            room_type,
            &[],
            palette,
        );
        let font = FontId::proportional(14.0);
        painter.text(
            pos2(rect.left() + SAMPLE_WIDTH + 12.0, rect.center().y),
            Align2::LEFT_CENTER,
            room_type.display_name(),
            font.clone(),
            palette.text,
        );
        painter.text(
            pos2(rect.right() - 4.0, rect.center().y),
            Align2::RIGHT_CENTER,
            counts.get(&room_type).copied().unwrap_or(0).to_string(),
            font,
            palette.muted_text,
        );
    }
}
