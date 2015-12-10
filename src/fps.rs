extern crate gunship;

mod bullet;
mod physics;
mod player;
mod gun;

use std::f32::consts::PI;

pub use gunship::*;

use self::bullet::*;
use self::physics::*;
use self::player::*;
use self::gun::*;

pub fn do_main() {
    let mut engine = Engine::new();
    game_init(&mut engine);
    engine.main_loop();
}

/// Super cool temporary macro to paper over the fact that Gunship's hotloading support is
/// basically one big hack.
///
/// This allows me to just list off the managers, systems, and models that are used in the game and
/// automatically handle registering at startup and reloading for hotloading support. Eventually
/// I'd like the engine to more gracefully handle this automatically (through compile time code
/// generation most likely), but for now I'm leaving this at the game layer so that it at least
/// doesn't infect the engine proper.
macro_rules! game_setup {
    (
        setup: $setup:ident,

        managers:
        $($manager:ty => $manager_instance:expr),*

        systems:
        $($system:ty => $system_instance:expr),*

        models:
        $($model:expr),*
    ) => {
        #[no_mangle]
        pub fn game_init(engine: &mut Engine) {
            $(engine.scene_mut().register_manager($manager_instance);)*
            $(engine.register_system($system_instance);)*

            $(engine.scene().resource_manager().load_resource_file($model).unwrap();)*

            $setup(engine.scene_mut());
        }

        #[no_mangle]
        pub fn game_reload(old_engine: &Engine, engine: &mut Engine) {
            $(engine.scene_mut().reload_manager::<$manager>(old_engine.scene());)*
            $(engine.register_system(old_engine.get_system_by_name::<$system>().clone());)*
        }
    }
}

game_setup! {
    setup: scene_setup,

    managers:
        BulletManager     => BulletManager::new(),
        GunPhysicsManager => GunPhysicsManager::new(),
        RigidbodyManager  => RigidbodyManager::new(),
        GunManager        => GunManager::new(),
        PlayerManager     => PlayerManager::new()

    systems:
        PlayerMoveSystem      => PlayerMoveSystem,
        GunPhysicsSystem      => GunPhysicsSystem,
        BulletSystem          => BulletSystem,
        RigidbodyUpdateSystem => RigidbodyUpdateSystem

    models:
        "meshes/cube.dae",
        "meshes/gun_small.dae",
        "meshes/bullet_small.dae",
        "meshes/sphere.dae"
}

fn scene_setup(scene: &mut Scene) {
    // Instantiate some entities from "prefabs", e.g. create their full mesh hierarchy. No way to
    // specify more prefab data than that at this point. Also we can't have the transform manager
    // or the mesh manager borrowed when we call `instantiate_model()` or we panic.
    let static_gun_entity = scene.instantiate_model("gun_small");
    let static_cube_entity = scene.instantiate_model("cube");
    let gun_entity = scene.instantiate_model("gun_small");

    // Create lights with a helper function.
    fn create_light(scene: &Scene, position: Point) -> Entity {
        let light_entity = scene.instantiate_model("sphere");

        let transform_manager = scene.get_manager::<TransformManager>();
        let mut light_manager = scene.get_manager_mut::<LightManager>();

        let mut transform = transform_manager.get_mut(light_entity);
        transform.set_position(position);
        transform.set_scale(Vector3::new(0.1, 0.1, 0.1));
        light_manager.assign(
            light_entity,
            Light::Point(PointLight {
                position: Point::origin()
            }));

        light_entity
    };
    create_light(scene, Point::new(-1.0, -1.5, 0.0));
    create_light(scene, Point::new(-1.0, 1.5, 0.0));

    let mut audio_manager         = scene.get_manager_mut::<AudioSourceManager>();
    let mut camera_manager        = scene.get_manager_mut::<CameraManager>();
    let mut gun_animation_manager = scene.get_manager_mut::<GunPhysicsManager>();
    let mut gun_manager           = scene.get_manager_mut::<GunManager>();
    let mut player_manager        = scene.get_manager_mut::<PlayerManager>();
    let mut rigidbody_manager     = scene.get_manager_mut::<RigidbodyManager>();
    let mut transform_manager     = scene.get_manager_mut::<TransformManager>();

    // Fully create gun entity.
    {
        audio_manager.assign(gun_entity, "audio/Shotgun_Blast-Jim_Rogers-1914772763.wav");
        gun_animation_manager.assign(gun_entity, GunPhysics {
            spring_constant: 500.0,
            angular_spring: 400.0,
        });
        rigidbody_manager.assign(gun_entity, Rigidbody::new());
        let mut gun = gun_manager.assign(gun_entity, Gun::new());
        gun.insert_magazine(Magazine {
            capacity: 6,
            rounds: 6,
        });

        gun_entity
    };

    let root_entity = {
        let entity = scene.create_entity();

        let mut transform = transform_manager.assign(entity);
        transform.set_position(Point::new(0.0, 0.0, 0.0));

        let mut rigidbody = rigidbody_manager.assign(entity, Rigidbody::new());
        rigidbody.mass = 70.0;
        rigidbody.linear_drag = 500.0;

        entity
    };

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

    transform_manager.set_child(root_entity, camera_entity);

    // Create gun root
    let gun_root = {
        let gun_root = scene.create_entity();
        let mut gun_root_transform = transform_manager.assign(gun_root);
        gun_root_transform.set_position(Point::new(0.1, -0.1, -0.3));

        gun_root
    };
    transform_manager.set_child(camera_entity, gun_root);

    // Make gun a child of the camera.
    transform_manager.set_child(gun_root, gun_entity);

    player_manager.assign(root_entity, Player {
        camera: camera_entity,
        gun_entity: gun_entity,
        bullet_offset: Vector3::new(0.0, 0.04, 0.2),
        gun_alarm: None,
    });

    // Create static gun and bullet meshes.
    {
        let mut gun_transform = transform_manager.get_mut(static_gun_entity);
        gun_transform.set_position(Point::new(0.0, 0.0, -1.0));
    }

    {
        let mut bullet_transform = transform_manager.get_mut(static_cube_entity);
        bullet_transform.set_position(Point::new(-1.0, 0.0, 0.0));
    }
}
