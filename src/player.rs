use bullet::*;
use gun::*;
use gunship::*;
use gunship::input::*;
use gunship::math::*;
use gunship::resource::Mesh;
use gunship::transform::Transform;
use physics::*;

const ACCELERATION: f32 = 50.0;
const MAX_SPEED: f32 = 5.0;

#[derive(Debug)]
pub struct Player {
    pub root_transform: Transform,
    pub root_rigidbody: Rigidbody,
    pub gun_transform: Transform,
    pub gun_rigidbody: Rigidbody,
    pub gun_physics: GunPhysics,
    pub gun: Gun,

    pub pitch: f32,
    pub yaw: f32,
    pub bullet_offset: Vector3,

    pub bullet_mesh: Mesh,
}

impl Player {
    pub fn update(&mut self) {
        // Cache off the position and rotation and then drop the transform
        // so that we don't have multiple borrows of transform_manager.
        let (movement_x, movement_y) = input::mouse_delta();
        self.yaw += (-movement_x as f32) * PI * 0.1 * time::delta_f32();
        self.pitch += (-movement_y as f32) * PI * 0.1 * time::delta_f32();

        // Set orientation by applying yaw first, then pitch. If we do both at once (e.g.
        // `Orientation::from_eulers(pitch, yaw, 0.0)`) then pitch is applied first, which causes
        // pitch to invert with the player turns around.
        self.root_transform.set_orientation(
            Orientation::from_eulers(0.0, self.yaw, 0.0) + Orientation::from_eulers(self.pitch, 0.0, 0.0),
        );

        // Handle movement through root entity.
        {
            let mut velocity = self.root_rigidbody.velocity();

            // Calculate the forward and right vectors.
            let forward_dir: Vector3 = self.root_transform.forward().set_y(0.0).normalized();
            let right_dir = self.root_transform.right();

            // Move camera based on input.
            if input::key_down(ScanCode::W) {
                velocity += forward_dir * time::delta_f32() * ACCELERATION;
            }

            if input::key_down(ScanCode::S) {
                velocity -= forward_dir * time::delta_f32() * ACCELERATION;
            }

            if input::key_down(ScanCode::D) {
                velocity += right_dir * time::delta_f32() * ACCELERATION;
            }

            if input::key_down(ScanCode::A) {
                velocity -= right_dir * time::delta_f32() * ACCELERATION;
            }

            if input::key_down(ScanCode::E) {
                velocity += Vector3::up() * time::delta_f32() * ACCELERATION;
            }

            if input::key_down(ScanCode::Q) {
                velocity += Vector3::down() * time::delta_f32() * ACCELERATION;
            }

            // Clamp the velocity to the maximum speed.
            if velocity.magnitude() > MAX_SPEED {
                velocity = velocity.normalized() * MAX_SPEED;
            }

            self.root_rigidbody.set_velocity(velocity);
        };

        let position = self.gun_transform.position();
        let rotation = self.gun_transform.orientation();

        let rotation_matrix = Matrix3::from(rotation);
        let up_dir = rotation_matrix.y_part();
        let right_dir = rotation_matrix.y_part();
        let forward_dir = -rotation_matrix.z_part();

        if input::mouse_button_pressed(1) {
            self.gun.pull_hammer();
        }

        if input::mouse_button_pressed(0) {
            if self.gun.can_fire() {
                self.gun.fire();

                // TODO: Play audio on gunshot.
                // let mut audio_source = audio_manager.get_mut(player.gun_entity).unwrap();
                // audio_source.reset();
                // audio_source.play();

                let bullet_pos = position
                               + (self.bullet_offset.x * right_dir)
                               + (self.bullet_offset.y * up_dir)
                               + (self.bullet_offset.z * forward_dir);
                let mut bullet = Bullet::new(&self.bullet_mesh, bullet_pos, rotation);
                engine::run_each_frame(move || {
                    bullet.update();
                });

                self.gun_rigidbody.add_velocity(Vector3::new(0.0, 3.0, 10.0));
                self.gun_rigidbody.add_angular_velocity(Vector3::new(15.0 * PI, -8.0 * PI, 5.0 * PI));
            }
        }

        self.root_rigidbody.update(&mut self.root_transform);
        self.gun_physics.update_target(&self.root_transform);
        self.gun_physics.update(&mut self.gun_rigidbody, &self.gun_transform);
        self.gun_rigidbody.update(&mut self.gun_transform);
    }
}
