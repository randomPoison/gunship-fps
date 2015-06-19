mod bullet;
mod physics;

use std::f32::consts::PI;

use gunship::*;
use gunship::ScanCode::*;

use self::bullet::*;
use self::physics::*;

#[no_mangle]
pub fn game_init(engine: &mut Engine) {
    engine.scene_mut().register_manager(BulletManager::new());
    engine.scene_mut().register_manager(GunPhysicsManager::new());
    engine.scene_mut().register_manager(RigidbodyManager::new());

    let (root_entity, camera_entity, gun_entity) = scene_setup(engine.scene_mut());

    engine.register_system(PlayerMoveSystem {
        root: root_entity,
        camera: camera_entity,
        gun_entity: gun_entity,
        bullet_offset: Vector3::new(0.0, 0.04, 0.2),
    });
    engine.register_system(GunPhysicsSystem);
    engine.register_system(BulletSystem);
}

#[no_mangle]
pub fn game_reload(old_engine: &Engine, engine: &mut Engine) {
    engine.scene_mut().register_manager(old_engine.scene().get_manager_by_name::<BulletManager>().clone());
    engine.scene_mut().register_manager(old_engine.scene().get_manager_by_name::<GunPhysicsManager>().clone());

    engine.register_system(BulletSystem);
    engine.register_system(GunPhysicsSystem);
    engine.register_system(old_engine.get_system_by_name::<PlayerMoveSystem>().clone());
}

fn scene_setup(scene: &mut Scene) -> (Entity, Entity, Entity) {

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
        let mut rigidbody_manager = scene.get_manager_mut::<RigidbodyManager>();

        gun_animation_manager.assign(gun_entity, GunPhysics {
            mass: 1.0,
            rotational_inertia: 1.0,

            spring_constant: 500.0,
            angular_spring: 400.0,

            damping: 10.0,
            angular_damping: 10.0,
        });
        rigidbody_manager.assign(gun_entity, Rigidbody::new());
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
