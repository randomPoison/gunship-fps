extern crate gunship;

mod bullet;

use std::f32::consts::PI;

use gunship::*;
use gunship::ScanCode::*;

use bullet::{Bullet, BulletManager, BulletSystem};

fn main() {
    let mut engine = Engine::new();

    engine.register_system(Box::new(BulletSystem));

    let (root_entity, camera_entity) = scene_setup(engine.scene_mut());

    engine.register_system(Box::new(PlayerMoveSystem {
        root: root_entity,
        camera: camera_entity,
        bullet_offset: Vector3::new(0.1, -0.06, 0.3),
    }));

    engine.main_loop();
}

fn scene_setup(scene: &mut Scene) -> (Entity, Entity) {
    let mut transform_handle = scene.get_manager::<TransformManager>();
    let mut transform_manager = transform_handle.get();

    let mut mesh_handle = scene.get_manager::<MeshManager>();
    let mut mesh_manager = mesh_handle.get();

    let mut camera_handle = scene.get_manager::<CameraManager>();
    let mut camera_manager = camera_handle.get();

    let mut light_handle = scene.get_manager::<LightManager>();
    let mut light_manager = light_handle.get();

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

    (root_entity, camera_entity)
}

struct PlayerMoveSystem {
    root: Entity,
    camera: Entity,
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

            let (root_position, root_rotation) = {
                let mut root_transform = transform_manager.get_mut(self.root);
                let position = root_transform.position();
                let rotation = root_transform.rotation();

                root_transform.set_rotation(Matrix4::rotation(0.0, (-movement_x as f32) * PI * 0.001, 0.0) * rotation);

                // Calculate the forward and right vectors.
                let forward_dir = -root_transform.rotation().z_part();
                let right_dir = root_transform.rotation().x_part();

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

                (position, rotation)
            };

            {
                let camera_transform = transform_manager.get_mut(self.camera);
                let rotation = camera_transform.rotation();

                // Apply a rotation to the camera based on mouse movmeent.
                camera_transform.set_rotation(
                    Matrix4::rotation((-movement_y as f32) * PI * 0.001, 0.0, 0.0)
                  * rotation);
            }

            transform_manager.update_single(self.camera);
            let camera_transform = transform_manager.get(self.camera);

            (camera_transform.position_derived(), camera_transform.rotation_derived())
        };

        let up_dir = rotation.y_part();
        let right_dir = rotation.y_part();
        let forward_dir = -rotation.z_part();

        // Maybe shoot some bullets?
        if scene.input.mouse_button_pressed(0) {
            let bullet_pos = position
                           + (self.bullet_offset.x * right_dir)
                           + (self.bullet_offset.y * up_dir)
                           + (self.bullet_offset.z * forward_dir);
            Bullet::new(scene, bullet_pos, rotation);
        }
    }
}
