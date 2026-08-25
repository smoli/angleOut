# angleOut
Breakout Clone with a Twist

## Controllers

The paddle is twisted with *both* sticks, so a pad that only delivers one of
them is effectively unplayable.

- **PlayStation DualSense** — works as-is.
- **8BitDo Pro 2** — set the mode switch on the back of the pad to **`D`**
  (D-input). In `D` mode it works exactly like the DualSense.

  In `X` (X-input) mode on macOS the right stick does nothing and the left
  stick's Y axis is inverted. That is not fixable from inside the game: `gilrs`
  resolves macOS HID axes through a fixed table that only covers the `X`, `Y`,
  `Z` and `Rz` usages, and in `X` mode the pad puts its right stick on `Rx`/`Ry`
  instead. Those arrive as `Axis::Unknown` and `bevy_gilrs` drops them before
  Bevy ever sees them. The inverted Y comes from the same place: gilrs always
  applies its macOS "Y points downwards" correction, which X-input does not
  need.

If a pad misbehaves, run the game from a terminal — every gamepad that connects
is logged with its name and USB vendor/product ids, which is what you need to
look it up in the [SDL controller database][sdldb]. A custom mapping can be
supplied without rebuilding, via the `SDL_GAMECONTROLLERCONFIG` environment
variable that gilrs reads on startup. Note that gilrs parses but ignores the
SDL `~` (invert) suffix, so a mapping cannot be used to flip an axis.

[sdldb]: https://github.com/mdqinc/SDL_GameControllerDB
