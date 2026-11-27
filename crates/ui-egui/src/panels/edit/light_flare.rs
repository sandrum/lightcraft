//! The Light Flare section: Apply and Place on Photo; the photo's flares as a list (one row each:
//! its number, its preset, an eye that hides it; Add Flare and Delete under it); then the selected
//! flare's preset and main sliders, and its Light, Shape, Color, Texture and Parts groups, folded
//! until opened. Parts lists the flare's own parts (glows, streaks, ghosts…) to show, hide, add,
//! copy, delete, reorder and tune one by one, and the centre point its ghosts line up through.

use super::{LightcraftApp, Tokens, control, register, text_button};
use crate::icons::{Icon, paint};
use crate::widgets::{announce_disclosure, announce_selected, icon_button, one_line};
use egui::{Align2, Rect, Sense, Stroke, pos2, vec2};
use lightcraft_develop::DevelopSettings;
use lightcraft_develop::flare::{FlareCategory, LensFlare, MAX_FLARES, PART_KINDS, TextureKind, presets, range};
use lightcraft_develop::{ControlSpec, Section, Track};
use serde_json::json;

/// The tool that places the light with a click or drag on the photo (`panels/detail/light_flare.rs`).
pub(crate) const PLACE_TOOL: &str = "lightFlare";

/// The focus id of flare `k`'s row in the list.
fn row_id(k: usize) -> egui::Id {
    egui::Id::unique(("lightFlareRow", k))
}

pub(super) fn light_flare(app: &mut LightcraftApp, ui: &mut egui::Ui, d: &DevelopSettings) {
    let lf = &d.light_flare;
    let n = lf.flares.len();
    // the selection stays on a flare the photo has (another photo may have fewer)
    if n > 0 && app.ui.flare_index >= n {
        app.ui.flare_index = n - 1;
    }
    // the light can't be placed while the flares are off or there are none
    if app.ui.tool == PLACE_TOOL && (!lf.enabled || n == 0) {
        app.ui.tool.clear();
    }
    apply_row(app, ui, lf.enabled, n);
    flare_list(app, ui, &lf.flares, lf.enabled);
    let i = app.ui.flare_index;
    let Some(f) = lf.flares.get(i) else { return };
    let on = lf.enabled;
    preset_row(app, ui, i, f, on);
    for c in ["brightness", "scale", "source"] {
        control(app, ui, d, &format!("lightFlare.{i}.{c}"), on);
    }
    if group(app, ui, "light", "Light", "") {
        for c in ["spread", "sourceBrightness"] {
            control(app, ui, d, &format!("lightFlare.{i}.{c}"), on);
        }
    }
    if group(app, ui, "shape", "Shape", "") {
        for c in ["artifacts", "aspect", "rotation", "softness", "fringe"] {
            control(app, ui, d, &format!("lightFlare.{i}.{c}"), on);
        }
        flips(app, ui, i, on);
    }
    let colour = if f.tint > 0.0 { crate::i18n::tr_format!("Hue {h}°", h = f.hue.round()) } else { crate::i18n::tr("Preset colours").to_string() };
    if group(app, ui, "color", "Color", &colour) {
        control(app, ui, d, &format!("lightFlare.{i}.saturation"), on);
        control(app, ui, d, &format!("lightFlare.{i}.hue"), on);
        control(app, ui, d, &format!("lightFlare.{i}.tint"), on);
    }
    if group(app, ui, "texture", "Texture", crate::i18n::tr(f.texture.kind.label())) {
        texture(app, ui, d, i, f, on);
    }
    let summary = crate::i18n::tr_format!("{n} parts", n = f.elements.len());
    if group(app, ui, "parts", "Parts", &summary) {
        parts(app, ui, i, f, on);
    }
    ui.add_space(6.0);
}

/// Apply, and Place on Photo on the right.
fn apply_row(app: &mut LightcraftApp, ui: &mut egui::Ui, enabled: bool, n: usize) {
    egui::Frame::NONE.inner_margin(egui::Margin { left: 24, right: 22, top: 4, bottom: 4 }).show(ui, |ui| {
        ui.horizontal(|ui| {
            let mut on = enabled;
            let r = ui.checkbox(&mut on, crate::i18n::tr("Apply"));
            register(ui, "check:lightFlare.enabled", r.rect);
            if r.on_hover_text(crate::i18n::tr("Add lens flares to the photo")).changed() {
                let _ = app.run("develop.merge", json!({"settings": {"light_flare": {"enabled": on}}, "label": "Light Flare"}));
                // switched on: the next click on the photo places the light
                if on && n > 0 {
                    app.ui.tool = PLACE_TOOL.to_string();
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_enabled_ui(enabled && n > 0, |ui| {
                    let placing = app.ui.tool == PLACE_TOOL;
                    let r = text_button(ui, "lightFlarePlace", "Place on Photo", placing)
                        .on_hover_text(crate::i18n::tr("Click or drag on the photo to move the selected flare's light"));
                    if r.clicked() {
                        if placing {
                            app.ui.tool.clear();
                        } else {
                            app.ui.tool = PLACE_TOOL.to_string();
                        }
                    }
                });
            });
        });
    });
}

/// The photo's flares, one row each (the selected one highlighted), then Add Flare and Delete. A
/// click on a row selects that flare; ↑ / ↓ move the selection (and the focus) along the list.
fn flare_list(app: &mut LightcraftApp, ui: &mut egui::Ui, flares: &[LensFlare], enabled: bool) {
    let t = Tokens::get(ui.ctx());
    let n = flares.len();
    let sel = app.ui.flare_index;
    egui::Frame::NONE.inner_margin(egui::Margin { left: 24, right: 22, top: 2, bottom: 4 }).show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 2.0;
        for (k, f) in flares.iter().enumerate() {
            let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 28.0), Sense::hover());
            let resp = ui.interact(r, row_id(k), Sense::click());
            let name = crate::i18n::tr_format!("Flare {n}: {preset}", n = k + 1, preset = f.preset);
            announce_selected(&resp, name, k == sel);
            register(ui, format!("lightFlareRow:{k}"), r);
            let p = ui.painter();
            let fill = if k == sel {
                t.tool_active
            } else if resp.hovered() {
                t.hover.gamma_multiply(0.7)
            } else {
                t.inset
            };
            p.rect_filled(r, 4.0, fill);
            if k == sel {
                // not by its fill alone: a bar down the selected row's edge
                p.rect_filled(Rect::from_min_size(r.min, vec2(3.0, r.height())), 1.5, t.accent);
            }
            if resp.has_focus() {
                p.rect_stroke(r, 4.0, Stroke::new(1.5, t.accent), egui::StrokeKind::Inside);
            }
            // the number in a ring, as on the photo's marker, then the preset
            let shown = !f.hidden;
            let ink = if !shown || !enabled { t.text_disabled } else { t.text };
            let badge = pos2(r.left() + 16.0, r.center().y);
            p.circle_stroke(badge, 8.5, Stroke::new(1.2, if k == sel { t.text } else { t.text_dim }));
            p.text(badge, Align2::CENTER_CENTER, (k + 1).to_string(), t.semibold(11.0), ink);
            let g = one_line(p, &f.preset, t.font(13.0), ink, r.width() - 34.0 - 30.0);
            p.galley(pos2(r.left() + 34.0, r.center().y - g.size().y / 2.0), g, ink);
            // show / hide: on hover, and always while hidden
            let er = Rect::from_center_size(pos2(r.right() - 15.0, r.center().y), vec2(22.0, 22.0));
            if ui.rect_contains_pointer(r) || !shown || resp.has_focus() {
                let eye = ui.interact(er, egui::Id::unique(("lightFlareVisible", k)), Sense::click());
                announce_selected(&eye, crate::i18n::tr_format!("Show flare {n}", n = k + 1), shown);
                register(ui, format!("lightFlareVisible:{k}"), er);
                let eye = eye.on_hover_text(crate::i18n::tr(if shown { "Hide this flare" } else { "Show this flare" }));
                paint(ui.painter(), er.shrink(3.0), if shown { Icon::Eye } else { Icon::EyeOff }, if eye.hovered() { t.text } else { t.text_dim });
                if eye.clicked() {
                    let _ = app.run("lightFlare.visible", json!({"index": k}));
                }
            }
            if resp.clicked() {
                app.ui.flare_index = k;
                // so ↑ / ↓ go on from the row clicked
                ui.memory_mut(|m| m.request_focus(row_id(k)));
            } else if resp.gained_focus() {
                app.ui.flare_index = k;
            }
            if resp.has_focus() {
                let (up, down) = ui.input(|i| (i.key_pressed(egui::Key::ArrowUp), i.key_pressed(egui::Key::ArrowDown)));
                let to = if up && k > 0 {
                    Some(k - 1)
                } else if down && k + 1 < n {
                    Some(k + 1)
                } else {
                    None
                };
                // once a frame: a frame laid out twice sees the same key press again
                let frame = ui.ctx().cumulative_frame_nr();
                let moved = egui::Id::unique("lightFlareRowMoved");
                if let Some(to) = to
                    && ui.data(|m| m.get_temp::<u64>(moved)) != Some(frame)
                {
                    ui.data_mut(|m| m.insert_temp(moved, frame));
                    app.ui.flare_index = to;
                    ui.memory_mut(|m| m.request_focus(row_id(to)));
                }
            }
            crate::widgets::context_menu(&resp, |ui| {
                if ui.button(crate::i18n::tr(if shown { "Hide" } else { "Show" })).clicked() {
                    let _ = app.run("lightFlare.visible", json!({"index": k}));
                    ui.close();
                }
                if ui.button(crate::i18n::tr("Delete")).clicked() {
                    remove(app, k);
                    ui.close();
                }
            });
        }
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let r = ui.add_enabled_ui(n < MAX_FLARES, |ui| text_button(ui, "lightFlareAdd", "Add Flare", false)).inner;
            if r.on_hover_text(crate::i18n::tr("Add another flare (up to 8)")).clicked()
                && let Ok(v) = app.run("lightFlare.add", json!({}))
                && let Some(k) = v.get("index").and_then(serde_json::Value::as_u64)
            {
                app.ui.flare_index = k as usize;
                app.ui.tool = PLACE_TOOL.to_string();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if icon_button(ui, "lightFlareRemove", Icon::Trash, vec2(26.0, 24.0), false, n > 0, "Delete the selected flare").clicked() {
                    remove(app, sel);
                }
                ui.label(egui::RichText::new(crate::i18n::tr_format!("{n} of {max}", n = n, max = MAX_FLARES)).font(t.font(12.0)).color(t.text_dim));
            });
        });
    });
}

/// Flare `k` deleted; the selection moves to the flare before it.
fn remove(app: &mut LightcraftApp, k: usize) {
    if app.run("lightFlare.remove", json!({"index": k})).is_ok() && app.ui.flare_index >= k {
        app.ui.flare_index = app.ui.flare_index.saturating_sub(1);
    }
}

/// A group of the selected flare's controls, folded until its title is clicked (the choice is
/// kept): a chevron, the title and, on the right, what the group is set to. Whether it is open.
fn group(app: &mut LightcraftApp, ui: &mut egui::Ui, key: &str, title: &str, summary: &str) -> bool {
    let t = Tokens::get(ui.ctx());
    let mut open = app.ui.flare_groups_open.iter().any(|g| g == key);
    let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 30.0), Sense::click());
    let title = crate::i18n::tr(title);
    announce_disclosure(&resp, true, open, title);
    register(ui, format!("lightFlareGroup:{key}"), r);
    if resp.clicked() {
        open = !open;
        if open {
            app.ui.flare_groups_open.push(key.to_string());
        } else {
            app.ui.flare_groups_open.retain(|g| g != key);
        }
    }
    let p = ui.painter();
    if ui.rect_contains_pointer(r) {
        p.rect_filled(r, 0.0, t.hover.gamma_multiply(0.4));
    }
    if resp.has_focus() {
        p.rect_stroke(r.shrink(1.0), 3.0, Stroke::new(1.5, t.accent), egui::StrokeKind::Inside);
    }
    paint(
        p,
        Rect::from_center_size(pos2(r.left() + 30.0, r.center().y), vec2(13.0, 13.0)),
        if open { Icon::ChevronDown } else { Icon::ChevronRight },
        t.text_label,
    );
    p.text(pos2(r.left() + 42.0, r.center().y), Align2::LEFT_CENTER, title, t.semibold(13.0), t.text_label);
    if !summary.is_empty() {
        p.text(pos2(r.right() - 22.0, r.center().y), Align2::RIGHT_CENTER, summary, t.font(12.0), t.text_dim);
    }
    open
}

/// The selected flare's preset: a menu of the library, one submenu per kind of lens.
fn preset_row(app: &mut LightcraftApp, ui: &mut egui::Ui, i: usize, f: &LensFlare, on: bool) {
    let t = Tokens::get(ui.ctx());
    egui::Frame::NONE.inner_margin(egui::Margin { left: 24, right: 22, top: 4, bottom: 6 }).show(ui, |ui| {
        ui.add_enabled_ui(on, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(crate::i18n::tr("Preset")).font(t.font(13.0)).color(t.text_dim));
                let max = ui.available_width();
                let r = crate::widgets::dropdown_within(ui, "lightFlarePreset", &f.preset, t.font(14.0), t.text_label, max);
                let below = r.rect.bottom();
                egui::Popup::menu(&r).show(|ui| {
                    crate::menu_level::level(ui, 1, Some(below), |ui| preset_menu(app, ui, i, &f.preset, below));
                });
            });
        });
    });
}

/// The lens dirt or film the selected flare's light shows, its flips and its opacity.
fn texture(app: &mut LightcraftApp, ui: &mut egui::Ui, d: &DevelopSettings, i: usize, f: &LensFlare, on: bool) {
    let t = Tokens::get(ui.ctx());
    let tex = f.texture;
    egui::Frame::NONE.inner_margin(egui::Margin { left: 24, right: 22, top: 0, bottom: 4 }).show(ui, |ui| {
        ui.add_enabled_ui(on, |ui| {
            ui.horizontal(|ui| {
                let max = ui.available_width();
                let r = crate::widgets::dropdown_within(ui, "lightFlareTexture", crate::i18n::tr(tex.kind.label()), t.font(14.0), t.text_label, max);
                egui::Popup::menu(&r).show(|ui| {
                    ui.set_min_width(170.0);
                    for k in TextureKind::ALL {
                        let item = ui.selectable_label(k == tex.kind, crate::i18n::tr(k.label()));
                        register(ui, format!("lightFlareTexture:{}", k.id()), item.rect);
                        if item.clicked() {
                            let _ = app.run("lightFlare.texture", json!({"index": i, "texture": k.id()}));
                            ui.close();
                        }
                    }
                });
            });
            ui.add_enabled_ui(tex.kind != TextureKind::None, |ui| {
                ui.horizontal(|ui| {
                    for (key, label, v0) in [("flipX", "Flip Horizontal", tex.flip_x), ("flipY", "Flip Vertical", tex.flip_y)] {
                        let mut v = v0;
                        let r = ui.checkbox(&mut v, crate::i18n::tr(label));
                        register(ui, format!("check:lightFlare.{key}"), r.rect);
                        if r.changed() {
                            let _ = app.run("lightFlare.texture", json!({"index": i, key: v}));
                        }
                    }
                });
            });
        });
    });
    control(app, ui, d, &format!("lightFlare.{i}.textureOpacity"), on && tex.kind != TextureKind::None);
}

fn preset_menu(app: &mut LightcraftApp, ui: &mut egui::Ui, index: usize, current: &str, below: f32) {
    let mut pick: Option<&'static str> = None;
    for c in FlareCategory::ALL {
        let r = crate::widgets::menu_button(ui, crate::i18n::tr(c.label()), |ui| {
            crate::menu_level::level(ui, 2, Some(below), |ui| {
                ui.set_min_width(170.0);
                for p in presets(c) {
                    let item = ui.selectable_label(p.name == current, p.name);
                    register(ui, format!("lightFlarePreset:{}", p.name), item.rect);
                    if item.clicked() {
                        pick = Some(p.name);
                    }
                }
            });
        });
        register(ui, format!("lightFlarePreset:group:{}", c.id()), r.response.rect);
        if r.inner.is_some() {
            crate::menu_level::set_anchor(ui.ctx(), 2, r.response.rect);
        }
    }
    if let Some(name) = pick {
        let _ = app.run("lightFlare.preset", json!({"preset": name, "index": index}));
        ui.close();
    }
}

/// Flip Horizontal and Flip Vertical: the whole flare mirrored (`lightFlare.flip`).
fn flips(app: &mut LightcraftApp, ui: &mut egui::Ui, i: usize, on: bool) {
    egui::Frame::NONE.inner_margin(egui::Margin { left: 24, right: 22, top: 2, bottom: 6 }).show(ui, |ui| {
        ui.add_enabled_ui(on, |ui| {
            ui.horizontal(|ui| {
                for (id, label, axis) in [("lightFlareFlipH", "Flip Horizontal", "horizontal"), ("lightFlareFlipV", "Flip Vertical", "vertical")] {
                    let r = text_button(ui, id, label, false)
                        .on_hover_text(crate::i18n::tr("Mirror the whole flare: its light, center point and angles"));
                    if r.clicked() {
                        let _ = app.run("lightFlare.flip", json!({"index": i, "axis": axis}));
                    }
                }
            });
        });
    });
}

/// A part's label in the list: its number and kind, and for a part off the light where it sits on
/// the axis (0 = the light, 1 = the centre point).
fn part_label(k: usize, e: &lightcraft_develop::flare::FlareElement) -> String {
    let id = e.kind_id();
    let name = crate::i18n::tr(PART_KINDS.iter().find(|(k, _)| *k == id).map_or(id, |(_, l)| *l));
    let t = serde_json::to_value(e).ok().and_then(|v| v.get("t").and_then(serde_json::Value::as_f64)).unwrap_or(0.0);
    if matches!(id, "ring" | "ghost") && t.abs() > 1e-3 {
        crate::i18n::tr_format!("{n}. {name} at {t}", n = k + 1, name = name, t = format!("{t:.2}"))
    } else {
        format!("{}. {name}", k + 1)
    }
}

const fn part_spec(id: &'static str, label: &'static str, min: f64, max: f64, step: f64, decimals: u8) -> ControlSpec {
    ControlSpec { id, label, section: Section::LightFlare, min, max, default: min, step, decimals, track: Track::Plain }
}

/// The settings of a part of `kind`: the field each sets, and its slider. Sizes are thousandths of the
/// frame's diagonal before the flare's Scale; Position is along the axis.
fn part_settings(kind: &str) -> &'static [(&'static str, ControlSpec)] {
    const B: ControlSpec = part_spec("lightFlarePart.intensity", "Brightness", 0.0, 2.0, 0.01, 2);
    const T: ControlSpec = part_spec("lightFlarePart.t", "Position", -1.0, 3.0, 0.01, 2);
    const C: ControlSpec = part_spec("lightFlarePart.chroma", "Rainbow Edge", 0.0, 2.0, 0.01, 2);
    const A: ControlSpec = part_spec("lightFlarePart.angle", "Angle", -180.0, 180.0, 1.0, 0);
    const GLOW: &[(&str, ControlSpec)] = &[
        ("intensity", B),
        ("radius", part_spec("lightFlarePart.glow.radius", "Size", 1.0, 400.0, 1.0, 0)),
        ("falloff", part_spec("lightFlarePart.falloff", "Falloff", 1.0, 2.0, 0.01, 2)),
    ];
    const STREAK: &[(&str, ControlSpec)] = &[
        ("intensity", B),
        ("length", part_spec("lightFlarePart.streak.length", "Length", 10.0, 2500.0, 1.0, 0)),
        ("thickness", part_spec("lightFlarePart.streak.thickness", "Thickness", 0.2, 60.0, 0.1, 1)),
        ("angle", A),
    ];
    const STAR: &[(&str, ControlSpec)] = &[
        ("intensity", B),
        ("rays", part_spec("lightFlarePart.rays", "Points", 2.0, 32.0, 1.0, 0)),
        ("length", part_spec("lightFlarePart.star.length", "Length", 10.0, 600.0, 1.0, 0)),
        ("thickness", part_spec("lightFlarePart.star.thickness", "Thickness", 0.2, 10.0, 0.1, 1)),
        ("angle", A),
    ];
    const RAYS: &[(&str, ControlSpec)] = &[
        ("intensity", B),
        ("count", part_spec("lightFlarePart.count", "Count", 1.0, 256.0, 1.0, 0)),
        ("length", part_spec("lightFlarePart.rays.length", "Length", 10.0, 600.0, 1.0, 0)),
        ("seed", part_spec("lightFlarePart.seed", "Variation", 0.0, 999.0, 1.0, 0)),
    ];
    const RING: &[(&str, ControlSpec)] = &[
        ("intensity", B),
        ("t", T),
        ("radius", part_spec("lightFlarePart.ring.radius", "Radius", 5.0, 600.0, 1.0, 0)),
        ("width", part_spec("lightFlarePart.width", "Width", 0.5, 80.0, 0.1, 1)),
        ("chroma", C),
        ("arc", part_spec("lightFlarePart.arc", "Half Ring", 0.0, 1.0, 0.01, 2)),
    ];
    const GHOST: &[(&str, ControlSpec)] = &[
        ("intensity", B),
        ("t", T),
        ("radius", part_spec("lightFlarePart.ghost.radius", "Radius", 1.0, 300.0, 1.0, 0)),
        ("sides", part_spec("lightFlarePart.sides", "Sides", 0.0, 12.0, 1.0, 0)),
        ("rim", part_spec("lightFlarePart.rim", "Rim", 0.0, 2.0, 0.01, 2)),
        ("softness", part_spec("lightFlarePart.softness", "Softness", 0.02, 1.0, 0.01, 2)),
        ("chroma", C),
    ];
    const WASH: &[(&str, ControlSpec)] = &[
        ("intensity", B),
        ("reach", part_spec("lightFlarePart.reach", "Reach", 50.0, 1000.0, 1.0, 0)),
        ("spread", part_spec("lightFlarePart.wash.spread", "Width", 1.0, 180.0, 1.0, 0)),
        ("chroma", C),
        ("angle", A),
    ];
    match kind {
        "glow" => GLOW,
        "streak" => STREAK,
        "star" => STAR,
        "rays" => RAYS,
        "ring" => RING,
        "ghost" => GHOST,
        "wash" => WASH,
        _ => &[],
    }
}

/// The Parts group: the selected flare's parts as rows (a click selects one, its eye hides it), the
/// buttons that add, copy, delete and reorder them, the centre point, and the selected part's sliders
/// and colour. Each change is one `lightFlare.part` command (a slider drag, one step).
fn parts(app: &mut LightcraftApp, ui: &mut egui::Ui, i: usize, f: &LensFlare, on: bool) {
    let t = Tokens::get(ui.ctx());
    let n = f.elements.len();
    if n > 0 && app.ui.flare_part >= n {
        app.ui.flare_part = n - 1;
    }
    let sel = app.ui.flare_part;
    let part = |app: &mut LightcraftApp, action: &str, k: usize| {
        if let Ok(v) = app.run("lightFlare.part", json!({"index": i, "part": k, "action": action}))
            && let Some(k) = v.get("part").and_then(serde_json::Value::as_u64)
        {
            app.ui.flare_part = k as usize;
        }
    };
    egui::Frame::NONE.inner_margin(egui::Margin { left: 24, right: 22, top: 2, bottom: 4 }).show(ui, |ui| {
        ui.add_enabled_ui(on, |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            for (k, e) in f.elements.iter().enumerate() {
                let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::hover());
                let resp = ui.interact(r, egui::Id::unique(("lightFlarePart", i, k)), Sense::click());
                let label = part_label(k, e);
                announce_selected(&resp, label.clone(), k == sel);
                register(ui, format!("lightFlarePart:{k}"), r);
                let shown = !f.is_part_hidden(k);
                let p = ui.painter();
                let fill = if k == sel {
                    t.tool_active
                } else if resp.hovered() {
                    t.hover.gamma_multiply(0.7)
                } else {
                    t.inset
                };
                p.rect_filled(r, 4.0, fill);
                if k == sel {
                    p.rect_filled(Rect::from_min_size(r.min, vec2(3.0, r.height())), 1.5, t.accent);
                }
                if resp.has_focus() {
                    p.rect_stroke(r, 4.0, Stroke::new(1.5, t.accent), egui::StrokeKind::Inside);
                }
                let ink = if shown { t.text } else { t.text_disabled };
                let g = one_line(p, &label, t.font(12.5), ink, r.width() - 12.0 - 30.0);
                p.galley(pos2(r.left() + 12.0, r.center().y - g.size().y / 2.0), g, ink);
                // show / hide: always there, struck through while hidden
                let er = Rect::from_center_size(pos2(r.right() - 14.0, r.center().y), vec2(20.0, 20.0));
                let eye = ui.interact(er, egui::Id::unique(("lightFlarePartVisible", i, k)), Sense::click());
                announce_selected(&eye, crate::i18n::tr_format!("Show part {n}", n = k + 1), shown);
                register(ui, format!("lightFlarePartVisible:{k}"), er);
                let eye = eye.on_hover_text(crate::i18n::tr(if shown { "Hide this part" } else { "Show this part" }));
                paint(ui.painter(), er.shrink(2.0), if shown { Icon::Eye } else { Icon::EyeOff }, if eye.hovered() { t.text } else { t.text_dim });
                if eye.clicked() {
                    part(app, if shown { "hide" } else { "show" }, k);
                }
                if resp.clicked() || resp.gained_focus() {
                    app.ui.flare_part = k;
                }
            }
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let add = ui.add_enabled_ui(n < range::MAX_ELEMENTS, |ui| text_button(ui, "lightFlarePartAdd", "Add Part", false)).inner;
                egui::Popup::menu(&add).show(|ui| {
                    ui.set_min_width(150.0);
                    for (id, name) in PART_KINDS {
                        let item = ui.selectable_label(false, crate::i18n::tr(name));
                        register(ui, format!("lightFlarePartAdd:{id}"), item.rect);
                        if item.clicked() {
                            if let Ok(v) = app.run("lightFlare.part", json!({"index": i, "action": "add", "kind": id}))
                                && let Some(k) = v.get("part").and_then(serde_json::Value::as_u64)
                            {
                                app.ui.flare_part = k as usize;
                            }
                            ui.close();
                        }
                    }
                });
                for (id, label, action, enabled) in [
                    ("lightFlarePartDuplicate", "Duplicate", "duplicate", n > 0 && n < range::MAX_ELEMENTS),
                    ("lightFlarePartRemove", "Delete", "remove", n > 0),
                ] {
                    if ui.add_enabled_ui(enabled, |ui| text_button(ui, id, label, false)).inner.clicked() {
                        part(app, action, sel);
                    }
                }
            });
            // the list's order (the light adds up the same in any order)
            ui.horizontal(|ui| {
                for (id, label, action, enabled) in
                    [("lightFlarePartUp", "Move Up", "up", sel > 0), ("lightFlarePartDown", "Move Down", "down", sel + 1 < n)]
                {
                    if ui.add_enabled_ui(enabled, |ui| text_button(ui, id, label, false)).inner.clicked() {
                        part(app, action, sel);
                    }
                }
            });
            ui.add_space(4.0);
            // the centre point the ghosts line up through (Alt on the photo moves it)
            ui.horizontal(|ui| {
                let (cx, cy) = f.center;
                ui.label(
                    egui::RichText::new(crate::i18n::tr_format!("Center point {x}, {y}", x = format!("{cx:.2}"), y = format!("{cy:.2}")))
                        .font(t.font(12.5))
                        .color(t.text_dim),
                );
                let off = (cx - 0.5).abs() > 1e-4 || (cy - 0.5).abs() > 1e-4;
                if ui.add_enabled_ui(off, |ui| text_button(ui, "lightFlareCenterReset", "Reset Center", false)).inner.clicked() {
                    let _ = app.run("develop.beginInteraction", json!({"label": "Light Flare Center"}));
                    let _ = app.run("develop.set", json!({"control": format!("lightFlare.{i}.centerX"), "value": 0.5}));
                    let _ = app.run("develop.set", json!({"control": format!("lightFlare.{i}.centerY"), "value": 0.5}));
                    let _ = app.run("develop.endInteraction", json!({}));
                }
            });
        });
    });
    let Some(e) = f.elements.get(sel) else { return };
    let v = serde_json::to_value(e).unwrap_or_default();
    super::sub_title(ui, &part_label(sel, e));
    for (key, spec) in part_settings(e.kind_id()) {
        let value = v.get(*key).and_then(serde_json::Value::as_f64).unwrap_or(spec.min);
        let out = crate::widgets::slider(ui, spec, value, on, None);
        super::apply_slider_out(app, spec, out, |app, x| {
            app.run("lightFlare.part", json!({"index": i, "part": sel, "action": "set", "values": {*key: x}}))
        });
    }
    part_colour(app, ui, i, sel, e, on);
}

/// The selected part's colour button: a change in the picker is one undo step.
fn part_colour(app: &mut LightcraftApp, ui: &mut egui::Ui, i: usize, k: usize, e: &lightcraft_develop::flare::FlareElement, on: bool) {
    let t = Tokens::get(ui.ctx());
    let mut rgb = e.clone().color_mut().map(|c| c.clamp(0.0, 1.0));
    let before = rgb;
    egui::Frame::NONE.inner_margin(egui::Margin { left: 24, right: 22, top: 4, bottom: 6 }).show(ui, |ui| {
        ui.add_enabled_ui(on, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(crate::i18n::tr("Color")).font(t.font(13.0)).color(t.text_dim));
                let r = ui.color_edit_button_rgb(&mut rgb);
                register(ui, "lightFlarePartColor", r.rect);
                r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::ColorWell, true, crate::i18n::tr("Part Color")));
            });
        });
    });
    if rgb != before {
        if !app.ui.flare_colour_editing {
            let _ = app.run("develop.beginInteraction", json!({"label": "Light Flare Part Color"}));
            app.ui.flare_colour_editing = true;
        }
        let _ = app.run("lightFlare.part", json!({"index": i, "part": k, "action": "set", "values": {"color": rgb}}));
    } else if app.ui.flare_colour_editing && !ui.input(|inp| inp.pointer.any_down()) {
        app.ui.flare_colour_editing = false;
        let _ = app.run("develop.endInteraction", json!({}));
    }
}
