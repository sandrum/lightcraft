//! Light Flare through the commands: choosing a preset applies the flare in one undo step and keeps
//! where the light is; the light moves with `develop.set`; flares are added and removed; the library
//! lists itself; flares are copied to another photo with their settings group.

use lightcraft_catalog::PhotoId;
use lightcraft_develop::LightFlare;
use serde_json::json;

use crate::Session;

fn flare(s: &Session, id: PhotoId) -> LightFlare {
    s.develop_of(id).unwrap().light_flare.clone()
}

#[test]
fn a_preset_applies_the_flare_in_one_step_and_keeps_the_light() {
    let mut s = Session::with_demo();
    let id = s.active().unwrap();
    s.execute("develop.set", &json!({"control": "lightFlare.0.x", "value": 0.8})).unwrap();
    s.execute("develop.set", &json!({"control": "lightFlare.0.y", "value": 0.15})).unwrap();
    assert!(!flare(&s, id).enabled, "moving the light doesn't apply the flare");
    let r = s.execute("lightFlare.preset", &json!({"preset": "morning spill"})).unwrap();
    assert_eq!(r["preset"], "Morning Spill");
    let f = flare(&s, id);
    assert!(f.enabled && s.develop_of(id).unwrap().section_enabled("lightFlare"));
    assert_eq!((f.flares[0].preset.as_str(), f.flares[0].source), ("Morning Spill", (0.8, 0.15)));
    s.execute("edit.undo", &json!({})).unwrap();
    let back = flare(&s, id);
    assert!(!back.enabled && back.flares[0].preset != "Morning Spill", "one undo step");
    // an unknown preset changes nothing
    assert!(s.execute("lightFlare.preset", &json!({"preset": "no such flare"})).is_err());
    assert_eq!(flare(&s, id), back);
}

#[test]
fn the_library_and_the_texture_commands() {
    let mut s = Session::with_demo();
    let id = s.active().unwrap();
    let lib = s.execute("lightFlare.presets", &json!({})).unwrap();
    let cats = lib["categories"].as_array().unwrap();
    assert_eq!(cats.len(), 4);
    assert!(cats.iter().all(|c| c["presets"].as_array().is_some_and(|p| p.len() >= 24)));
    assert!(lib["textures"].as_array().unwrap().iter().any(|t| t["id"] == "dust"));
    assert!(s.execute("lightFlare.presets", &json!({"category": "nope"})).is_err());
    s.execute("lightFlare.texture", &json!({"texture": "dust", "flipX": true})).unwrap();
    let t = flare(&s, id).flares[0].texture;
    assert_eq!((t.kind.id(), t.flip_x, t.flip_y), ("dust", true, false));
    assert!(s.execute("lightFlare.texture", &json!({"texture": "mud"})).is_err());
}

#[test]
fn a_flare_is_copied_with_its_group() {
    let mut s = Session::with_demo();
    let ids: Vec<PhotoId> = s.visible_cloned().into_iter().take(2).collect();
    s.execute("library.select", &json!({"ids": [ids[0].0]})).unwrap();
    s.execute("lightFlare.preset", &json!({"preset": "Blue Horizon"})).unwrap();
    s.execute("develop.set", &json!({"control": "lightFlare.0.brightness", "value": 1.6})).unwrap();
    s.execute("develop.copy", &json!({"groups": ["lightFlare"]})).unwrap();
    s.execute("library.select", &json!({"ids": [ids[1].0]})).unwrap();
    s.execute("develop.paste", &json!({})).unwrap();
    assert_eq!(flare(&s, ids[1]), flare(&s, ids[0]));
}

#[test]
fn flares_are_added_and_removed() {
    let mut s = Session::with_demo();
    let id = s.active().unwrap();
    // the first Add on a photo without flares gives it one, at the place asked for
    let r = s.execute("lightFlare.add", &json!({"preset": "Sun Burst", "x": 0.1, "y": 0.9})).unwrap();
    assert_eq!(r["index"], 0);
    let f = flare(&s, id);
    assert!(f.enabled && f.flares.len() == 1 && f.flares[0].preset == "Sun Burst" && f.flares[0].source == (0.1, 0.9));
    // more, each with its own controls and preset
    for k in 1..8 {
        assert_eq!(s.execute("lightFlare.add", &json!({})).unwrap()["index"], k);
    }
    assert!(s.execute("lightFlare.add", &json!({})).is_err(), "at most 8");
    s.execute("develop.set", &json!({"control": "lightFlare.3.brightness", "value": 2.5})).unwrap();
    s.execute("lightFlare.preset", &json!({"preset": "Morning Spill", "index": 3})).unwrap();
    let f = flare(&s, id);
    assert_eq!((f.flares[3].brightness, f.flares[3].preset.as_str()), (2.5, "Morning Spill"));
    assert_eq!(f.flares[2].brightness, 1.0);
    assert!(s.execute("lightFlare.preset", &json!({"preset": "Sun Burst", "index": 8})).is_err());
    // removing one shifts the rest down; one undo step brings it back
    s.execute("lightFlare.remove", &json!({"index": 0})).unwrap();
    let f = flare(&s, id);
    assert_eq!((f.flares.len(), f.flares[2].preset.as_str()), (7, "Morning Spill"));
    s.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(flare(&s, id).flares.len(), 8);
    assert!(s.execute("lightFlare.remove", &json!({"index": 8})).is_err());
}

#[test]
fn a_hidden_flare_keeps_its_settings_and_stops_rendering() {
    let mut s = Session::with_demo();
    let id = s.active().unwrap();
    s.execute("lightFlare.add", &json!({})).unwrap();
    s.execute("develop.set", &json!({"control": "lightFlare.0.brightness", "value": 2.0})).unwrap();
    let r = s.execute("lightFlare.visible", &json!({"index": 0})).unwrap();
    assert_eq!(r["visible"], false, "no `visible`: it toggles");
    let f = flare(&s, id);
    assert!(f.flares[0].hidden && f.flares[0].brightness == 2.0);
    assert!(!f.is_active(), "its only flare hidden: nothing renders");
    s.execute("lightFlare.visible", &json!({"index": 0, "visible": true})).unwrap();
    assert!(flare(&s, id).is_active());
    assert!(s.execute("lightFlare.visible", &json!({"index": 1})).is_err());
}

#[test]
fn a_flares_parts_are_added_copied_moved_hidden_set_and_removed() {
    let mut s = Session::with_demo();
    let id = s.active().unwrap();
    s.execute("lightFlare.add", &json!({"preset": "Bare Bulb"})).unwrap();
    let n = flare(&s, id).flares[0].elements.len();
    let r = s.execute("lightFlare.part", &json!({"action": "add", "kind": "ghost"})).unwrap();
    assert_eq!(r["part"], n);
    s.execute("lightFlare.part", &json!({"action": "hide", "part": n})).unwrap();
    s.execute("lightFlare.part", &json!({"action": "duplicate", "part": n})).unwrap();
    let f = &flare(&s, id).flares[0];
    assert!(f.elements.len() == n + 2 && f.is_part_hidden(n) && !f.is_part_hidden(n + 1), "the copy shows");
    // moving a hidden part up carries its mark
    let r = s.execute("lightFlare.part", &json!({"action": "up", "part": n})).unwrap();
    assert_eq!(r["part"], n - 1);
    assert!(flare(&s, id).flares[0].is_part_hidden(n - 1));
    // set: a number, a whole number and the colour, one undo step
    s.execute("lightFlare.part", &json!({"action": "set", "part": n + 1, "values": {"radius": 55.5, "sides": 6.4, "color": [1.0, 0.5, 2.0]}}))
        .unwrap();
    let v = serde_json::to_value(&flare(&s, id).flares[0].elements[n + 1]).unwrap();
    assert_eq!((v["radius"].as_f64(), v["sides"].as_u64(), v["color"][2].as_f64()), (Some(55.5), Some(6), Some(1.0)));
    s.execute("edit.undo", &json!({})).unwrap();
    assert_ne!(serde_json::to_value(&flare(&s, id).flares[0].elements[n + 1]).unwrap()["radius"], 55.5);
    // wrong fields, kinds and indices change nothing
    for bad in [
        json!({"action": "set", "part": 0, "values": {"wings": 2}}),
        json!({"action": "set", "part": 0, "values": {"radius": "big"}}),
        json!({"action": "add", "kind": "comet"}),
        json!({"action": "remove", "part": 99}),
        json!({"action": "explode", "part": 0}),
    ] {
        assert!(s.execute("lightFlare.part", &bad).is_err(), "{bad}");
    }
    s.execute("lightFlare.part", &json!({"action": "remove", "part": n - 1})).unwrap();
    let f = &flare(&s, id).flares[0];
    assert!(f.elements.len() == n + 1 && (0..f.elements.len()).all(|k| !f.is_part_hidden(k)), "the hidden part went with its mark");
}

#[test]
fn a_flare_flips_and_its_centre_point_and_new_sliders_are_controls() {
    let mut s = Session::with_demo();
    let id = s.active().unwrap();
    s.execute("lightFlare.add", &json!({"x": 0.2, "y": 0.3})).unwrap();
    for (c, v) in
        [("centerX", 0.7), ("centerY", 0.6), ("spread", 1.8), ("saturation", 0.4), ("sourceBrightness", 0.5), ("softness", 0.3), ("fringe", -0.5)]
    {
        s.execute("develop.set", &json!({"control": format!("lightFlare.0.{c}"), "value": v})).unwrap();
    }
    s.execute("lightFlare.flip", &json!({"axis": "horizontal"})).unwrap();
    let f = &flare(&s, id).flares[0];
    assert!((f.source.0 - 0.8).abs() < 1e-6 && (f.center.0 - 0.3).abs() < 1e-6 && f.center.1 == 0.6 && f.texture.flip_x);
    assert_eq!((f.spread, f.saturation, f.source_brightness, f.softness, f.fringe), (1.8, 0.4, 0.5, 0.3, -0.5));
    assert!(s.execute("lightFlare.flip", &json!({"axis": "diagonal"})).is_err());
}
