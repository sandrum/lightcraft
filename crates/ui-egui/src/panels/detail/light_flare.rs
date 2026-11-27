//! Light Flare on the photo (Place on Photo): a marker at every flare's light, the selected one
//! highlighted, and the selected flare's centre point as a diamond with the axis its ghosts line up
//! on. A press on another flare's marker selects that flare and drags it; a press on the diamond, or
//! anywhere with Alt held, moves the centre point; a press anywhere else moves the selected flare's
//! light there. One click or drag is one undo step.

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

/// Flare `i`'s light (or with `center` its centre point) moves to `at` (both coordinates in the
/// step the interaction makes).
fn place(app: &mut LightcraftApp, i: usize, at: [f64; 2], center: bool) {
    let (x, y) = if center { ("centerX", "centerY") } else { ("x", "y") };
    let _ = app.run("develop.set", json!({"control": format!("lightFlare.{i}.{x}"), "value": at[0]}));
    let _ = app.run("develop.set", json!({"control": format!("lightFlare.{i}.{y}"), "value": at[1]}));
}

/// The centre point's marker: a keylined diamond, and the axis from the light through it.
fn center_marker(p: &egui::Painter, light: Pos2, c: Pos2) {
    let mirror = c + (c - light);
    p.line_segment([light, mirror], Stroke::new(1.0, Color32::from_white_alpha(90)));
    let d = 7.0;
    let pts = vec![c + vec2(0.0, -d), c + vec2(d, 0.0), c + vec2(0.0, d), c + vec2(-d, 0.0)];
    p.add(egui::Shape::closed_line(pts.clone(), Stroke::new(3.5, Color32::from_black_alpha(150))));
    p.add(egui::Shape::closed_line(pts, Stroke::new(1.5, Color32::from_white_alpha(240))));
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
    let center =
        d.light_flare.flares.get(selected).map_or(r.center(), |f| Pos2::new(r.left() + f.center.0 * r.width(), r.top() + f.center.1 * r.height()));
    center_marker(&p, lights[selected], center);
    register(ui, "lightFlare:center", Rect::from_center_size(center, vec2(18.0, 18.0)));
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
    // the centre point takes a press on its diamond (unless it is on a light marker), or any with Alt
    let alt = ui.input(|i| i.modifiers.alt);
    let on_center = |q: Pos2| alt || (center.distance(q) <= GRAB && lights.iter().all(|l| l.distance(q) > GRAB));
    if resp.drag_started()
        && let Some(q) = ui.input(|i| i.pointer.press_origin()).or(resp.interact_pointer_pos())
    {
        if on_center(q) {
            let _ = app.run("develop.beginInteraction", json!({"label": "Light Flare Center"}));
            app.gesture = Some(Gesture::LightFlareCenter);
            if alt {
                place(app, selected, frame_fraction(map, q), true);
            }
        } else {
            let k = target(q);
            app.ui.flare_index = k;
            let _ = app.run("develop.beginInteraction", json!({"label": "Light Flare Position"}));
            app.gesture = Some(Gesture::LightFlare);
            // grabbing a marker keeps the light where it is until the pointer moves; elsewhere it jumps there
            if lights[k].distance(q) > GRAB {
                place(app, k, frame_fraction(map, q), false);
            }
        }
    } else if resp.clicked()
        && let Some(q) = resp.interact_pointer_pos()
    {
        if on_center(q) {
            // Alt-click puts the centre point there; a click on the diamond leaves it be
            if alt {
                let _ = app.run("develop.beginInteraction", json!({"label": "Light Flare Center"}));
                place(app, selected, frame_fraction(map, q), true);
                let _ = app.run("develop.endInteraction", json!({}));
            }
        } else {
            let k = target(q);
            app.ui.flare_index = k;
            // a click on a marker only selects that flare
            if lights[k].distance(q) > GRAB {
                let _ = app.run("develop.beginInteraction", json!({"label": "Light Flare Position"}));
                place(app, k, frame_fraction(map, q), false);
                let _ = app.run("develop.endInteraction", json!({}));
            }
        }
    }
    let dragging_center = matches!(app.gesture, Some(Gesture::LightFlareCenter));
    if resp.dragged()
        && (dragging_center || matches!(app.gesture, Some(Gesture::LightFlare)))
        && let Some(q) = resp.interact_pointer_pos()
    {
        place(app, app.ui.flare_index, frame_fraction(map, q), dragging_center);
    }
    if resp.drag_stopped() && matches!(app.gesture, Some(Gesture::LightFlare | Gesture::LightFlareCenter)) {
        app.gesture = None;
        let _ = app.run("develop.endInteraction", json!({}));
    }
}
