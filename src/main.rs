extern crate gunship;

mod bullet;

use std::f32::consts::PI;

use gunship::*;
use gunship::ScanCode::*;

use bullet::{Bullet, BulletManager, BulletSystem};

fn main() {
    let mut engine = Engine::new();

    let (root_entity, camera_entity, gun_entity) = scene_setup(engine.scene_mut());

    engine.register_system(PlayerMoveSystem {
        root: root_entity,
        camera: camera_entity,
        gun_entity: gun_entity,
        bullet_offset: Vector3::new(0.0, 0.04, 0.2),
    });
    engine.register_system(GunPhysicsSystem);
    engine.register_system(BulletSystem);

    engine.main_loop();
}

fn scene_setup(scene: &mut Scene) -> (Entity, Entity, Entity) {
    scene.register_manager(BulletManager::new());
    scene.register_manager(GunPhysicsManager::new());

    fn create_light(scene: &Scene, position: Point) -> Entity {
        let mut transform_manager = scene.get_manager_mut::<TransformManager>();
        let mut light_manager = scene.get_manager_mut::<LightManager>();
        let mut mesh_manager = scene.get_manager_mut::<MeshManager>();

        let light_entity = scene.create_entity();
        let mut transform = transform_manager.assign(light_entity);
        transform.set_position(position);
        transform.set_scale(Vector3::new(0.1, 0.1, 0.1));
        light_manager.assign(
            light_entity,
            Light::Point(PointLight {
                position: Point::origin()
            }));
        mesh_manager.assign(light_entity, "meshes/cube.dae");

        light_entity
    };
    create_light(scene, Point::new(-1.0, -1.5, 0.0));
    create_light(scene, Point::new(-1.0, 1.5, 0.0));

    let mut transform_manager = scene.get_manager_mut::<TransformManager>();
    let mut mesh_manager = scene.get_manager_mut::<MeshManager>();
    let mut camera_manager = scene.get_manager_mut::<CameraManager>();
    let mut audio_manager = scene.get_manager_mut::<AudioSourceManager>();

    let root_entity = {
        let entity = scene.create_entity();
        let mut transform = transform_manager.assign(entity);
        transform.set_position(Point::new(0.0, 0.0, 0.0));
        entity
    };
    println!("root entity: {:?}", root_entity);

    // Create camera.
    let camera_entity = {
        let camera_entity = scene.create_entity();
        transform_manager.assign(camera_entity);
        camera_manager.assign(
            camera_entity,
            Camera::new(
                PI / 3.0,
                1.0,
                0.001,
                100.0));

        camera_entity
    };
    println!("camera entity: {:?}", camera_entity);

    transform_manager.set_child(root_entity, camera_entity);

    // Create gun root
    let gun_root = {
        let gun_root = scene.create_entity();
        let mut gun_root_transform = transform_manager.assign(gun_root);
        gun_root_transform.set_position(Point::new(0.1, -0.1, -0.3));

        gun_root
    };
    transform_manager.set_child(camera_entity, gun_root);

    // Create gun mesh.
    let gun_entity = {
        let gun_entity = scene.create_entity();
        transform_manager.assign(gun_entity);

        mesh_manager.assign(gun_entity, "meshes/gun_small.dae");
        audio_manager.assign(gun_entity, "audio/Shotgun_Blast-Jim_Rogers-1914772763.wav");

        gun_entity
    };
    println!("gun entity: {:?}", gun_entity);

    // Make gun a child of the camera.
    transform_manager.set_child(gun_root, gun_entity);

    // Create static gun and bullet meshes.
    {
        let static_gun_entity = scene.create_entity();
        let static_bullet_entity = scene.create_entity();

        {
            let mut gun_transform = transform_manager.assign(static_gun_entity);
            gun_transform.set_position(Point::new(0.0, 0.0, -1.0));
        }

        {
            let mut bullet_transform = transform_manager.assign(static_bullet_entity);
            bullet_transform.set_position(Point::new(-1.0, 0.0, 0.0));
        }

        mesh_manager.assign(static_gun_entity, "meshes/gun_small.dae");
        mesh_manager.assign(static_bullet_entity, "meshes/cube.dae");
    }

    // Add gun animation manager to player gun.
    {
        let mut gun_animation_manager = scene.get_manager_mut::<GunPhysicsManager>();

        gun_animation_manager.assign(gun_entity, GunPhysics {
            mass: 1.0,
            rotational_inertia: 1.0,

            spring_constant: 500.0,
            angular_spring: 400.0,

            damping: 10.0,
            angular_damping: 10.0,

            velocity: Vector3::zero(),
            angular_velocity: Vector3::zero(),
        });
    }

    (root_entity, camera_entity, gun_entity)
}

#[derive(Debug, Clone, Copy)]
struct PlayerMoveSystem {
    root: Entity,
    camera: Entity,
    gun_entity: Entity,
    bullet_offset: Vector3,
}

impl System for PlayerMoveSystem {
    fn update(&mut self, scene: &mut Scene, delta: f32) {
        // Cache off the position and rotation and then drop the transform
        // so that we don't have multiple borrows of transform_manager.
        let (position, rotation) = {
            let transform_manager = scene.get_manager::<TransformManager>();

            let (movement_x, movement_y) = scene.input.mouse_delta();

            {
                let mut root_transform = transform_manager.get_mut(self.root);
                let position = root_transform.position();
                let rotation = root_transform.rotation();

                root_transform.set_rotation(Quaternion::from_eulers(0.0, (-movement_x as f32) * PI * 0.001, 0.0) * rotation);

                // Calculate the forward and right vectors.
                let forward_dir = -root_transform.rotation().as_matrix().z_part();
                let right_dir = root_transform.rotation().as_matrix().x_part();

                // Move camera based on input.
                if scene.input.key_down(W) {
                    root_transform.set_position(position + forward_dir * delta);
                }

                if scene.input.key_down(S) {
                    root_transform.set_position(position - forward_dir * delta);
                }

                if scene.input.key_down(D) {
                    root_transform.set_position(position + right_dir * delta);
                }

                if scene.input.key_down(A) {
                    root_transform.set_position(position - right_dir * delta);
                }

                if scene.input.key_down(E) {
                    root_transform.set_position(position + Vector3::up() * delta);
                }

                if scene.input.key_down(Q) {
                    root_transform.set_position(position + Vector3::down() * delta);
                }
            };

            {
                let mut camera_transform = transform_manager.get_mut(self.camera);
                let rotation = camera_transform.rotation();

                // Apply a rotation to the camera based on mouse movmeent.
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
            let gun_animation_manager = scene.get_manager::<GunPhysicsManager>();

            let mut audio_source = audio_manager.get_mut(self.gun_entity);
            audio_source.reset();
            audio_source.play();

            let bullet_pos = position
                           + (self.bullet_offset.x * right_dir)
                           + (self.bullet_offset.y * up_dir)
                           + (self.bullet_offset.z * forward_dir);
            Bullet::new(scene, bullet_pos, rotation);

            let mut physics = gun_animation_manager.get_mut(self.gun_entity);
            physics.deflect(Vector3::new(0.0, 3.0, 5.0), Vector3::new(15.0 * PI, -8.0 * PI, 5.0 * PI));
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GunPhysics {
    /// Mass in kilograms. Not a measure of anything specific, just used for the simulation.
    pub mass: f32,
    pub rotational_inertia: f32,

    pub spring_constant: f32,
    pub angular_spring: f32,

    pub damping: f32,
    pub angular_damping: f32,

    /// The current velocity of the simulation in meters per second.
    velocity: Vector3,

    /// Euler angles.
    angular_velocity: Vector3,
}

impl GunPhysics {
    pub fn deflect(&mut self, velocity: Vector3, angular_velocity: Vector3) {
        self.velocity = self.velocity + velocity;
        self.angular_velocity = self.angular_velocity + angular_velocity;
    }
}

pub type GunPhysicsManager = StructComponentManager<GunPhysics>;

pub struct GunPhysicsSystem;

impl System for GunPhysicsSystem {
    fn update(&mut self, scene: &mut Scene, delta: f32) {
        let transform_manager = scene.get_manager::<TransformManager>();
        let gun_animation_manager = scene.get_manager::<GunPhysicsManager>();

        for (mut physics, entity) in gun_animation_manager.iter_mut() {
            let mut transform = transform_manager.get_mut(entity);

            // Calculate the force based on the offset from equilibrium (the origin).
            let offset = transform.position().as_vector3();
            let spring = -physics.spring_constant * offset;
            let damping = -physics.damping * physics.velocity;
            let force = spring + damping;

            // Calculate the resulting acceleration using SCIENCE!
            let acceleration = force / physics.mass;
            physics.velocity = physics.velocity + acceleration * delta;

            // Update the position.
            transform.translate(physics.velocity * delta);

            // Calculate torque.
            let angular_offset = transform.rotation().as_eulers();
            let spring_torque = -physics.angular_spring * angular_offset;
            let damping_torque = -physics.angular_damping * physics.angular_velocity;
            let torque = spring_torque + damping_torque;

            let angular_acceleration = torque / physics.rotational_inertia;
            physics.angular_velocity = physics.angular_velocity + angular_acceleration * delta;

            let temp = physics.angular_velocity * delta;
            transform.rotate(Quaternion::from_eulers(temp.x, temp.y, temp.z));
        }
    }
}
