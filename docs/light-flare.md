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
- The flare list has a row per flare: its number (the same number shows next to its marker on the
  photo) and its preset. A click on a row selects that flare, and so does Tab or ↑ / ↓ in the list;
  the selected row is marked by a bar on its left as well as its fill. The row's eye (on hover, and
  always while hidden) hides that flare without losing it; right-click a row to show, hide or
  delete it. **Add Flare** puts another flare down (at a free spot, selected, ready to be placed),
  the bin deletes the selected one, and the count shows how many of the 8 are used. On the photo,
  every flare's light has a numbered marker (the selected one larger): a press on another marker
  selects that flare and drags it. The selected flare's **center point** (where the line its ghosts
  and rings line up on runs through from the light; the frame centre unless moved) shows as a diamond
  with that line drawn through it: drag the diamond, or Alt-click / Alt-drag anywhere on the photo, to
  move it.
- **Preset** picks a preset from the library: Anamorphic, Spherical, Sun Flares and Streaks, 24 each
  (all original designs). A new preset keeps the light's position, the sliders and the texture.
- Brightness, Scale and Source (size of the light at the source) are always shown. The rest are in
  groups that stay folded until their title is clicked (the choice is kept), each showing what it is
  set to:
  - **Light**: Spread (a tone curve on the flare's light: below 1 only the hot core is left, above 1
    the faint halo and ghosts spread further) and Source Brightness (strength of the light at the
    source: 0 leaves only the lens artifacts).
  - **Shape**: Artifacts (strength of the ghosts and rings along the axis), Aspect (horizontal stretch
    of the source's glow: above 1 the wide anamorphic band, below 1 a tall oval), Rotation, Softness
    (thin lines thicken and edges blur), Color Fringe (red and blue part about the light, the sign
    choosing which goes outward), and **Flip Horizontal** / **Flip Vertical**, which mirror the whole
    flare: its light and center point, its rotation and every part's own angle, and its texture.
  - **Color**: Saturation (0 = white light, rainbow edges included; 2 = stronger), Hue, and Tint (how
    far the flare's colours move towards the hue).
  - **Texture**: lens dirt or film grain the flare's light shows up on, its opacity and Flip
    Horizontal / Flip Vertical of the texture.
  - **Parts**: the flare's own parts (Glow, Streak, Star, Rays, Ring, Ghost, Edge Wash), numbered,
    with where a ring or ghost sits on the axis (`Ghost at 0.70`: 0 = the light, 1 = the center
    point, 2 = as far again). A click selects a part; its eye hides it without losing its settings.
    **Add Part** (a menu of the kinds), **Duplicate**, **Delete**, **Move Up** / **Move Down** (the
    order is only the list's), the center point and **Reset Center**; then the selected part's own
    sliders (Brightness, Size, Length, Thickness, Angle, Points, Count, Variation, Position, Radius,
    Width, Rainbow Edge, Half Ring, Sides, Rim, Softness, Reach…, as its kind has them) and its
    **Color**. Each change is one undo step (a slider drag, or a change in the colour picker, too).
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
  `aspect`, `rotation`, `hue`, `tint`, `textureOpacity`, `spread`, `saturation`, `sourceBrightness`,
  `softness`, `fringe`, `centerX`, `centerY`.
- `develop.merge {settings: {light_flare: {enabled: true}}}` applies the flares.
- `lightFlare.add {preset?, x?, y?}` → `{index}`: another flare (at most 8); on a photo without flares,
  its first.
- `lightFlare.remove {index}`.
- `lightFlare.visible {index, visible?}` → `{index, visible}`: shows or hides that flare (no
  `visible`: toggles); a hidden flare keeps its settings and renders nothing.
- `lightFlare.preset {preset, index?}`: the preset's elements for flare `index` (default 0), keeping its
  position and controls; applies the flares.
- `lightFlare.texture {index?, texture?, flipX?, flipY?}`: `"none"` removes the texture.
- `lightFlare.part {index?, part?, action, kind?, values?}` → `{part}`: flare `index`'s parts;
  `action` is `add` (`kind`: glow, streak, star, rays, ring, ghost, wash), `duplicate`, `remove`,
  `up`, `down`, `show`, `hide` or `set` (`values`: the part's own fields as `elements` stores them,
  and `color: [r, g, b]`). Hidden marks follow their parts through every change of order.
- `lightFlare.flip {index?, axis: "horizontal"|"vertical"}`: mirrors the whole flare.
- `lightFlare.presets {category?}`: the library and the textures.
- `section.lightFlare` (Cmd+7) opens the section; `ui.pointer` on the photo places the light while
  the Place on Photo tool (`ui.tool` `"lightFlare"`) is on; the markers are `lightFlare:light:<i>`; the
  list's rows are `lightFlareRow:<i>`, their eyes `lightFlareVisible:<i>`, then `button:lightFlareAdd`
  and `icon:lightFlareRemove`; the groups' titles are `lightFlareGroup:light|shape|color|texture|parts`;
  the center point's diamond is `lightFlare:center` (Alt with `ui.pointer` moves it); the parts are
  `lightFlarePart:<k>` with eyes `lightFlarePartVisible:<k>`, then `button:lightFlarePartAdd` (its
  menu `lightFlarePartAdd:<kind>`), `button:lightFlarePartDuplicate`, `…Remove`, `…Up`, `…Down`,
  `button:lightFlareCenterReset` and `lightFlarePartColor`.

## Catalog

Catalog format 14 added `DevelopSettings.light_flare` (`{enabled, flares: [...]}`; a flare's
`hidden` is written only while it is hidden, and `hiddenParts`, a bit per part, only while a part is;
settings saved before the center point and the Light / Shape / Color additions read with their
defaults). It is not written
while the section is off and untouched, so photos without flares keep their stored settings and
cached previews. A single `flare` object, as the section's first builds wrote, reads as a list of one.
