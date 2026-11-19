//! Colors of the light and dark theme, the same as in the Python UI.

use dungeon_generator::RoomType;
use eframe::egui::{Color32, Visuals};

/// Colors of one theme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub window: Color32,
    pub canvas: Color32,
    pub grid: Color32,
    pub room_outline: Color32,
    pub door: Color32,
    pub teleport_link: Color32,
    pub text: Color32,
    pub muted_text: Color32,
    pub entry: Color32,
    /// Room colors in the order of [`RoomType::ALL`]
    rooms: [Color32; 9],
}

const fn hex(value: u32) -> Color32 {
    Color32::from_rgb((value >> 16) as u8, (value >> 8) as u8, value as u8)
}

pub const LIGHT: Palette = Palette {
    window: hex(0xe5e9f0),
    canvas: hex(0xf4f6f9),
    grid: hex(0xdde2ea),
    room_outline: hex(0x4c566a),
    door: hex(0x4c566a),
    teleport_link: hex(0x8e6bb8),
    text: hex(0x2e3440),
    muted_text: hex(0x5e6779),
    entry: hex(0xffffff),
    rooms: [
        hex(0xcdd5e1),
        hex(0xb7c1d1),
        hex(0x92c47f),
        hex(0xebcb6e),
        hex(0xee9d5e),
        hex(0xb28fd8),
        hex(0xd9665f),
        hex(0x7393cf),
        hex(0x5fb3b0),
    ],
};

pub const DARK: Palette = Palette {
    window: hex(0x1b1e24),
    canvas: hex(0x232730),
    grid: hex(0x2c313c),
    room_outline: hex(0x11131a),
    door: hex(0x8d97a8),
    teleport_link: hex(0xb28fd8),
    text: hex(0xe5e9f0),
    muted_text: hex(0xa3abb9),
    entry: hex(0x2e3440),
    rooms: [
        hex(0x4c5669),
        hex(0x3e4757),
        hex(0x5f9a55),
        hex(0xc4a446),
        hex(0xcf7a3d),
        hex(0x8a64b5),
        hex(0xb84d47),
        hex(0x4f6fae),
        hex(0x3f8f8b),
    ],
};

impl Palette {
    pub fn get(dark: bool) -> &'static Palette {
        if dark { &DARK } else { &LIGHT }
    }

    pub fn room(&self, room_type: RoomType) -> Color32 {
        let index = RoomType::ALL
            .iter()
            .position(|other| *other == room_type)
            .expect("every room type is in RoomType::ALL");
        self.rooms[index]
    }

    /// egui widget colors that match the palette.
    pub fn visuals(&self, dark: bool) -> Visuals {
        let mut visuals = if dark {
            Visuals::dark()
        } else {
            Visuals::light()
        };
        visuals.panel_fill = self.window;
        visuals.window_fill = self.window;
        visuals.extreme_bg_color = self.entry;
        visuals.override_text_color = Some(self.text);
        visuals
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_the_python_colors() {
        assert_eq!(
            LIGHT.room(RoomType::Boss),
            Color32::from_rgb(0xd9, 0x66, 0x5f)
        );
        assert_eq!(
            DARK.room(RoomType::SuperSecret),
            Color32::from_rgb(0x3f, 0x8f, 0x8b)
        );
        assert_eq!(Palette::get(true), &DARK);
    }
}
