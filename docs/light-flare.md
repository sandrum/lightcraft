# Light Flare

The **Light Flare** section of the Edit panel (after Lens Blur) adds lens flares to the photo: light
from sources you place, with the glows, streaks, rings and ghosts a real lens makes. A photo holds up
to 8 flares, each with its own light, preset, sliders and texture; their light adds up. It is an
ordinary edit setting (`DevelopSettings.light_flare`), so it is non-destructive and goes with copy and
paste (the Light Flare group), presets, sync, versions, history and Before / After.

## Using it

- **Apply** turns the flares on. **Place on Photo** comes on too, so the next click on the photo puts
  the selected flare's light there; a drag carries it along (one undo step). The button puts the tool
  down and picks it up again; closing the section or switching the flares off puts it down.
- **Flares** lists the photo's flares as numbered buttons: the selected one is what the controls below
  and Place on Photo act on. **Add** puts another flare down (at a free spot, selected, ready to be
  placed); **Remove** takes the selected one away. On the photo every flare's light has a marker (the
  selected one larger): a press on another marker selects that flare and drags it.
- **Flare** picks a preset from the library: Anamorphic, Spherical, Sun Flares and Streaks, 24 each
  (all original designs). A new preset keeps the light's position, the sliders and the texture.
- Sliders: Brightness, Scale, Source (size of the light at the source), Artifacts (strength of the
  ghosts and rings along the axis), Aspect (horizontal stretch of the source's glow), Rotation; Color:
  Hue and Tint (how far the flare's colours move towards the hue); Texture: lens dirt or film grain the
  flare's light shows up on, its opacity and Flip Horizontal / Flip Vertical.
- The section's eye switches the flares off without losing them.

## How it renders

The flare is laid out on the cropped output frame: its position, sizes and texture are fractions of
the frame, so a preview, the loupe and an export show the same flare. Thin details (the hot core,
streaks, rays) are sized from the full-size frame's diagonal, not from the render's. Its light is
screened over the display-encoded pixel in the finish stage, after the vignette and the tone curves
and before grain: `out = px + L·(1 − px)`, once per flare (screening is order-free). Above SDR white
(HDR renders) it adds nothing.

The flare's model and per-pixel arithmetic come from PhotoCraft's Light Flare fill layers
(`crates/develop/src/flare.rs`, `crates/pipeline/src/flare.rs`).

**GPU:** there is no GPU kernel yet. The GPU renderer hands a photo with an active flare to the CPU
(`Fallback::NotTried`, as for content-aware Fill spots), so the flare always renders, but such a
photo renders at CPU speed.

## Commands

- The sliders are indexed controls, `lightFlare.<i>.<field>` for flare `i` (as Point Color's):
  `develop.set {control: "lightFlare.0.x", value}` with fields `x`, `y` (position as a fraction of the
  cropped frame, −1…2: the light may sit outside it), `brightness`, `scale`, `source`, `artifacts`,
  `aspect`, `rotation`, `hue`, `tint`, `textureOpacity`.
- `develop.merge {settings: {light_flare: {enabled: true}}}` applies the flares.
- `lightFlare.add {preset?, x?, y?}` → `{index}`: another flare (at most 8); on a photo without flares,
  its first.
- `lightFlare.remove {index}`.
- `lightFlare.preset {preset, index?}`: the preset's elements for flare `index` (default 0), keeping its
  position and controls; applies the flares.
- `lightFlare.texture {index?, texture?, flipX?, flipY?}`: `"none"` removes the texture.
- `lightFlare.presets {category?}`: the library and the textures.
- `section.lightFlare` (Cmd+7) opens the section; `ui.pointer` on the photo places the light while
  the Place on Photo tool (`ui.tool` `"lightFlare"`) is on; the markers are `lightFlare:light:<i>`, the
  flare buttons `button:lightFlareSelect:<i>`, `button:lightFlareAdd` and `button:lightFlareRemove`.

## Catalog

Catalog format 14 added `DevelopSettings.light_flare` (`{enabled, flares: [...]}`). It is not written
while the section is off and untouched, so photos without flares keep their stored settings and
cached previews. A single `flare` object, as the section's first builds wrote, reads as a list of one.
