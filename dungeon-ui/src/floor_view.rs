//! Draws a floor with its rooms, doors and teleport links.

use std::collections::HashSet;

use dungeon_generator::room_description::describe_room;
use dungeon_generator::utils::door_face::DoorFace;
use dungeon_generator::{Floor, Room, RoomType};
use eframe::egui::{
    self, ImageSource, Painter, Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui, Vec2,
    include_image,
};

use crate::geometry::{
    CORNER_RADIUS, ICON_SIZE, View, canvas_size, cell_at, cell_rect, door_rect, room_edge,
    room_rect, rounded_rect_outline,
};
use crate::palette::Palette;

/// Icon of a room type. Normal rooms and dead ends have none,
/// the super secret room uses the icon of the secret room.
fn icon(room_type: RoomType) -> Option<ImageSource<'static>> {
    match room_type {
        RoomType::Start => Some(include_image!("../assets/room_icons/start_room.png")),
        RoomType::Item => Some(include_image!("../assets/room_icons/item_room.png")),
        RoomType::Shop => Some(include_image!("../assets/room_icons/shop_room.png")),
        RoomType::Boss => Some(include_image!("../assets/room_icons/boss_room.png")),
        RoomType::Secret | RoomType::SuperSecret => {
            Some(include_image!("../assets/room_icons/secret_room.png"))
        }
        RoomType::Normal | RoomType::DeadEnd | RoomType::Teleport => None,
    }
}

/// Draws a room with its doors in the cell, given in canvas coordinates.
pub fn paint_room(
    ui: &Ui,
    painter: &Painter,
    view: &View,
    cell: Rect,
    room_type: RoomType,
    doors: &[DoorFace],
    palette: &Palette,
) {
    for door in doors {
        painter.rect_filled(view.rect(door_rect(cell, *door)), 0.0, palette.door);
    }
    let room = view.rect(room_rect(cell));
    let radius = view.len(CORNER_RADIUS);
    painter.rect_filled(room, radius, palette.room(room_type));
    let outline = Stroke::new(view.len(2.0).max(1.0), palette.room_outline);
    if matches!(room_type, RoomType::Secret | RoomType::SuperSecret) {
        // Secret rooms are hidden, so their outline is dashed
        let mut points = rounded_rect_outline(room, radius);
        points.push(points[0]);
        painter.extend(Shape::dashed_line(
            &points,
            outline,
            view.len(4.0),
            view.len(3.0),
        ));
    } else {
        painter.rect_stroke(room, radius, outline, StrokeKind::Middle);
    }

    let center = room.center();
    if room_type == RoomType::Teleport {
        for (radius, width) in [(14.0, 3.0), (7.0, 2.0)] {
            painter.circle_stroke(
                center,
                view.len(radius),
                Stroke::new(view.len(width), palette.canvas),
            );
        }
    } else if let Some(source) = icon(room_type) {
        let size = Vec2::splat(view.len(ICON_SIZE));
        egui::Image::new(source).paint_at(ui, Rect::from_center_size(center, size));
    }
}

/// Draws a dashed arrow from every teleport room to its connected room.
fn paint_teleport_links(painter: &Painter, view: &View, floor: &Floor, palette: &Palette) {
    for room in floor.rooms() {
        let Some(target) = room
            .connected_room_id()
            .and_then(|id| floor.get_room_by_id(id))
        else {
            continue;
        };
        let start = cell_rect(room.position()).center();
        let end = cell_rect(target.position()).center();
        let from = view.pos(room_edge(start, end));
        let tip = view.pos(room_edge(end, start));
        let line = tip - from;
        if line.length() < 1.0 {
            continue;
        }
        let direction = line.normalized();
        let stroke = Stroke::new(view.len(2.0), palette.teleport_link);
        let neck = tip - direction * view.len(10.0);
        painter.extend(Shape::dashed_line(
            &[from, neck],
            stroke,
            view.len(6.0),
            view.len(4.0),
        ));
        let back = tip - direction * view.len(12.0);
        let side = direction.rot90() * view.len(5.0);
        painter.add(Shape::convex_polygon(
            vec![tip, back + side, back - side],
            palette.teleport_link,
            Stroke::NONE,
        ));
    }
}

/// Draws the floor scaled into the available space and shows the details of the
/// room below the pointer. With `visible`, only these rooms are drawn and the
/// teleport links are left out, which is used while the floor is revealed.
pub fn floor_view(
    ui: &mut Ui,
    floor: &Floor,
    visible: Option<&HashSet<u32>>,
    palette: &Palette,
) -> (Response, View) {
    let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::hover());
    let view = View::fit(canvas_size(floor.width(), floor.height()), response.rect);
    painter.rect_filled(response.rect, 0.0, palette.canvas);

    let grid = Stroke::new(1.0, palette.grid);
    for y in 0..floor.height() {
        for x in 0..floor.width() {
            let cell = view.rect(room_rect(cell_rect((x, y))));
            painter.rect_stroke(cell, 0.0, grid, StrokeKind::Inside);
        }
    }

    let is_visible = |room: &Room| visible.is_none_or(|ids| ids.contains(&room.id()));
    for room in floor.rooms().iter().filter(|room| is_visible(room)) {
        let cell = cell_rect(room.position());
        paint_room(
            ui,
            &painter,
            &view,
            cell,
            room.room_type(),
            room.doors(),
            palette,
        );
    }
    if visible.is_none() {
        paint_teleport_links(&painter, &view, floor, palette);
    }

    let hovered_room = response
        .hover_pos()
        .and_then(|pointer| cell_at(view.to_canvas(pointer), floor.width(), floor.height()))
        .and_then(|cell| {
            floor
                .rooms()
                .iter()
                .find(|room| room.position() == cell && is_visible(room))
        });
    let response = match hovered_room {
        Some(room) => response.on_hover_text_at_pointer(describe_room(floor, room)),
        None => response,
    };
    (response, view)
}
