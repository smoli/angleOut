---
id: i0001
title: Controls not working on 8bitDo Pro 2
status: done
type: issue
created: 2026-08-25
updated: 2026-08-25
status-changed: 2026-08-25T22:53:48
usage-tokens: 43198
usage-cost: 8.511365
---

I traced the whole input path (details in **Notes** above) and the fix depends on facts only the attached hardware can give. Two things I need:

**1. Which position is the mode switch on the back of the Pro 2 set to?**

This decides which vendor/product id the pad enumerates as, and therefore whether gilrs matches it in the bundled SDL controller DB at all. The DB only has `Mac OS X` entries for vendor `c82d` / product `0660`.

- [ ] `S` (Switch mode)
- [x] `X` (X-input mode)
- [ ] `D` (D-input mode)
- [ ] `macOS` mode
- [ ] Not sure / it is a "Pro 2 for Xbox" variant

**2. What exactly misbehaves?** Push each stick and say what the paddle does — that alone tells me which axis is flipped.

- [x] Push a stick **up** → paddle moves **towards me** instead of away (Y inverted)
- [ ] Push a stick **left** → paddle moves **right** (X inverted)
- [ ] Paddle **tilts the wrong way** but moves correctly
- [ ] Sticks do **nothing at all**
- [ ] Paddle **drifts / never returns to rest** when the sticks are centred (deadzone, not inversion)
- [ ] Only the **right** stick is wrong, left is fine
- [ ] Something else — describe below

Also useful if you know it: do the **triggers** (paddle up/down) and the **face buttons** (launch/grab) work on the Pro 2?

---

If answering #2 precisely is awkward, I can instead add a small opt-in input-debug overlay (behind an env var, off in normal play) that prints the detected pad name, its SDL GUID and the live raw axis values, so you just read the numbers off the screen. Say the word and I'll build that first:

- [ ] Just fix it from my answers above
- [ ] Build the input-debug overlay first, I'll report the values

Only left stick works ad is inverted. Right stick does nothing. Moving it to D makes it work perfectly

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
  (`gamepad.rs:1171`) applies that flip to every axis it recognises as
  `LeftStickY` / `RightStickY` / `DPadY`, unconditionally — there is no
  per-device escape. The flip is correct for pads that follow the HID
  "down is positive" convention and wrong for pads that do not.
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

### Root cause (confirmed against the hardware)

The answer above pinned it down: the pad was in **`X`** (X-input) mode, where the
left stick works but is Y-inverted and the right stick does nothing at all.
Moving the switch to **`D`** makes it work perfectly. That is fully explained by
the audit above:

- gilrs' macOS fallback table (`macos/io_kit.rs:312`) only resolves the HID
  usages `X`, `Y`, `Z` and `Rz` — mapping them to left-X, left-Y, right-X and
  right-Y respectively. In `D` mode the Pro 2 puts its right stick on `Z`/`Rz`,
  which is exactly what that table expects, so everything lines up.
- In `X` mode the pad follows the Xbox descriptor and puts the right stick on
  `Rx`/`Ry`, which the table does not cover. Those events become `Axis::Unknown`,
  and `bevy_gilrs` (`gilrs_system.rs:99`) drops them with `continue` before they
  reach Bevy. **The right stick is therefore unreachable from game code** — no
  remapping, deadzone or inversion setting on our side can recover it.
- The inverted Y has the same origin: gilrs applies its macOS
  `IS_Y_AXIS_REVERSED` flip unconditionally, which is right for the HID
  "down is positive" convention that `D` mode follows and wrong for X-input.

A custom SDL mapping via `SDL_GAMECONTROLLERCONFIG` could in principle re-point
the right stick, but SDL axis indices resolve against the device's own element
enumeration order (`mapping/mod.rs:294`), which is not knowable without the pad
in hand — and it still could not fix the inversion, since gilrs ignores the `~`
suffix. So `D` mode is the answer, not a code change.

### What was done

Since the defect is outside the game, the fix is to stop it being undiagnosable:

- `src/input/mod.rs` (new, wired up in `src/main.rs`) logs every gamepad that
  connects with its name and USB vendor/product ids, and warns once when a pad
  has been steering on the left stick alone for a sustained stretch without a
  single right-stick event ever arriving — the exact signature of this fault.
  The evidence logic is pure and unit-tested (8 tests).
- `README.md` gains a **Controllers** section recording the `D`-mode
  requirement, why `X` mode cannot work, and the `SDL_GAMECONTROLLERCONFIG`
  escape hatch.

### Acceptance

- [x] Root cause identified and explained rather than guessed at.
- [x] The `D`-mode requirement is documented where a player will find it.
- [x] A pad that loses a stick this way now announces itself in the log instead
      of failing silently.
- [x] `cargo test` green (14 passed, 8 new); `cargo check` adds no new warnings.

Not done, deliberately: nothing makes `X` mode playable. The right stick never
reaches the process, and the game needs both sticks to twist the paddle.

## Log

- 2026-08-25 status → ready (app)
- 2026-08-25 status → in-progress (agent)
- 2026-08-25 status → review (agent)
- 2026-08-25 status → done (app)
