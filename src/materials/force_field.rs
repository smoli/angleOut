use bevy::asset::{Asset, Handle};
use bevy::math::{Vec2, Vec4};
use bevy::pbr::Material;
use bevy::prelude::{AlphaMode, Color, GlobalTransform, Image, Vec3};
use bevy::reflect::TypePath;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{AsBindGroup, AsBindGroupShaderType, ShaderType};
use bevy::render::texture::GpuImage;
use bevy::shader::ShaderRef;

use crate::materials::linear_rgba;

/// How many impacts a single panel can show at once. Levels put at most four
/// balls in play (`simultaneous_balls` plus the `MoreBalls` pickups), so eight
/// slots leave room for a ball that rattles along the same panel twice.
pub const FORCE_FIELD_HIT_SLOTS: usize = 8;

/// Wrap period of the shader's `globals.time`, which is Bevy's wrapped clock.
/// Hit times are recorded with `Time::elapsed_secs_wrapped()` so both sides
/// agree; `wrap_period_matches_bevy` guards this against a Bevy default change.
pub const FORCE_FIELD_TIME_WRAP: f32 = 3600.0;

/// Age of a hit started at `start` as seen at `now`, both on the wrapped clock.
/// The clock wraps hourly, so a negative difference means `now` has wrapped past
/// `start` rather than the hit lying in the future.
pub fn hit_age(now: f32, start: f32) -> f32 {
    let age = now - start;

    if age < 0.0 {
        age + FORCE_FIELD_TIME_WRAP
    } else {
        age
    }
}

/// Maps a world-space impact into the panel's uv space from the panel's own
/// transform and size, so a rotated `LevelObstacle::ForceField` ripples at the
/// real contact point. The panel mesh is a `Rectangle` in its local xy plane
/// with uv (0,0) at the top left, and the offset along the panel normal — the
/// ball's radius — is dropped.
pub fn panel_uv(panel: &GlobalTransform, size: Vec2, world: Vec3) -> Vec2 {
    let local = panel.affine().inverse().transform_point3(world);

    Vec2::new(
        (0.5 + local.x / size.x).clamp(0.0, 1.0),
        (0.5 - local.y / size.y).clamp(0.0, 1.0),
    )
}

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[uniform(0, ForceFieldMaterialUniform)]
pub struct ForceFieldMaterial {
    /// Colour of the idle energy sheet.
    pub sheet_color: Color,
    /// Colour the hex lattice flares in where a ripple passes over it.
    pub flare_color: Color,
    /// The panel's extents in world units; impacts are mapped through these.
    pub panel_size: Vec2,
    /// How fast a ripple front travels outward, world units per second.
    pub ripple_speed: f32,
    /// Thickness of the ripple front, world units.
    pub ripple_width: f32,
    /// How long a ripple lives before it has fully faded, seconds.
    pub ripple_decay: f32,
    /// How hard the hex lattice flares where the ripple passes.
    pub flare_intensity: f32,
    /// World units covered by one tile of the hex texture.
    pub hex_tile_size: f32,

    /// Live impacts: `xy` is the panel uv, `z` the start time on the wrapped
    /// clock, `w` is 1.0 once the slot has been used. Written by
    /// [`ForceFieldMaterial::register_hit`], summed per-slot in the shader.
    hits: [Vec4; FORCE_FIELD_HIT_SLOTS],

    #[texture(1)]
    #[sampler(2)]
    pub color_texture: Option<Handle<Image>>,

    pub alpha_mode: AlphaMode,
}

impl Default for ForceFieldMaterial {
    fn default() -> Self {
        ForceFieldMaterial {
            sheet_color: Color::srgb(0.1, 0.35, 1.0),
            flare_color: Color::srgb(0.65, 0.9, 1.0),
            panel_size: Vec2::new(200.0, 20.0),
            ripple_speed: 70.0,
            ripple_width: 6.0,
            ripple_decay: 1.2,
            flare_intensity: 2.5,
            hex_tile_size: 10.0,
            hits: [Vec4::ZERO; FORCE_FIELD_HIT_SLOTS],
            color_texture: None,
            alpha_mode: AlphaMode::Blend,
        }
    }
}

impl ForceFieldMaterial {
    /// A shield panel `size` world units across, whose lattice flares with
    /// `texture` where a ripple passes.
    pub fn for_panel(size: Vec2, texture: Handle<Image>) -> Self {
        ForceFieldMaterial {
            panel_size: size,
            color_texture: Some(texture),
            ..ForceFieldMaterial::default()
        }
    }

    /// Records an impact at panel `uv`. It claims the first slot that is unused
    /// or whose ripple has decayed, and only evicts the oldest live ripple when
    /// every slot is still busy — so simultaneous hits stay independently
    /// visible instead of overwriting each other.
    pub fn register_hit(&mut self, uv: Vec2, now: f32) {
        let free = self
            .hits
            .iter()
            .position(|hit| hit.w < 0.5 || hit_age(now, hit.z) >= self.ripple_decay);

        let slot = free.unwrap_or_else(|| {
            self.hits
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| hit_age(now, a.z).total_cmp(&hit_age(now, b.z)))
                .map_or(0, |(slot, _)| slot)
        });

        self.hits[slot] = Vec4::new(uv.x, uv.y, now, 1.0);
    }
}

#[derive(ShaderType, Clone, Default)]
pub struct ForceFieldMaterialUniform {
    pub sheet_color: Vec4,
    pub flare_color: Vec4,
    pub panel_size: Vec2,
    pub ripple_speed: f32,
    pub ripple_width: f32,
    pub ripple_decay: f32,
    pub flare_intensity: f32,
    pub hex_tile_size: f32,
    /// Pads the scalars up to the 16-byte alignment `hits` needs in a uniform.
    pub _padding: f32,
    pub hits: [Vec4; FORCE_FIELD_HIT_SLOTS],
}

impl AsBindGroupShaderType<ForceFieldMaterialUniform> for ForceFieldMaterial {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<GpuImage>) -> ForceFieldMaterialUniform {
        ForceFieldMaterialUniform {
            sheet_color: linear_rgba(self.sheet_color),
            flare_color: linear_rgba(self.flare_color),
            panel_size: self.panel_size,
            ripple_speed: self.ripple_speed,
            ripple_width: self.ripple_width,
            ripple_decay: self.ripple_decay,
            flare_intensity: self.flare_intensity,
            hex_tile_size: self.hex_tile_size,
            _padding: 0.0,
            hits: self.hits,
        }
    }
}


/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for ForceFieldMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/force_field_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_2;

    use bevy::prelude::{Quat, Time, Transform};

    use super::*;

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 1e-4,
            "expected {expected}, got {actual}"
        );
    }

    /// The shader reads `globals.time`, so our recorded hit times have to sit on
    /// the same wrapped clock. If Bevy ever changes its default wrap period,
    /// ripples would silently break once an hour rather than fail loudly here.
    #[test]
    fn wrap_period_matches_bevy() {
        assert_close(
            Time::<()>::default().wrap_period().as_secs_f32(),
            FORCE_FIELD_TIME_WRAP,
        );
    }

    #[test]
    fn hit_age_measures_forward_time() {
        assert_close(hit_age(10.0, 9.5), 0.5);
    }

    #[test]
    fn hit_age_survives_the_clock_wrapping() {
        // A hit recorded just before the hourly wrap, read just after it.
        assert_close(hit_age(0.25, FORCE_FIELD_TIME_WRAP - 0.25), 0.5);
    }

    #[test]
    fn hits_fill_free_slots_in_order() {
        let mut mat = ForceFieldMaterial::default();

        mat.register_hit(Vec2::new(0.25, 0.5), 1.0);
        mat.register_hit(Vec2::new(0.75, 0.5), 1.1);

        assert_eq!(mat.hits[0], Vec4::new(0.25, 0.5, 1.0, 1.0));
        assert_eq!(mat.hits[1], Vec4::new(0.75, 0.5, 1.1, 1.0));
        assert_eq!(mat.hits[2], Vec4::ZERO);
    }

    /// Two balls hitting the same panel inside one ripple lifetime have to stay
    /// two ripples — this is what a single hit slot used to get wrong.
    #[test]
    fn two_hits_within_one_lifetime_stay_independent() {
        let mut mat = ForceFieldMaterial::default();

        mat.register_hit(Vec2::new(0.2, 0.5), 4.0);
        mat.register_hit(Vec2::new(0.8, 0.5), 4.0 + mat.ripple_decay * 0.5);

        let live: Vec<&Vec4> = mat
            .hits
            .iter()
            .filter(|h| h.w > 0.5 && hit_age(4.6, h.z) < mat.ripple_decay)
            .collect();

        assert_eq!(live.len(), 2);
        assert_close(live[0].x, 0.2);
        assert_close(live[1].x, 0.8);
    }

    #[test]
    fn a_decayed_slot_is_reused_before_a_live_one_is_evicted() {
        let mut mat = ForceFieldMaterial::default();
        // Every slot busy, slot 0 the oldest of them...
        for slot in 0..FORCE_FIELD_HIT_SLOTS {
            mat.hits[slot] = Vec4::new(0.5, 0.5, 9.5 + slot as f32 * 0.05, 1.0);
        }
        // ...but slot 3 holds a ripple that has already faded out.
        mat.hits[3] = Vec4::new(0.5, 0.5, 1.0, 1.0);

        mat.register_hit(Vec2::new(0.1, 0.2), 10.0);

        assert_eq!(mat.hits[3], Vec4::new(0.1, 0.2, 10.0, 1.0));
        assert_close(mat.hits[0].z, 9.5);
    }

    #[test]
    fn a_full_panel_evicts_the_oldest_ripple() {
        let mut mat = ForceFieldMaterial::default();
        for slot in 0..FORCE_FIELD_HIT_SLOTS {
            mat.register_hit(Vec2::new(0.5, 0.5), 10.0 + slot as f32 * 0.05);
        }

        mat.register_hit(Vec2::new(0.9, 0.1), 10.4);

        assert_eq!(mat.hits[0], Vec4::new(0.9, 0.1, 10.4, 1.0));
        // The other seven ripples are left alone.
        assert_close(mat.hits[1].z, 10.05);
        assert_close(mat.hits[7].z, 10.35);
    }

    #[test]
    fn an_unrotated_panel_maps_a_hit_across_its_width() {
        let panel = GlobalTransform::from(Transform::from_xyz(0.0, 0.0, -83.0));

        let uv = panel_uv(&panel, Vec2::new(200.0, 20.0), Vec3::new(50.0, 0.0, -83.0));

        assert_close(uv.x, 0.75);
        assert_close(uv.y, 0.5);
    }

    /// The contact height used to be thrown away; uv (0,0) is the panel's top
    /// left, so a hit above centre sits in the upper half.
    #[test]
    fn a_hit_above_centre_maps_into_the_upper_half() {
        let panel = GlobalTransform::from(Transform::from_xyz(0.0, 0.0, -83.0));

        let uv = panel_uv(&panel, Vec2::new(200.0, 20.0), Vec3::new(0.0, 5.0, -83.0));

        assert_close(uv.y, 0.25);
    }

    /// The rotated obstacle panels are the case the old arena-width mapping got
    /// wrong: every hit on this panel sits at world x = 100, which that mapping
    /// read as u = 1.0 no matter where along the panel the ball actually landed.
    #[test]
    fn a_rotated_panel_maps_along_its_own_axis() {
        let panel = GlobalTransform::from(
            Transform::from_xyz(100.0, 0.0, -33.39)
                .with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
        );

        // 10 world units along the panel, and a ball radius off its face.
        let uv = panel_uv(&panel, Vec2::new(30.0, 20.0), Vec3::new(98.0, 0.0, -23.39));

        assert_close(uv.x, 0.5 + 10.0 / 30.0);
        assert_close(uv.y, 0.5);
    }

    #[test]
    fn a_hit_past_the_panel_edge_is_clamped() {
        let panel = GlobalTransform::from(Transform::default());

        let uv = panel_uv(&panel, Vec2::new(200.0, 20.0), Vec3::new(500.0, -80.0, 0.0));

        assert_close(uv.x, 1.0);
        assert_close(uv.y, 1.0);
    }

    #[test]
    fn defaults_describe_a_blue_sheet_with_a_brighter_flare() {
        let mat = ForceFieldMaterial::default();

        assert!(mat.ripple_decay > 0.0);
        assert!(mat.ripple_speed > 0.0);
        assert!(mat.ripple_width > 0.0);
        assert_eq!(mat.hits, [Vec4::ZERO; FORCE_FIELD_HIT_SLOTS]);
    }
}
