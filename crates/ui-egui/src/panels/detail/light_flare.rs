//! Light Flare on the photo (Place on Photo): a marker at every flare's light, the selected one
//! highlighted. A press on another flare's marker selects that flare and drags it; a press anywhere
//! else moves the selected flare's light there. One click or drag is one undo step.

use super::geometry::CanvasMap;
use super::{Color32, DevelopSettings, Gesture, LightcraftApp, Pos2, Rect, Stroke, json, register, vec2};
use lightcraft_develop::flare::range::POSITION;

/// How close (screen points) a press must be to a marker to grab that flare.
const GRAB: f32 = 14.0;

/// Where `q` (screen) is as a fraction of the cropped frame on screen, kept in the flare's range (a
/// light may sit outside the frame).
fn frame_fraction(map: &CanvasMap, q: Pos2) -> [f64; 2] {
    let r = map.rect;
    let f = |v: f32, lo: f32, len: f32| (f64::from((v - lo) / len.max(1.0))).clamp(f64::from(POSITION.0), f64::from(POSITION.1));
    [f(q.x, r.left(), r.width()), f(q.y, r.top(), r.height())]
}

/// Flare `i`'s light moves to `at` (both coordinates in the step the interaction makes).
fn place(app: &mut LightcraftApp, i: usize, at: [f64; 2]) {
    let _ = app.run("develop.set", json!({"control": format!("lightFlare.{i}.x"), "value": at[0]}));
    let _ = app.run("develop.set", json!({"control": format!("lightFlare.{i}.y"), "value": at[1]}));
}

/// The marker at a light: a ring with a cross and the flare's number (as in the section's list),
/// keylined so it shows over bright and dark photos; the selected flare's is larger and solid, the
/// others' smaller and fainter.
fn marker(p: &egui::Painter, c: Pos2, selected: bool, number: usize, font: egui::FontId) {
    let (ring, arm, white, black) = if selected { (9.0, 16.0, 240, 150) } else { (6.0, 11.0, 170, 106) };
    for (w, col) in [(3.5, Color32::from_black_alpha(black)), (1.5, Color32::from_white_alpha(white))] {
        let s = Stroke::new(w, col);
        p.circle_stroke(c, ring, s);
        let gap = ring * 0.55;
        for (a, b) in [
            (vec2(-arm, 0.0), vec2(-gap, 0.0)),
            (vec2(gap, 0.0), vec2(arm, 0.0)),
            (vec2(0.0, -arm), vec2(0.0, -gap)),
            (vec2(0.0, gap), vec2(0.0, arm)),
        ] {
            p.line_segment([c + a, c + b], s);
        }
    }
    let at = c + vec2(ring + 3.0, -ring - 1.0);
    for (d, col) in [(vec2(1.0, 1.0), Color32::from_black_alpha(black)), (vec2(0.0, 0.0), Color32::from_white_alpha(white))] {
        p.text(at + d, egui::Align2::LEFT_BOTTOM, number.to_string(), font.clone(), col);
    }
}

pub(super) fn light_flare_overlay(app: &mut LightcraftApp, ui: &mut egui::Ui, resp: &egui::Response, map: &CanvasMap, d: &DevelopSettings) {
    let r = map.rect;
    let lights: Vec<Pos2> =
        d.light_flare.flares.iter().map(|f| Pos2::new(r.left() + f.source.0 * r.width(), r.top() + f.source.1 * r.height())).collect();
    if lights.is_empty() {
        return;
    }
    let selected = app.ui.flare_index.min(lights.len() - 1);
    let p = ui.painter_at(app.canvas_rect.unwrap_or(r));
    let font = crate::theme::Tokens::get(ui.ctx()).semibold(12.0);
    // the selected one last, on top
    for (k, c) in lights.iter().enumerate().filter(|(k, _)| *k != selected).chain(std::iter::once((selected, &lights[selected]))) {
        marker(&p, *c, k == selected, k + 1, font.clone());
        register(ui, format!("lightFlare:light:{k}"), Rect::from_center_size(*c, vec2(18.0, 18.0)));
    }
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
    }
    // the flare a press at `q` acts on: the one whose marker it is on (the selected one first), else the selected one
    let target = |q: Pos2| -> usize {
        if lights[selected].distance(q) <= GRAB {
            return selected;
        }
        lights
            .iter()
            .enumerate()
            .filter(|(_, c)| c.distance(q) <= GRAB)
            .min_by(|a, b| a.1.distance(q).total_cmp(&b.1.distance(q)))
            .map_or(selected, |(k, _)| k)
    };
    if resp.drag_started()
        && let Some(q) = ui.input(|i| i.pointer.press_origin()).or(resp.interact_pointer_pos())
    {
        let k = target(q);
        app.ui.flare_index = k;
        let _ = app.run("develop.beginInteraction", json!({"label": "Light Flare Position"}));
        app.gesture = Some(Gesture::LightFlare);
        // grabbing a marker keeps the light where it is until the pointer moves; elsewhere it jumps there
        if lights[k].distance(q) > GRAB {
            place(app, k, frame_fraction(map, q));
        }
    } else if resp.clicked()
        && let Some(q) = resp.interact_pointer_pos()
    {
        let k = target(q);
        app.ui.flare_index = k;
        // a click on a marker only selects that flare
        if lights[k].distance(q) > GRAB {
            let _ = app.run("develop.beginInteraction", json!({"label": "Light Flare Position"}));
            place(app, k, frame_fraction(map, q));
            let _ = app.run("develop.endInteraction", json!({}));
        }
    }
    if resp.dragged()
        && matches!(app.gesture, Some(Gesture::LightFlare))
        && let Some(q) = resp.interact_pointer_pos()
    {
        place(app, app.ui.flare_index, frame_fraction(map, q));
    }
    if resp.drag_stopped() && matches!(app.gesture, Some(Gesture::LightFlare)) {
        app.gesture = None;
        let _ = app.run("develop.endInteraction", json!({}));
    }
}
