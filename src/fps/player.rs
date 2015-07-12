use std::f32::consts::PI;

use gunship::*;
use fps::physics::*;
use fps::bullet::*;

#[derive(Debug, Clone, Copy)]
pub struct PlayerMoveSystem {
    pub root: Entity,
    pub camera: Entity,
    pub gun_entity: Entity,
    pub bullet_offset: Vector3,
    pub gun_alarm: Option<AlarmID>,
}

const ACCELERATION: f32 = 50.0;
const MAX_SPEED: f32 = 5.0;

impl System for PlayerMoveSystem {
    fn update(&mut self, scene: &Scene, delta: f32) {
        {
            let mut alarm_manager = scene.get_manager_mut::<AlarmManager>();
            if scene.input.key_down(ScanCode::W)
            || scene.input.key_down(ScanCode::A)
            || scene.input.key_down(ScanCode::S)
            || scene.input.key_down(ScanCode::D) {
                if self.gun_alarm.is_none() {
                    let alarm_id = alarm_manager.assign_repeating(self.gun_entity, 0.5, |scene, entity| {
                        let rigidbody_manager = scene.get_manager::<RigidbodyManager>();

                        let mut rigidbody = rigidbody_manager.get_mut(entity);
                        rigidbody.add_velocity(Vector3::new(0.0, -0.25, 0.0));
                        rigidbody.add_angular_velocity(Vector3::new(-0.5 * PI, 0.0, 0.0));
                    });
                    self.gun_alarm = Some(alarm_id);
                }
            } else if self.gun_alarm.is_some() {
                let alarm_id = self.gun_alarm.unwrap();
                alarm_manager.cancel(alarm_id);
                self.gun_alarm = None;
            }
        }

        // Cache off the position and rotation and then drop the transform
        // so that we don't have multiple borrows of transform_manager.
        let (position, rotation) = {
            let transform_manager = scene.get_manager::<TransformManager>();
            let rigidbody_manager = scene.get_manager::<RigidbodyManager>();

            let (movement_x, movement_y) = scene.input.mouse_delta();

            // Handle movement through root entity.
            // The root entity handles all translation as well as rotation around the Y axis.
            {
                let mut transform = transform_manager.get_mut(self.root);
                let mut rigidbody = rigidbody_manager.get_mut(self.root);

                let rotation = transform.rotation();
                let mut velocity = rigidbody.velocity();

                transform.set_rotation(Quaternion::from_eulers(0.0, (-movement_x as f32) * PI * 0.001, 0.0) * rotation);

                // Calculate the forward and right vectors.
                // TODO: Directly retrieve local axis from transform without going through rotation matrix.
                let forward_dir = -transform.rotation().as_matrix().z_part();
                let right_dir = transform.rotation().as_matrix().x_part();

                // Move camera based on input.
                if scene.input.key_down(ScanCode::W) {
                    velocity = velocity + forward_dir * delta * ACCELERATION;
                }

                if scene.input.key_down(ScanCode::S) {
                    velocity = velocity - forward_dir * delta * ACCELERATION;
                }

                if scene.input.key_down(ScanCode::D) {
                    velocity = velocity + right_dir * delta * ACCELERATION;
                }

                if scene.input.key_down(ScanCode::A) {
                    velocity = velocity - right_dir * delta * ACCELERATION;
                }

                if scene.input.key_down(ScanCode::E) {
                    velocity = velocity + Vector3::up() * delta * ACCELERATION;
                }

                if scene.input.key_down(ScanCode::Q) {
                    velocity = velocity + Vector3::down() * delta * ACCELERATION;
                }

                // Clamp the velocity to the maximum speed.
                if velocity.magnitude() > MAX_SPEED {
                    velocity = velocity.normalized() * MAX_SPEED;
                }

                rigidbody.set_velocity(velocity);
            };

            {
                let mut camera_transform = transform_manager.get_mut(self.camera);
                let rotation = camera_transform.rotation();

                // Apply a rotation to the camera based on mouse movement.
                camera_transform.set_rotation(
                    Quaternion::from_eulers((-movement_y as f32) * PI * 0.001, 0.0, 0.0)
                  * rotation);
            }

            transform_manager.update_single(self.gun_entity);
            let gun_transform = transform_manager.get(self.gun_entity);

            (gun_transform.position_derived(), gun_transform.rotation_derived())
        };

        let up_dir = rotation.as_matrix().y_part();
        let right_dir = rotation.as_matrix().y_part();
        let forward_dir = -rotation.as_matrix().z_part();

        // Maybe shoot some bullets?
        if scene.input.mouse_button_pressed(0) {
            let audio_manager = scene.get_manager::<AudioSourceManager>();
            let rigidbody_manager = scene.get_manager::<RigidbodyManager>();

            let mut audio_source = audio_manager.get_mut(self.gun_entity);
            audio_source.reset();
            audio_source.play();

            let bullet_pos = position
                           + (self.bullet_offset.x * right_dir)
                           + (self.bullet_offset.y * up_dir)
                           + (self.bullet_offset.z * forward_dir);
            Bullet::new(scene, bullet_pos, rotation);

            let mut rigidbody = rigidbody_manager.get_mut(self.gun_entity);
            rigidbody.add_velocity(Vector3::new(0.0, 3.0, 10.0));
            rigidbody.add_angular_velocity(Vector3::new(15.0 * PI, -8.0 * PI, 5.0 * PI));
        }
    }
}
