use gunship::*;

#[derive(Debug, Clone, Copy)]
pub struct Bullet {
    pub speed: f32,
}

impl Bullet {
    pub fn new(scene: &Scene, position: Point, rotation: Quaternion) -> Entity {
        let bullet_entity = scene.create_entity();

        let mut transform_manager = scene.get_manager_mut::<TransformManager>();
        let mut mesh_manager = scene.get_manager_mut::<MeshManager>();
        let mut bullet_manager = scene.get_manager_mut::<BulletManager>();
        let mut alarm_manager = scene.get_manager_mut::<AlarmManager>();

        let mut bullet_transform = transform_manager.assign(bullet_entity);
        bullet_transform.set_position(position);
        bullet_transform.set_rotation(rotation);

        mesh_manager.assign(bullet_entity, "meshes/bullet_small.dae");
        bullet_manager.assign(bullet_entity, Bullet {
            speed: 5.0
        });

        alarm_manager.assign(bullet_entity, 1.0, |_scene, entity| {
            // TODO: Destroy the bullet.
            println!("Time to destroy bullet entity {:?}", entity);
        });

        bullet_entity
    }
}

pub type BulletManager = StructComponentManager<Bullet>;

pub struct BulletSystem;

impl System for BulletSystem {
    fn update(&mut self, scene: &Scene, delta: f32) {
        let bullet_manager = scene.get_manager::<BulletManager>();
        let transform_manager = scene.get_manager::<TransformManager>();

        for (bullet, entity) in bullet_manager.iter() {
            let mut transform = transform_manager.get_mut(entity);
            let position = transform.position();
            let forward = -transform.rotation().as_matrix().z_part();
            transform.set_position(position + forward * bullet.speed * delta);
        }
    }
}
