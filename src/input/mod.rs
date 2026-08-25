//! Tells the player which gamepad the game is actually seeing, and warns when a
//! pad arrives with a stick the input stack cannot deliver.
//!
//! Background (gello card i0001). The paddle is steered with both sticks, so a
//! pad that only delivers one is unplayable — but from inside the game the
//! failure looks like nothing at all: the axis events simply never arrive.
//!
//! On macOS `gilrs` resolves HID axes through a fixed table that only knows the
//! `X`, `Y`, `Z` and `Rz` usages. A pad that puts its right stick on `Rx`/`Ry`
//! produces `Axis::Unknown` events, and `bevy_gilrs` drops those in
//! `convert_axis` before they ever reach Bevy. That is exactly what an 8BitDo
//! Pro 2 does in `X` (X-input) mode: right stick dead, and the left stick's `Y`
//! inverted on top, because gilrs' unconditional macOS `IS_Y_AXIS_REVERSED`
//! flip assumes the HID "down is positive" convention that X-input does not
//! follow. In `D` mode the same pad lands on `X`/`Y`/`Z`/`Rz` and works.
//!
//! None of that is recoverable on our side of `bevy_gilrs`, so the best this
//! module can do is notice the silence and say something useful about it.

use bevy::app::{App, Plugin, Update};
use bevy::input::gamepad::{
    GamepadConnection, GamepadConnectionEvent, RawGamepadAxisChangedEvent,
};
use bevy::prelude::{info, warn, Entity, GamepadAxis, MessageReader, ResMut, Resource};
use std::collections::HashMap;

/// Deflection below this is stick drift or a thumb resting on the cap, not the
/// player actually steering.
const STICK_ACTIVITY_THRESHOLD: f32 = 0.5;

/// How many deflected left-stick samples to collect before concluding that a
/// silent right stick is a mapping problem and not just a stick nobody has
/// touched yet. A stick sweep emits events continuously, so this is a second or
/// two of real steering — long enough that a player who simply favours the left
/// stick for a moment is not accused of having a broken pad.
const LEFT_STICK_SAMPLES_BEFORE_ADVICE: u32 = 90;

/// What to tell a player whose right stick never shows up.
const RIGHT_STICK_MISSING_ADVICE: &str =
    "This gamepad's right stick is not reaching the game - only the left stick is. On an 8BitDo \
     Pro 2, set the mode switch on the back to `D`: in `X` mode the right stick sits on HID axes \
     that gilrs does not read, and the left stick's Y axis arrives inverted as well.";

/// Per-gamepad evidence about which sticks are actually delivering events.
#[derive(Default, Debug)]
struct StickWatch {
    left_stick_samples: u32,
    seen_right_stick: bool,
    advised: bool,
}

impl StickWatch {
    /// Fold one raw axis event into the evidence.
    fn note_axis(&mut self, axis: GamepadAxis, value: f32) {
        match axis {
            GamepadAxis::LeftStickX | GamepadAxis::LeftStickY => {
                if value.abs() >= STICK_ACTIVITY_THRESHOLD {
                    self.left_stick_samples = self.left_stick_samples.saturating_add(1);
                }
            }
            // Any right-stick event at all - even a centring one - proves the
            // axis is mapped, which is the only thing being tested here.
            GamepadAxis::RightStickX | GamepadAxis::RightStickY => self.seen_right_stick = true,
            _ => {}
        }
    }

    /// `true` exactly once, the first time the evidence says the right stick is
    /// never going to arrive. Latches, so the warning is not repeated per frame.
    fn take_advice(&mut self) -> bool {
        if self.advised
            || self.seen_right_stick
            || self.left_stick_samples < LEFT_STICK_SAMPLES_BEFORE_ADVICE
        {
            return false;
        }

        self.advised = true;
        true
    }
}

/// Renders an optional USB id the way the SDL controller database writes them,
/// so a reported value can be grepped for directly.
fn format_usb_id(id: Option<u16>) -> String {
    id.map_or_else(|| "unknown".to_string(), |id| format!("{id:#06x}"))
}

#[derive(Resource, Default)]
struct GamepadWatch {
    pads: HashMap<Entity, StickWatch>,
}

pub struct InputDiagnosticsPlugin;

impl Plugin for InputDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GamepadWatch>()
            .add_systems(Update, (report_connections, watch_sticks));
    }
}

/// Logs the identity of every pad that connects. A bug report that says "the
/// controls do not work" is unactionable without this; one that carries the
/// name and USB ids can be looked up in the SDL controller database directly.
fn report_connections(
    mut events: MessageReader<GamepadConnectionEvent>,
    mut watch: ResMut<GamepadWatch>,
) {
    for event in events.read() {
        match &event.connection {
            GamepadConnection::Connected {
                name,
                vendor_id,
                product_id,
            } => {
                info!(
                    "Gamepad connected: {:?} (vendor {}, product {})",
                    name,
                    format_usb_id(*vendor_id),
                    format_usb_id(*product_id)
                );
                watch.pads.insert(event.gamepad, StickWatch::default());
            }
            GamepadConnection::Disconnected => {
                watch.pads.remove(&event.gamepad);
            }
        }
    }
}

fn watch_sticks(
    mut events: MessageReader<RawGamepadAxisChangedEvent>,
    mut watch: ResMut<GamepadWatch>,
) {
    for event in events.read() {
        let Some(pad) = watch.pads.get_mut(&event.gamepad) else {
            continue;
        };

        pad.note_axis(event.axis, event.value);

        if pad.take_advice() {
            warn!("{}", RIGHT_STICK_MISSING_ADVICE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{format_usb_id, StickWatch, LEFT_STICK_SAMPLES_BEFORE_ADVICE};

    use bevy::prelude::GamepadAxis;

    /// Steer the left stick hard for `samples` events.
    fn steer_left(watch: &mut StickWatch, samples: u32) {
        for _ in 0..samples {
            watch.note_axis(GamepadAxis::LeftStickY, 0.9);
        }
    }

    #[test]
    fn says_nothing_about_a_pad_nobody_has_touched() {
        let mut watch = StickWatch::default();

        assert!(!watch.take_advice());
    }

    #[test]
    fn says_nothing_while_the_evidence_is_still_thin() {
        let mut watch = StickWatch::default();

        steer_left(&mut watch, LEFT_STICK_SAMPLES_BEFORE_ADVICE - 1);

        assert!(!watch.take_advice());
    }

    #[test]
    fn advises_once_the_left_stick_has_carried_the_whole_session_alone() {
        let mut watch = StickWatch::default();

        steer_left(&mut watch, LEFT_STICK_SAMPLES_BEFORE_ADVICE);

        assert!(watch.take_advice());
    }

    #[test]
    fn advises_only_once() {
        let mut watch = StickWatch::default();

        steer_left(&mut watch, LEFT_STICK_SAMPLES_BEFORE_ADVICE * 2);

        assert!(watch.take_advice());
        assert!(!watch.take_advice(), "the warning must not repeat every frame");
    }

    #[test]
    fn a_working_right_stick_silences_the_advice_forever() {
        let mut watch = StickWatch::default();

        // A single centring event is enough to prove the axis is mapped.
        watch.note_axis(GamepadAxis::RightStickX, 0.0);
        steer_left(&mut watch, LEFT_STICK_SAMPLES_BEFORE_ADVICE * 2);

        assert!(!watch.take_advice());
    }

    #[test]
    fn resting_thumbs_and_stick_drift_are_not_steering() {
        let mut watch = StickWatch::default();

        for _ in 0..LEFT_STICK_SAMPLES_BEFORE_ADVICE * 2 {
            watch.note_axis(GamepadAxis::LeftStickX, 0.2);
        }

        assert!(!watch.take_advice());
    }

    #[test]
    fn triggers_and_exotic_axes_do_not_count_as_stick_input() {
        let mut watch = StickWatch::default();

        for _ in 0..LEFT_STICK_SAMPLES_BEFORE_ADVICE * 2 {
            watch.note_axis(GamepadAxis::LeftZ, 1.0);
            watch.note_axis(GamepadAxis::Other(7), 1.0);
        }

        assert!(!watch.take_advice());
    }

    #[test]
    fn usb_ids_are_rendered_the_way_the_sdl_database_writes_them() {
        assert_eq!(format_usb_id(Some(0x2dc8)), "0x2dc8");
        assert_eq!(format_usb_id(Some(0x045e)), "0x045e");
        assert_eq!(format_usb_id(None), "unknown");
    }
}
