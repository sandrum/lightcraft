//! The Light Flare section: Apply, the photo's flares (pick one, add another, remove it), Place on
//! Photo (move the selected flare's light on the photo), and the selected flare's preset, sliders,
//! colour and the lens texture its light shows.

use super::{LightcraftApp, RichText, Tokens, control, register, sub_title, text_button};
use lightcraft_develop::DevelopSettings;
use lightcraft_develop::flare::{FlareCategory, MAX_FLARES, TextureKind, presets};
use serde_json::json;

/// The tool that places the light with a click or drag on the photo (`panels/detail/light_flare.rs`).
pub(crate) const PLACE_TOOL: &str = "lightFlare";

pub(super) fn light_flare(app: &mut LightcraftApp, ui: &mut egui::Ui, d: &DevelopSettings) {
    let t = Tokens::get(ui.ctx());
    let lf = &d.light_flare;
    let n = lf.flares.len();
    // the selection stays on a flare the photo has (another photo may have fewer)
    if n > 0 && app.ui.flare_index >= n {
        app.ui.flare_index = n - 1;
    }
    let i = app.ui.flare_index;
    // the light can't be placed while the flares are off or there are none
    if app.ui.tool == PLACE_TOOL && (!lf.enabled || n == 0) {
        app.ui.tool.clear();
    }
    egui::Frame::NONE.inner_margin(egui::Margin { left: 24, right: 22, top: 4, bottom: 4 }).show(ui, |ui| {
        ui.horizontal(|ui| {
            let mut on = lf.enabled;
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
                ui.add_enabled_ui(lf.enabled && n > 0, |ui| {
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
    // the flares: one button each (the selected one is highlighted), Add and Remove
    egui::Frame::NONE.inner_margin(egui::Margin { left: 24, right: 22, top: 2, bottom: 4 }).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(crate::i18n::tr("Flares")).font(t.font(13.0)).color(t.text_dim));
            for k in 0..n {
                let r = text_button(ui, &format!("lightFlareSelect:{k}"), &(k + 1).to_string(), k == i).on_hover_text(crate::i18n::tr_format!(
                    "Flare {n}: {preset}",
                    n = k + 1,
                    preset = lf.flares[k].preset
                ));
                if r.clicked() {
                    app.ui.flare_index = k;
                }
            }
            let r = ui.add_enabled_ui(n < MAX_FLARES, |ui| text_button(ui, "lightFlareAdd", "Add", false)).inner;
            if r.on_hover_text(crate::i18n::tr("Add another flare (up to 8)")).clicked()
                && let Ok(v) = app.run("lightFlare.add", json!({}))
                && let Some(k) = v.get("index").and_then(serde_json::Value::as_u64)
            {
                app.ui.flare_index = k as usize;
                app.ui.tool = PLACE_TOOL.to_string();
            }
            let r = ui.add_enabled_ui(n > 0, |ui| text_button(ui, "lightFlareRemove", "Remove", false)).inner;
            if r.on_hover_text(crate::i18n::tr("Remove the selected flare")).clicked() && app.run("lightFlare.remove", json!({"index": i})).is_ok() {
                app.ui.flare_index = i.saturating_sub(1);
            }
        });
    });
    let Some(f) = lf.flares.get(i) else { return };
    let on = lf.enabled;
    // the selected flare's preset: a menu of the library, one submenu per kind of lens
    egui::Frame::NONE.inner_margin(egui::Margin { left: 24, right: 22, top: 4, bottom: 6 }).show(ui, |ui| {
        ui.add_enabled_ui(on, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(crate::i18n::tr("Flare")).font(t.font(13.0)).color(t.text_dim));
                let max = ui.available_width();
                let r = crate::widgets::dropdown_within(ui, "lightFlarePreset", &f.preset, t.font(14.0), t.text_label, max);
                let below = r.rect.bottom();
                egui::Popup::menu(&r).show(|ui| {
                    crate::menu_level::level(ui, 1, Some(below), |ui| preset_menu(app, ui, i, &f.preset, below));
                });
            });
        });
    });
    for c in ["brightness", "scale", "source", "artifacts", "aspect", "rotation"] {
        control(app, ui, d, &format!("lightFlare.{i}.{c}"), on);
    }
    sub_title(ui, "Color");
    control(app, ui, d, &format!("lightFlare.{i}.hue"), on);
    control(app, ui, d, &format!("lightFlare.{i}.tint"), on);
    sub_title(ui, "Texture");
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
