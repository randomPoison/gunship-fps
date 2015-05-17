use gunship::*;

pub struct Bullet {
    pub speed: f32,
}

impl Bullet {
    pub fn new(scene: &mut Scene, position: Point, rotation: Matrix4) -> Entity {
        let bullet_entity = scene.entity_manager.create();

        let mut transform_handle = scene.get_manager::<TransformManager>();
        let mut transform_manager = transform_handle.get();

        let mut mesh_handle = scene.get_manager::<MeshManager>();
        let mut mesh_manager = mesh_handle.get();

        // Block is needed to end borrow of scene.transform_manager
        // before scene.get_manager_mut() can be called.
        {
            let bullet_transform = transform_manager.create(bullet_entity);
            bullet_transform.set_position(position);
            bullet_transform.set_rotation(rotation);
            mesh_manager.create(bullet_entity, "meshes/bullet_small.dae");
        }

        let mut bullet_handle = scene.get_manager::<BulletManager>();
        let mut bullet_manager = bullet_handle.get();

        bullet_manager.create(bullet_entity, Bullet {
            speed: 5.0
        });

        bullet_entity
    }
}

pub type BulletManager = StructComponentManager<Bullet>;

pub struct BulletSystem;

impl System for BulletSystem {
    fn update(&mut self, scene: &mut Scene, delta: f32) {
        let mut bullet_handle = scene.get_manager::<BulletManager>();
        let bullet_manager = bullet_handle.get();

        let mut transform_handle = scene.get_manager::<TransformManager>();
        let mut transform_manager = transform_handle.get();

        for (bullet, entity) in bullet_manager.iter() {
            let transform = transform_manager.get_mut(entity);
            let position = transform.position();
            let forward = -transform.rotation_matrix().z_part();
            transform.set_position(position + forward * bullet.speed * delta);
        }
    }
}
