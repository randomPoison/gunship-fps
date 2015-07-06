mod bullet;
mod physics;
mod player;

use std::f32::consts::PI;

use gunship::*;

use self::bullet::*;
use self::physics::*;
use self::player::*;

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

    engine.register_system(RigidbodyUpdateSystem);
}

#[no_mangle]
pub fn game_reload(old_engine: &Engine, engine: &mut Engine) {
    engine.scene_mut().register_manager(old_engine.scene().get_manager_by_name::<BulletManager>().clone());
    engine.scene_mut().register_manager(old_engine.scene().get_manager_by_name::<GunPhysicsManager>().clone());
    engine.scene_mut().register_manager(old_engine.scene().get_manager_by_name::<RigidbodyManager>().clone());

    engine.register_system(old_engine.get_system_by_name::<PlayerMoveSystem>().clone());
    engine.register_system(BulletSystem);
    engine.register_system(GunPhysicsSystem);

    engine.register_system(RigidbodyUpdateSystem);
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
    let mut gun_animation_manager = scene.get_manager_mut::<GunPhysicsManager>();
    let mut rigidbody_manager = scene.get_manager_mut::<RigidbodyManager>();

    let root_entity = {
        let entity = scene.create_entity();

        let mut transform = transform_manager.assign(entity);
        transform.set_position(Point::new(0.0, 0.0, 0.0));

        let mut rigidbody = rigidbody_manager.assign(entity, Rigidbody::new());
        rigidbody.mass = 70.0;
        rigidbody.linear_drag = 500.0;

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
        gun_animation_manager.assign(gun_entity, GunPhysics {
            spring_constant: 500.0,
            angular_spring: 400.0,
        });
        rigidbody_manager.assign(gun_entity, Rigidbody::new());
    }

    (root_entity, camera_entity, gun_entity)
}
