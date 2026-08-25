---
id: i0001
title: Controls not working on 8bitDo Pro 2
status: in-progress
type: issue
created: 2026-08-25
updated: 2026-08-25
status-changed: 2026-08-25T22:24:22
---

Testing it wuth the Dualsense the two analog sticks work perfectly for controlling the pad.

Using the 8bitDo Pro 2 there’s some issue might be that some inversion of axes

## Notes

### Where the paddle input comes from

- `src/ship/mod.rs:101` binds `MatchActions::ArticulateLeft/Right` to
  `GamepadStick::LEFT` / `GamepadStick::RIGHT`, plus the two analog triggers for
  up/down.
- `ship_articulate` (`src/ship/mod.rs:140`) is the only consumer. It uses the
  stick `y` values twice: for the paddle tilt
  (`d = Vec2::new(-1.0, l.y) - Vec2::new(1.0, r.y)`, then `d.perp().angle_to(-Y)`)
  and for the forward/back position (`tz = ARENA_HEIGHT_H - comp.y * PADDLE_LIFT`).
  A sign flip on `y` therefore both mirrors the tilt and pushes the paddle the
  wrong way — which matches the "inversion of axes" hunch.

### Why this is device-dependent (input chain audit)

`leafwing-input-manager 0.21` → `bevy_gilrs 0.19` → `gilrs 0.11.2` →
`gilrs-core 0.6.8`.

- `bevy_gilrs` (`converter.rs:28`) is a plain 1:1 passthrough of gilrs axes; it
  adds no per-device handling. So the whole difference between DualSense and
  8BitDo Pro 2 is decided inside gilrs.
- On macOS, `gilrs-core` sets `IS_Y_AXIS_REVERSED = true` and `gilrs`
  (`gamepad.rs:1171`) only applies that flip when the axis has already been
  *recognised* as `LeftStickY` / `RightStickY` / `DPadY`. An axis that falls
  through as `Axis::Unknown` is **not** flipped — so a device whose sticks are
  not resolved by the mapping DB comes out inverted on Y relative to a device
  that is. That is a concrete mechanism for exactly the reported symptom.
- Recognition is by SDL GUID (bus/vendor/product/version, `macos/gamepad.rs:279`)
  against the bundled `SDL_GameControllerDB`. The DB has `Mac OS X` entries for
  8BitDo Pro 2 under vendor `c82d`, products `0660` (versions `0001`/`0002`) —
  but the Pro 2 enumerates with a *different* vendor/product per switch position
  on the back (S / X / D / macOS mode). In S mode it presents as a Switch Pro
  Controller, in X mode as an Xbox pad, etc. So which mode the pad is in decides
  whether it is matched at all.
- Separately: gilrs parses the SDL `~` (invert) suffix in
  `mapping/parser.rs:201` but never applies it — `Token::AxisMapping.inverted`
  is `#[allow(dead_code)]` and is read nowhere else in the crate. That is an
  upstream limitation. It does not bite the `Mac OS X` 8BitDo entries (they have
  no `~`), but it would bite any mapping that needs one.

### Why this is blocked

The correct fix differs per root cause, and they are indistinguishable from here
without the hardware:

- pad not matched by the mapping DB → ship a custom SDL mapping for it;
- pad matched but mapped wrong → override the mapping;
- only Y inverted → invert at the `ship_articulate` boundary behind a setting;
- sticks fine but drifting past the deadzone → tune the deadzone instead.

Guessing would mean writing an untestable fix for a device that is not attached
to this machine, so the question below is on the card instead.

## Log

- 2026-08-25 status → ready (app)
- 2026-08-25 status → in-progress (agent)
