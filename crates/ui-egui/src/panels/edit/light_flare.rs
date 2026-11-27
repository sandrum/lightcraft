//! The Light Flare section: Apply and Place on Photo; the photo's flares as a list (one row each:
//! its number, its preset, an eye that hides it; Add Flare and Delete under it); then the selected
//! flare's preset and main sliders, and its Shape, Color and Texture groups, folded until opened.

use super::{LightcraftApp, Tokens, control, register, text_button};
use crate::icons::{Icon, paint};
use crate::widgets::{announce_disclosure, announce_selected, icon_button, one_line};
use egui::{Align2, Rect, Sense, Stroke, pos2, vec2};
use lightcraft_develop::DevelopSettings;
use lightcraft_develop::flare::{FlareCategory, LensFlare, MAX_FLARES, TextureKind, presets};
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
    if group(app, ui, "shape", "Shape", "") {
        for c in ["artifacts", "aspect", "rotation"] {
            control(app, ui, d, &format!("lightFlare.{i}.{c}"), on);
        }
    }
    let colour = if f.tint > 0.0 { crate::i18n::tr_format!("Hue {h}°", h = f.hue.round()) } else { crate::i18n::tr("Preset colours").to_string() };
    if group(app, ui, "color", "Color", &colour) {
        control(app, ui, d, &format!("lightFlare.{i}.hue"), on);
        control(app, ui, d, &format!("lightFlare.{i}.tint"), on);
    }
    if group(app, ui, "texture", "Texture", crate::i18n::tr(f.texture.kind.label())) {
        texture(app, ui, d, i, f, on);
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
