extern crate gunship;

mod bullet;

use std::f32::consts::PI;

use gunship::*;
use gunship::ScanCode::*;

use bullet::{Bullet, BulletManager, BulletSystem};

fn main() {
    let mut engine = Engine::new();

    engine.register_system(Box::new(BulletSystem));

    let (root_entity, camera_entity, gun_entity) = scene_setup(engine.scene_mut());

    engine.register_system(Box::new(PlayerMoveSystem {
        root: root_entity,
        camera: camera_entity,
        gun_entity: gun_entity,
        bullet_offset: Vector3::new(0.0, 0.04, 0.2),
    }));
    engine.register_system(Box::new(GunAnimationSystem));

    engine.main_loop();
}

fn scene_setup(scene: &mut Scene) -> (Entity, Entity, Entity) {
    let mut transform_handle = scene.get_manager::<TransformManager>();
    let mut transform_manager = transform_handle.get();

    let mut mesh_handle = scene.get_manager::<MeshManager>();
    let mut mesh_manager = mesh_handle.get();

    let mut camera_handle = scene.get_manager::<CameraManager>();
    let mut camera_manager = camera_handle.get();

    let mut light_handle = scene.get_manager::<LightManager>();
    let mut light_manager = light_handle.get();

    let mut audio_handle = scene.get_manager::<AudioSourceManager>();
    let mut audio_manager = audio_handle.get();

    // Create light.
    {
        let light_entity = scene.entity_manager.create();
        let transform = transform_manager.create(light_entity);
        transform.set_position(Point::new(-1.0, -1.5, 0.0));
        transform.set_scale(Vector3::new(0.1, 0.1, 0.1));
        light_manager.create(
            light_entity,
            Light::Point(PointLight {
                position: Point::origin()
            }));
        mesh_manager.create(light_entity, "meshes/cube.dae");
    }

    // Create light.
    {
        let light_entity = scene.entity_manager.create();
        let transform = transform_manager.create(light_entity);
        transform.set_position(Point::new(-1.0, 1.5, 0.0));
        transform.set_scale(Vector3::new(0.1, 0.1, 0.1));
        light_manager.create(
            light_entity,
            Light::Point(PointLight {
                position: Point::origin()
            }));
        mesh_manager.create(light_entity, "meshes/cube.dae");
    }

    let root_entity = {
        let entity = scene.entity_manager.create();
        let transform = transform_manager.create(entity);
        transform.set_position(Point::new(0.0, 0.0, 0.0));
        entity
    };
    println!("root entity: {:?}", root_entity);

    // Create camera.
    let camera_entity = {
        let camera_entity = scene.entity_manager.create();
        transform_manager.create(camera_entity);
        camera_manager.create(
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

    // Create gun mesh.
    let gun_entity = {
        let gun_entity = scene.entity_manager.create();
        let gun_transform = transform_manager.create(gun_entity);
        gun_transform.set_position(Point::new(0.1, -0.1, -0.3));
        mesh_manager.create(gun_entity, "meshes/gun_small.dae");
        audio_manager.assign(gun_entity, "audio/Shotgun_Blast-Jim_Rogers-1914772763.wav");

        gun_entity
    };
    println!("gun entity: {:?}", gun_entity);

    // Make gun a child of the camera.
    transform_manager.set_child(camera_entity, gun_entity);

    // Create static gun and bullet meshes.
    {
        let static_gun_entity = scene.entity_manager.create();
        let static_bullet_entity = scene.entity_manager.create();

        {
            let gun_transform = transform_manager.create(static_gun_entity);
            gun_transform.set_position(Point::new(0.0, 0.0, -1.0));
        }

        {
            let bullet_transform = transform_manager.create(static_bullet_entity);
            bullet_transform.set_position(Point::new(-1.0, 0.0, 0.0));
        }

        mesh_manager.create(static_gun_entity, "meshes/gun_small.dae");
        mesh_manager.create(static_bullet_entity, "meshes/cube.dae");
    }

    scene.register_manager::<BulletManager>(Box::new(StructComponentManager::new()));
    scene.register_manager::<GunAnimationManager>(Box::new(GunAnimationManager::new()));

    // Add gun animation manager to player gun.
    {
        let mut gun_anim_handle = scene.get_manager::<GunAnimationManager>();
        let mut gun_animation_manager = gun_anim_handle.get();

        gun_animation_manager.create(gun_entity, GunAnimation {
            default_orientation: Quaternion::identity(),
            desired_orientation: Quaternion::look_rotation(Vector3::new(0.0, 1.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
        });
    }

    (root_entity, camera_entity, gun_entity)
}

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
            let mut transform_handle = scene.get_manager::<TransformManager>();
            let mut transform_manager = transform_handle.get();

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
                let camera_transform = transform_manager.get_mut(self.camera);
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
            let mut audio_handle = scene.get_manager::<AudioSourceManager>();
            let mut audio_manager = audio_handle.get();

            let mut audio_source = audio_manager.get_mut(self.gun_entity);
            audio_source.reset();
            audio_source.play();

            let bullet_pos = position
                           + (self.bullet_offset.x * right_dir)
                           + (self.bullet_offset.y * up_dir)
                           + (self.bullet_offset.z * forward_dir);
            Bullet::new(scene, bullet_pos, rotation);
        }
    }
}

pub struct GunAnimation {
    pub default_orientation: Quaternion,
    pub desired_orientation: Quaternion,
}

pub type GunAnimationManager = StructComponentManager<GunAnimation>;

pub struct GunAnimationSystem;

impl System for GunAnimationSystem {
    fn update(&mut self, scene: &mut Scene, delta: f32) {
        let mut transform_handle = scene.get_manager::<TransformManager>();
        let mut transform_manager = transform_handle.get();

        let mut gun_anim_handle = scene.get_manager::<GunAnimationManager>();
        let mut gun_animation_manager = gun_anim_handle.get();

        for (gun_animation, entity) in gun_animation_manager.iter_mut() {
            let transform = transform_manager.get_mut(entity);

            let temp = Quaternion::nlerp(transform.rotation(), gun_animation.desired_orientation, 1.0 * delta);
            transform.set_rotation(temp);
        }
    }
}
