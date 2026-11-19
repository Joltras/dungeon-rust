//! Positions and sizes of the drawn floor, independent of egui's painting.
//!
//! Everything is computed in canvas coordinates, where a cell is
//! `ROOM_WIDTH × ROOM_HEIGHT` like in the Python UI. [`View`] maps them to the screen.

use dungeon_generator::utils::Position;
use dungeon_generator::utils::constants::{ROOM_HEIGHT, ROOM_WIDTH};
use dungeon_generator::utils::door_face::DoorFace;
use eframe::egui::{Pos2, Rect, Vec2, pos2, vec2};

/// Size of a cell of the floor
pub const CELL: Vec2 = vec2(ROOM_WIDTH as f32, ROOM_HEIGHT as f32);
/// Space between the border of a cell and the room drawn in it
pub const ROOM_PADDING: Vec2 = vec2(10.0, 8.0);
/// Radius of the rounded room corners
pub const CORNER_RADIUS: f32 = 8.0;
/// Width of the door connectors between two rooms
pub const DOOR_WIDTH: f32 = 14.0;
pub const ICON_SIZE: f32 = 32.0;

/// Size of the canvas for a floor with the given number of cells.
pub fn canvas_size(width: i32, height: i32) -> Vec2 {
    vec2(width as f32 * CELL.x, height as f32 * CELL.y)
}

/// Rectangle of the cell at the position.
pub fn cell_rect((x, y): Position) -> Rect {
    Rect::from_min_size(pos2(x as f32 * CELL.x, y as f32 * CELL.y), CELL)
}

/// Rectangle of the room drawn in the cell.
pub fn room_rect(cell: Rect) -> Rect {
    cell.shrink2(ROOM_PADDING)
}

/// Connector from the room to the border of its cell for a door.
/// It overlaps the room by 2 so there is no gap at the rounded outline.
pub fn door_rect(cell: Rect, door: DoorFace) -> Rect {
    let room = room_rect(cell);
    let center = cell.center();
    let half = DOOR_WIDTH / 2.0;
    let (min, max) = match door {
        DoorFace::Top => (
            pos2(center.x - half, cell.top()),
            pos2(center.x + half, room.top() + 2.0),
        ),
        DoorFace::Bottom => (
            pos2(center.x - half, room.bottom() - 2.0),
            pos2(center.x + half, cell.bottom()),
        ),
        DoorFace::West => (
            pos2(cell.left(), center.y - half),
            pos2(room.left() + 2.0, center.y + half),
        ),
        DoorFace::East => (
            pos2(room.right() - 2.0, center.y - half),
            pos2(cell.right(), center.y + half),
        ),
    };
    Rect::from_min_max(min, max)
}

/// Point where a line from the center of a room towards another point leaves the room.
pub fn room_edge(center: Pos2, towards: Pos2) -> Pos2 {
    let delta = towards - center;
    let half = CELL / 2.0 - ROOM_PADDING;
    let scale_x = if delta.x != 0.0 {
        half.x / delta.x.abs()
    } else {
        f32::INFINITY
    };
    let scale_y = if delta.y != 0.0 {
        half.y / delta.y.abs()
    } else {
        f32::INFINITY
    };
    let scale = scale_x.min(scale_y);
    if scale.is_finite() {
        center + delta * scale
    } else {
        center
    }
}

/// Cell at a canvas position, `None` outside of the floor.
pub fn cell_at(position: Pos2, width: i32, height: i32) -> Option<Position> {
    if position.x < 0.0 || position.y < 0.0 {
        return None;
    }
    let x = (position.x / CELL.x) as i32;
    let y = (position.y / CELL.y) as i32;
    (x < width && y < height).then_some((x, y))
}

/// Points of the outline of a rectangle with rounded corners, without repeating the first.
pub fn rounded_rect_outline(rect: Rect, radius: f32) -> Vec<Pos2> {
    const SEGMENTS: usize = 6;
    let radius = radius.min(rect.width() / 2.0).min(rect.height() / 2.0);
    // Corner centers with the start angle of their arc, clockwise from the top right
    let corners = [
        (pos2(rect.right() - radius, rect.top() + radius), -90.0f32),
        (pos2(rect.right() - radius, rect.bottom() - radius), 0.0),
        (pos2(rect.left() + radius, rect.bottom() - radius), 90.0),
        (pos2(rect.left() + radius, rect.top() + radius), 180.0),
    ];
    let mut points = Vec::with_capacity(corners.len() * (SEGMENTS + 1));
    for (center, start) in corners {
        for step in 0..=SEGMENTS {
            let angle = (start + 90.0 * step as f32 / SEGMENTS as f32).to_radians();
            points.push(center + vec2(angle.cos(), angle.sin()) * radius);
        }
    }
    points
}

/// Maps canvas coordinates to the screen, scaled to fit while keeping the aspect ratio.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    pub origin: Pos2,
    pub scale: f32,
}

impl View {
    /// Fits a canvas of the given size centered into the available rectangle.
    pub fn fit(canvas: Vec2, available: Rect) -> Self {
        let scale = (available.width() / canvas.x)
            .min(available.height() / canvas.y)
            .max(0.0);
        Self {
            origin: available.center() - canvas * scale / 2.0,
            scale,
        }
    }

    /// View that draws the canvas unscaled at the origin.
    pub fn at(origin: Pos2) -> Self {
        Self { origin, scale: 1.0 }
    }

    pub fn pos(self, position: Pos2) -> Pos2 {
        self.origin + position.to_vec2() * self.scale
    }

    pub fn rect(self, rect: Rect) -> Rect {
        Rect::from_min_max(self.pos(rect.min), self.pos(rect.max))
    }

    pub fn len(self, length: f32) -> f32 {
        length * self.scale
    }

    /// Canvas position of a screen position.
    pub fn to_canvas(self, position: Pos2) -> Pos2 {
        ((position - self.origin) / self.scale).to_pos2()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rooms_and_doors_are_placed_like_in_python() {
        let cell = cell_rect((1, 2));
        assert_eq!(
            cell,
            Rect::from_min_max(pos2(120.0, 120.0), pos2(240.0, 180.0))
        );
        assert_eq!(
            room_rect(cell),
            Rect::from_min_max(pos2(130.0, 128.0), pos2(230.0, 172.0))
        );
        assert_eq!(
            door_rect(cell, DoorFace::Top),
            Rect::from_min_max(pos2(173.0, 120.0), pos2(187.0, 130.0))
        );
        assert_eq!(
            door_rect(cell, DoorFace::East),
            Rect::from_min_max(pos2(228.0, 143.0), pos2(240.0, 157.0))
        );
    }

    #[test]
    fn room_edge_is_on_the_room_border() {
        let center = pos2(60.0, 30.0);
        assert_eq!(room_edge(center, pos2(600.0, 30.0)), pos2(110.0, 30.0));
        assert_eq!(room_edge(center, pos2(60.0, 0.0)), pos2(60.0, 8.0));
        assert_eq!(room_edge(center, center), center);
    }

    #[test]
    fn finds_the_cell_below_a_position() {
        assert_eq!(cell_at(pos2(0.0, 0.0), 9, 8), Some((0, 0)));
        assert_eq!(cell_at(pos2(1079.0, 479.0), 9, 8), Some((8, 7)));
        assert_eq!(cell_at(pos2(1080.0, 10.0), 9, 8), None);
        assert_eq!(cell_at(pos2(-1.0, 10.0), 9, 8), None);
    }

    #[test]
    fn view_fits_and_maps_back() {
        let available = Rect::from_min_size(pos2(10.0, 20.0), vec2(540.0, 400.0));
        let view = View::fit(canvas_size(9, 8), available);
        assert_eq!(view.scale, 0.5);
        assert_eq!(view.pos(pos2(0.0, 0.0)), pos2(10.0, 100.0));
        assert_eq!(
            view.to_canvas(view.pos(pos2(300.0, 70.0))),
            pos2(300.0, 70.0)
        );
    }

    #[test]
    fn rounded_outline_stays_in_the_rectangle() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 44.0));
        let points = rounded_rect_outline(rect, 8.0);
        assert_eq!(points.len(), 28);
        assert!(points.iter().all(|p| rect.expand(0.001).contains(*p)));
    }
}
