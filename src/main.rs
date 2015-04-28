extern crate gunship;

use std::f32::consts::PI;

use gunship::*;
use gunship::ScanCode::*;

fn main() {
    // Start Gunship.
    let mut engine = Engine::new();

    engine.register_system(Box::new(CameraMoveSystem {
        rotation_x: 0.0,
        rotation_y: 0.0,
        bullet_offset: Vector3::new(0.1, -0.06, 0.3),
    }));
    engine.register_system(Box::new(BulletSystem));

    // Block needed to end borrow of engine.scene before call to register_system().
    scene_setup(engine.scene_mut());

    engine.main_loop();
}

fn scene_setup(scene: &mut Scene) {
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
        transform_manager.create(light_entity);
        light_manager.create(
            light_entity,
            Light::Point(PointLight {
                position: Point::origin()
            }));
    }

    // Create camera.
    let camera_entity = {
        let camera_entity = scene.entity_manager.create();
        let transform = transform_manager.create(camera_entity);
        transform.position = Point::new(0.0, 0.0, 0.0);
        camera_manager.create(
            camera_entity,
            Camera::new(
                PI / 3.0,
                1.0,
                0.001,
                100.0));

        camera_entity
    };

    // Create gun mesh.
    let gun_entity = {
        let gun_entity = scene.entity_manager.create();
        let gun_transform = transform_manager.create(gun_entity);
        gun_transform.position = Point::new(-0.1, -0.1, 0.3);
        mesh_manager.create(gun_entity, "meshes/gun_small.dae");

        gun_entity
    };

    // Make gun a child of the camera.
    transform_manager.set_child(camera_entity, gun_entity);

    // Create reference gun and bullet meshes.
    {
        let static_gun_entity = scene.entity_manager.create();
        let static_bullet_entity = scene.entity_manager.create();

        {
            let gun_transform = transform_manager.create(static_gun_entity);
            gun_transform.position = Point::new(1.0, 0.0, 0.0);
        }

        {
            let bullet_transform = transform_manager.create(static_bullet_entity);
            bullet_transform.position = Point::new(-1.0, 0.0, 0.0);
        }

        mesh_manager.create(static_gun_entity, "meshes/gun_small.dae");
        mesh_manager.create(static_bullet_entity, "meshes/cube.dae");
    }

    scene.register_manager::<BulletManager>(Box::new(StructComponentManager::new()));
}

struct CameraMoveSystem {
    rotation_x: f32,
    rotation_y: f32,
    bullet_offset: Vector3,
}

impl System for CameraMoveSystem {
    fn update(&mut self, scene: &mut Scene, delta: f32) {
        let mut camera_handle = scene.get_manager::<CameraManager>();
        let camera_manager = camera_handle.get();

        let mut transform_handle = scene.get_manager::<TransformManager>();
        let mut transform_manager = transform_handle.get();

        let mut mesh_handle = scene.get_manager::<MeshManager>();
        let mut mesh_manager = mesh_handle.get();

        let entity = camera_manager.entities()[0];

        // Cache off the position and rotation and then drop the transform
        // so that we don't have multiple borrows of transform_manager.
        let (position, rotation, forward_dir, right_dir, up_dir) = {
            let transform = transform_manager.get_mut(entity);
            let (movement_x, movement_y) = scene.input.mouse_delta();

            // Add mouse movement to total rotation.
            self.rotation_x += (-movement_y as f32) * PI * 0.001;
            self.rotation_y += (-movement_x as f32) * PI * 0.001;

            // Apply a rotation to the camera based on mouse movmeent.
            transform.rotation =
                Matrix4::rotation(
                    self.rotation_x,
                    self.rotation_y,
                    0.0);

            // Calculate the forward and right vectors.
            let forward_dir = -transform.rotation.z_part();
            let right_dir = transform.rotation.x_part();

            // Move camera based on input.
            if scene.input.key_down(W) {
                transform.position = transform.position + forward_dir * delta;
            }

            if scene.input.key_down(S) {
                transform.position = transform.position - forward_dir * delta;
            }

            if scene.input.key_down(D) {
                transform.position = transform.position + right_dir * delta;
            }

            if scene.input.key_down(A) {
                transform.position = transform.position - right_dir * delta
            }

            (transform.position, transform.rotation, forward_dir, right_dir, transform.rotation.y_part())
        };

        // Maybe shoot some bullets?
        if scene.input.mouse_button_pressed(0) {
            let bullet_entity = scene.entity_manager.create();

            // Block is needed to end borrow of scene.transform_manager
            // before scene.get_manager_mut() can be called.
            {
                let bullet_transform = transform_manager.create(bullet_entity);
                bullet_transform.position =
                    position
                  + (self.bullet_offset.x * right_dir)
                  + (self.bullet_offset.y * up_dir)
                  + (self.bullet_offset.z * forward_dir);
                bullet_transform.rotation = rotation;
                mesh_manager.create(bullet_entity, "meshes/bullet_small.dae");
            }

            let mut bullet_handle = scene.get_manager::<BulletManager>();
            let mut bullet_manager = bullet_handle.get();

            bullet_manager.create(bullet_entity, Bullet {
                speed: 5.0
            });
        }
    }
}

struct Bullet {
    speed: f32,
}

pub type BulletManager = StructComponentManager<Bullet>;

struct BulletSystem;

impl System for BulletSystem {
    fn update(&mut self, scene: &mut Scene, delta: f32) {
        let mut bullet_handle = scene.get_manager::<BulletManager>();
        let bullet_manager = bullet_handle.get();

        let mut transform_handle = scene.get_manager::<TransformManager>();
        let mut transform_manager = transform_handle.get();

        for (bullet, entity) in bullet_manager.iter() {
            let transform = transform_manager.get_mut(entity);
            let forward = transform.rotation_matrix().z_part();
            transform.position = transform.position + forward * bullet.speed * delta;
        }
    }
}
