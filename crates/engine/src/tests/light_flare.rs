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
