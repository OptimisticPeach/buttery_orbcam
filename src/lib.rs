use std::ops::Neg;

use bevy::{
    app::Plugin,
    input::{mouse::MouseWheel, ButtonInput},
    math::{Quat, Vec3},
    prelude::*,
    time::Time,
};
use buttery::{Rotate, TransformComponent, Translate};

#[derive(Default)]
pub struct OrbitCamPlugin(OrbitCamConfig);

impl Plugin for OrbitCamPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(self.0)
            .add_systems(Update, (update_orbitcams, OrbitCam::process_input));
    }
}

#[derive(Component)]
pub struct OrbitCam {
    pub up: TransformComponent<Rotate>,
    pub inclination: TransformComponent<Translate<f32>>,
    pub distance: TransformComponent<Translate<f32>>,
    pub target_height: TransformComponent<Translate<f32>>,
    pub min: TransformComponent<Translate<f32>>,
}

#[derive(Resource, Copy, Clone)]
pub struct OrbitCamConfig {
    pub forward: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub backward: KeyCode,

    pub cw: KeyCode,
    pub ccw: KeyCode,

    pub tilt_up: KeyCode,
    pub tilt_down: KeyCode,

    pub zoom_in: KeyCode,
    pub zoom_out: KeyCode,
}

impl Default for OrbitCamConfig {
    fn default() -> Self {
        OrbitCamConfig {
            forward: KeyCode::KeyW,
            left: KeyCode::KeyA,
            right: KeyCode::KeyD,
            backward: KeyCode::KeyS,
            cw: KeyCode::ArrowLeft,
            ccw: KeyCode::ArrowRight,
            tilt_up: KeyCode::ArrowUp,
            tilt_down: KeyCode::ArrowDown,
            zoom_in: KeyCode::ShiftLeft,
            zoom_out: KeyCode::Space,
        }
    }
}

fn update_orbitcams(mut query: Query<(&mut Transform, &mut OrbitCam)>, delta: Res<Time>) {
    let delta = delta.delta_seconds();

    for (mut transform, mut orbcam) in query.iter_mut() {
        let new_transform = orbcam.drive(delta);
        *transform = new_transform;
    }
}

impl OrbitCam {
    pub fn drive(&mut self, time: f32) -> Transform {
        let up = self.up.drive(time);
        let incl = self.inclination.drive(time);
        let dist = self.distance.drive(time);
        let height = self.target_height.drive(time);
        let min = self.min.drive(time);

        let arm = dist * Quat::from_rotation_x(-incl).mul_vec3(Vec3::Z);
        let mut pos = Vec3::Y * height + arm;
        let pos_len = pos.length();
        if pos_len < min {
            pos.y += min - pos_len;
        }
        let pos = up * pos;
        let rotation = up * Quat::from_rotation_x(-incl);

        Transform {
            rotation,
            translation: pos,
            scale: Vec3::ONE,
        }
    }

    pub fn process_input(
        mut cameras: Query<&mut OrbitCam>,
        keys: Res<ButtonInput<KeyCode>>,
        mut scroll: EventReader<MouseWheel>,
        config: Res<OrbitCamConfig>,
    ) {
        let (mut yaw, mut pitch) = (0.0, 0.0);

        if keys.pressed(config.tilt_up) {
            pitch -= 1.0;
        }

        if keys.pressed(config.tilt_down) {
            pitch += 1.0;
        }

        if keys.pressed(config.cw) {
            yaw += 1.0;
        }

        if keys.pressed(config.ccw) {
            yaw -= 1.0;
        }

        yaw *= 0.05;
        pitch *= 0.08;

        let mut delta_zoom = 0.0;

        for scroll_event in scroll.read() {
            match scroll_event.unit {
                bevy::input::mouse::MouseScrollUnit::Line => delta_zoom += scroll_event.y,
                bevy::input::mouse::MouseScrollUnit::Pixel => delta_zoom += scroll_event.y * 0.1,
            }
        }

        if keys.pressed(config.zoom_out) {
            delta_zoom += 0.2;
        } else if keys.pressed(config.zoom_in) {
            delta_zoom -= 0.2;
        }

        let (mut up, mut right) = (0.0, 0.0);

        if keys.pressed(config.right) {
            right -= 1.0;
        }

        if keys.pressed(config.left) {
            right += 1.0;
        }

        if keys.pressed(config.forward) {
            up -= 1.0;
        }

        if keys.pressed(config.backward) {
            up += 1.0;
        }

        for mut camera in cameras.iter_mut() {
            camera.distance.target *= 1.0 + delta_zoom * 0.2;
            camera.up.target = (camera.up.target * Quat::from_rotation_y(yaw)).normalize();
            camera.inclination.target =
                (camera.inclination.target + pitch).clamp(0.0, std::f32::consts::FRAC_PI_2);

            let want_distance = camera.distance.current;

            let speed_scl = 0.04 * ((want_distance.sqrt().neg().exp() + 1.0).recip() * 2.0 - 1.0);

            let axis = Vec3::Y.cross(Vec3::new(right * -speed_scl, 0.0, up * speed_scl));
            let len = axis.length();
            if len == 0.0 {
                return;
            }

            camera.up.target = (camera.up.target * Quat::from_scaled_axis(axis)).normalize();
        }
    }

    pub fn from_radius(radius: f32) -> Self {
        OrbitCam {
            up: TransformComponent::new_rotate(Quat::IDENTITY),
            inclination: TransformComponent::new_angle(0.0),
            distance: TransformComponent::new_zoom(4.0),
            target_height: TransformComponent::new(0.01, radius),
            min: TransformComponent::new(0.01, radius),
        }
    }
}

impl Default for OrbitCam {
    fn default() -> Self {
        OrbitCam {
            up: TransformComponent::new_rotate(Quat::IDENTITY),
            inclination: TransformComponent::new_angle(0.0),
            distance: TransformComponent::new_zoom(4.0),
            target_height: TransformComponent::new(0.01, 1.0),
            min: TransformComponent::new(0.01, 1.0),
        }
    }
}
