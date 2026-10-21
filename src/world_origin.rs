//! Minimales Floating Origin: der Render-Ursprung folgt der Kamera
//! (unverändert aus spcgame übernommen).
//!
//! Gemessenes Problem: bei 1,8·10⁶ Einheiten Entfernung wackelt eine einzige
//! f32-`clip_from_world`-Matrix um ~1,4 px pro Frame, 1 km über der Oberfläche.
//! Lösung: Absolute Positionen leben in f64 ([`AbsolutePosition`]);
//! Render-Transforms sind relativ zum [`WorldOrigin`], der zur Kamera
//! springt, wann immer sie weiter als [`RECENTER_DISTANCE`] von ihm entfernt
//! ist. Alles nahe der Kamera hat dadurch kleine, präzise f32-Koordinaten.

use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::transform::TransformSystems;

/// Abstand vom Ursprung, ab dem die Welt auf die Kamera rezentriert wird.
pub const RECENTER_DISTANCE: f32 = 1_000.0;

/// Ausführung des Rezentrierungs-Systems, damit andere Systeme sich davor-
/// oder dahinter einordnen können (z. B. der Himmel in `sky`).
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RecenterWorld;

pub struct WorldOriginPlugin;

impl Plugin for WorldOriginPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldOrigin>().add_systems(
            PostUpdate,
            recenter_on_camera
                .before(TransformSystems::Propagate)
                .in_set(RecenterWorld),
        );
    }
}

/// Absolute Position des Render-Ursprungs.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct WorldOrigin(pub DVec3);

impl WorldOrigin {
    /// Absolute Position von etwas, das bei `translation` gerendert wird.
    #[allow(dead_code)] // API für die kommenden Spielsysteme
    pub fn absolute(&self, translation: Vec3) -> DVec3 {
        self.0 + translation.as_dvec3()
    }
}

/// Exakte Position einer Wurzel-Entity in der Welt. Ihre `Transform`-
/// Translation wird daraus und vom aktuellen [`WorldOrigin`] abgeleitet.
#[derive(Component, Clone, Copy, Debug)]
pub struct AbsolutePosition(pub DVec3);

/// Render-Translation einer absoluten Position für einen Ursprung.
pub fn render_translation(absolute: DVec3, origin: DVec3) -> Vec3 {
    (absolute - origin).as_vec3()
}

fn recenter_on_camera(
    mut origin: ResMut<WorldOrigin>,
    mut camera: Single<&mut Transform, With<Camera3d>>,
    mut placed: Query<(&AbsolutePosition, &mut Transform), Without<Camera3d>>,
) {
    if camera.translation.length() > RECENTER_DISTANCE {
        origin.0 += camera.translation.as_dvec3();
        camera.translation = Vec3::ZERO;
    }
    // Billig für eine Handvoll Entities, und neue werden auch platziert.
    for (absolute, mut transform) in &mut placed {
        let translation = render_translation(absolute.0, origin.0);
        if transform.translation != translation {
            transform.translation = translation;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ein langer Flug hält die Kamera nahe dem Ursprung und jede Entity
    /// exakt dort, wo ihre absolute Position sie haben will.
    #[test]
    fn langer_flug_haelt_render_koordinaten_klein_und_konsistent() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(WorldOriginPlugin);
        let saturn = DVec3::new(312_567.0, 0.0, -1_772_646.0);
        let body = app
            .world_mut()
            .spawn((AbsolutePosition(saturn), Transform::default()))
            .id();
        let camera = app
            .world_mut()
            .spawn((Camera3d::default(), Transform::default()))
            .id();

        let step = Vec3::new(31.0, 0.5, -177.0);
        let frames = 10_000;
        for _ in 0..frames {
            app.world_mut()
                .get_mut::<Transform>(camera)
                .unwrap()
                .translation += step;
            app.update();
            let camera_translation = app.world().get::<Transform>(camera).unwrap().translation;
            assert!(camera_translation.length() <= RECENTER_DISTANCE);
        }

        let origin = app.world().resource::<WorldOrigin>().0;
        let camera_translation = app.world().get::<Transform>(camera).unwrap().translation;
        let travelled = origin + camera_translation.as_dvec3();
        let expected = step.as_dvec3() * frames as f64;
        assert!(
            (travelled - expected).length() < 1e-6,
            "{travelled} vs. {expected}"
        );

        let body_translation = app.world().get::<Transform>(body).unwrap().translation;
        assert_eq!(body_translation, render_translation(saturn, origin));
    }
}
