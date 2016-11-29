use gun::*;
use gunship::*;
use gunship::camera::Camera;
use gunship::input::*;
use gunship::math::*;
use gunship::mesh_renderer::MeshRenderer;
use gunship::resource::Mesh;
use gunship::transform::Transform;
use physics::*;
use std::sync::Arc;

const ACCELERATION: f32 = 50.0;
const MAX_SPEED: f32 = 5.0;

#[derive(Debug)]
pub struct Player {
    pub camera: Camera,
    pub transform: Transform,
    pub rigidbody: Rigidbody,

    pub gun: Revolver,
    pub gun_physics: GunPhysics,

    pub pitch: f32,
    pub yaw: f32,

    pub cartridge_mesh: Arc<Mesh>,
}

impl Player {
    pub fn update(&mut self) {
        // Cache off the position and rotation and then drop the transform
        // so that we don't have multiple borrows of transform_manager.
        let (movement_x, movement_y) = input::mouse_delta();
        self.yaw += (-movement_x as f32) * PI * 0.1 * time::delta_f32();
        self.pitch += (-movement_y as f32) * PI * 0.1 * time::delta_f32();
        self.pitch = self.pitch.clamp(-0.45 * PI, 0.45 * PI);

        // Set orientation by applying yaw first, then pitch. If we do both at once (e.g.
        // `Orientation::from_eulers(pitch, yaw, 0.0)`) then pitch is applied first, which causes
        // pitch to invert with the player turns around.
        self.transform.set_orientation(
            Orientation::from_eulers(0.0, self.yaw, 0.0) + Orientation::from_eulers(self.pitch, 0.0, 0.0),
        );

        // Handle movement through root entity.
        {
            let mut velocity = self.rigidbody.velocity();

            // Calculate the forward and right vectors.
            let forward_dir: Vector3 = self.transform.forward().set_y(0.0).normalized();
            let right_dir = self.transform.right();

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

            self.rigidbody.set_velocity(velocity);
        };

        self.rigidbody.update(&mut self.transform);
        self.gun_physics.update_target(&self.transform);
        self.gun_physics.update(&mut self.gun.rigidbody, &self.gun.transform);
        self.gun.rigidbody.update(&mut self.gun.transform);

        if input::mouse_scroll() != 0 {
            self.gun.rotate_cylinder(input::mouse_scroll() as isize);
        }

        if input::key_pressed(ScanCode::R) {
            // Create the cartridge.
            let mut cartridge_transform = Transform::new();
            cartridge_transform.set_scale(Vector3::new(0.01, 0.01, 0.03));

            let cartridge_renderer = MeshRenderer::new(&self.cartridge_mesh, &cartridge_transform);

            // TODO: Animate cartridge being inserted.
            // TODO: Animate failure when cartidge doesn't go in.
            let _ = self.gun.load_cartridge(Cartridge {
                transform: cartridge_transform,
                mesh_renderer: cartridge_renderer,

                has_fired: false,
            });
        }

        if input::mouse_button_pressed(1) {
            self.gun.pull_hammer();
        }

        if input::mouse_button_pressed(0) && self.gun.fire() {
            // Apply kickback animation.
            self.gun.rigidbody.add_velocity(Vector3::new(0.0, 3.0, 10.0));
            self.gun.rigidbody.add_angular_velocity(Vector3::new(15.0 * PI, -8.0 * PI, 5.0 * PI));
        }

        self.gun.update_transforms();
    }
}
