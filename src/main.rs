extern crate gunship;

pub mod bullet;
pub mod physics;
pub mod player;
pub mod gun;

use gunship::*;
use gunship::camera::Camera;
use gunship::engine::*;
use gunship::transform::Transform;
use gunship::mesh_renderer::MeshRenderer;
use gunship::math::*;
use std::mem;

use self::physics::*;
use self::player::*;
use self::gun::*;

pub fn main() {
    let mut builder = EngineBuilder::new();
    builder.max_workers(8);
    builder.build(|| {
        setup_scene();
    });
}

fn setup_scene() {
    input::set_capture(true);

    // Load all meshes for the game.
    let gun_mesh_task = resource::load_mesh("meshes/gun_small.dae");
    let cube_mesh_task = resource::load_mesh("meshes/cube.dae");

    let gun_mesh = gun_mesh_task.await().expect("Failed to load gun_small.dae");
    let cube_mesh = cube_mesh_task.await().expect("Failed to load cube.dae");

    // Create static gun and bullet meshes, used for points of reference when running around.
    // TODO: Create some kind of level with a floor and some walls and stuff, some kind of actual
    // testing grounds.
    {
        let mut transform = Transform::new();
        let mesh_renderer = MeshRenderer::new(&gun_mesh, &transform);
        transform.set_position(Point::new(0.0, 0.0, -1.0));

        // Make the mesh "static" by ensuring the destructor won't be run.
        // TODO: Figure out a better way to keep track of static scene elements.
        mem::forget(transform);
        mem::forget(mesh_renderer);
    }

    {
        let mut transform = Transform::new();
        let mesh_renderer = MeshRenderer::new(&cube_mesh, &transform);
        transform.set_position(Point::new(-1.0, 0.0, 0.0));

        // Make the mesh "static" by ensuring the destructor won't be run.
        // TODO: Figure out a better way to keep track of static scene elements.
        mem::forget(transform);
        mem::forget(mesh_renderer);
    }

    // Create camera.
    let mut root_transform = Transform::new();
    root_transform.set_position(Point::new(0.0, 0.0, 10.0));
    let camera = Camera::new(&root_transform);

    // Create the player avatar.
    let mut root_rigidbody = Rigidbody::new();
    root_rigidbody.mass = 70.0;
    root_rigidbody.linear_drag = 500.0;

    let gun_physics = GunPhysics {
        linear_spring: 500.0,
        angular_spring: 400.0,

        position_offset: Vector3::new(0.0, -0.1, -0.3),

        .. GunPhysics::default()
    };
    let mut gun = Gun::new(
        &gun_mesh,
        cube_mesh,
        root_transform.position() + gun_physics.position_offset,
        root_transform.orientation(),
    );
    gun.insert_magazine(Magazine {
        capacity: 6,
        rounds: 6,
    });

    let mut player = Player {
        camera: camera,
        transform: root_transform,
        rigidbody: root_rigidbody,

        gun: gun,
        gun_physics: gun_physics,

        pitch: 0.0,
        yaw: 0.0,
    };

    engine::run_each_frame(move || {
        player.update();
    });
}
