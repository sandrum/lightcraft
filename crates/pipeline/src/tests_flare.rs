//! Light Flare through the whole pipeline: screened over the finished photo where its light is,
//! placed on the output frame, several adding up, switched off by its section's eye, and the same
//! flare in a preview as in the export (`crate::flare`).

use lightcraft_develop::{DevelopSettings, LightFlare};
use lightcraft_raster::Rgb32f;

use crate::{OutputDepth, RenderRequest, SourceInfo, render};

fn flare_at(x: f32, y: f32) -> DevelopSettings {
    let mut light_flare = LightFlare { enabled: true, ..LightFlare::default() };
    light_flare.flares[0].source = (x, y);
    DevelopSettings { light_flare, ..DevelopSettings::default() }
}

/// Display-encoded luminance (0..255, 16-bit precision) of a render of `src` at `w × h`.
fn luma(src: &Rgb32f, s: &DevelopSettings, w: usize, h: usize) -> Vec<f64> {
    let mut req = RenderRequest::fit(w, h);
    req.depth = OutputDepth::U16;
    let r = render(src, &SourceInfo::default(), s, &req);
    let Some(crate::DeepSamples::U16(v)) = r.deep.map(|d| d.samples) else { panic!("no 16-bit render") };
    v.chunks(3).map(|p| (0.2126 * p[0] as f64 + 0.7152 * p[1] as f64 + 0.0722 * p[2] as f64) / 257.0).collect()
}

/// Mean of `v` (a `w`-wide image) over the box around fraction `(fx, fy)`, `r` pixels each way.
fn around(v: &[f64], w: usize, fx: f64, fy: f64, r: usize) -> f64 {
    let h = v.len() / w;
    let (cx, cy) = ((fx * w as f64) as usize, (fy * h as f64) as usize);
    let (x0, x1, y0, y1) = (cx.saturating_sub(r), (cx + r).min(w - 1), cy.saturating_sub(r), (cy + r).min(h - 1));
    let mut sum = 0.0;
    for y in y0..=y1 {
        for x in x0..=x1 {
            sum += v[y * w + x];
        }
    }
    sum / ((x1 - x0 + 1) * (y1 - y0 + 1)) as f64
}

#[test]
fn the_light_brightens_the_photo_around_the_source_only_when_applied() {
    let src = Rgb32f::filled(320, 200, [0.08; 3]);
    let plain = luma(&src, &DevelopSettings::default(), 320, 200);
    let mut off = flare_at(0.25, 0.4);
    off.light_flare.enabled = false;
    assert_eq!(luma(&src, &off, 320, 200), plain, "a flare that is not applied changes nothing");
    let lit = luma(&src, &flare_at(0.25, 0.4), 320, 200);
    let gain = |fx, fy| around(&lit, 320, fx, fy, 3) - around(&plain, 320, fx, fy, 3);
    assert!(gain(0.25, 0.4) > 60.0, "the source is bright: +{}", gain(0.25, 0.4));
    assert!(gain(0.25, 0.4) > 4.0 * gain(0.95, 0.05).max(0.5), "brightest at the source: {} vs {}", gain(0.25, 0.4), gain(0.95, 0.05));
    assert!(lit.iter().zip(&plain).all(|(a, b)| a >= &(b - 0.5)), "screened light never darkens");
}

#[test]
fn the_light_follows_its_position_on_the_frame() {
    let src = Rgb32f::filled(320, 200, [0.08; 3]);
    let left = luma(&src, &flare_at(0.2, 0.5), 320, 200);
    let right = luma(&src, &flare_at(0.8, 0.5), 320, 200);
    assert!(around(&left, 320, 0.2, 0.5, 3) > around(&right, 320, 0.2, 0.5, 3) + 30.0);
    assert!(around(&right, 320, 0.8, 0.5, 3) > around(&left, 320, 0.8, 0.5, 3) + 30.0);
}

/// Each flare adds its own light: two lights, two bright spots, and the second flare adds to the
/// first's light where they overlap.
#[test]
fn several_flares_each_add_their_light() {
    let src = Rgb32f::filled(320, 200, [0.08; 3]);
    let one = flare_at(0.2, 0.5);
    let mut two = one.clone();
    two.light_flare.add(None, Some((0.8, 0.5)));
    let (a, b) = (luma(&src, &one, 320, 200), luma(&src, &two, 320, 200));
    assert!(around(&b, 320, 0.8, 0.5, 3) > around(&a, 320, 0.8, 0.5, 3) + 30.0, "the second light is there");
    assert!((around(&b, 320, 0.2, 0.5, 3) - around(&a, 320, 0.2, 0.5, 3)).abs() < 15.0, "the first is still there");
    assert!(b.iter().zip(&a).all(|(x, y)| *x >= y - 0.5), "a second flare never takes light away");
}

#[test]
fn the_sections_eye_switches_the_flare_off() {
    let src = Rgb32f::filled(160, 100, [0.08; 3]);
    let mut s = flare_at(0.5, 0.5);
    s.set_section_enabled("lightFlare", false);
    assert_eq!(luma(&src, &s, 160, 100), luma(&src, &DevelopSettings::default(), 160, 100));
}

/// A preview shows the export's flare: the light at the same place and of the same strength,
/// box-downscaled to its size.
#[test]
fn a_small_render_shows_the_large_renders_flare() {
    let src = Rgb32f::filled(300, 200, [0.1; 3]);
    let s = flare_at(0.3, 0.45);
    let (w, h, k) = (1200, 800, 4);
    let big = luma(&src, &s, w, h);
    let small = luma(&src, &s, w / k, h / k);
    let (sw, sh) = (w / k, h / k);
    let mut worst = 0.0f64;
    let mut total = 0.0;
    for y in 0..sh {
        for x in 0..sw {
            let mut sum = 0.0;
            for dy in 0..k {
                for dx in 0..k {
                    sum += big[(y * k + dy) * w + x * k + dx];
                }
            }
            let d = (sum / (k * k) as f64 - small[y * sw + x]).abs();
            total += d;
            // at the source the hot core is a point a few pixels wide: a sample of it and an average
            // over it differ (2 % of the width around it)
            let (dx, dy) = (x as f64 - 0.3 * sw as f64, y as f64 - 0.45 * sh as f64);
            if dx.hypot(dy) > 0.02 * sw as f64 {
                worst = worst.max(d);
            }
        }
    }
    let mean = total / (sw * sh) as f64;
    assert!(mean < 1.5, "on average within 1.5 of 255: {mean}");
    assert!(worst < 40.0, "no part of the flare is missing from the preview: {worst}");
}
