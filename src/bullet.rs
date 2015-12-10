use gunship::*;

#[derive(Debug, Clone, Copy)]
pub struct Bullet {
    pub speed: f32,
}

impl Bullet {
    pub fn new(scene: &Scene, position: Point, rotation: Quaternion) -> Entity {
        let bullet_entity = scene.instantiate_model("bullet_small");

        let transform_manager = scene.get_manager::<TransformManager>();
        let mut bullet_manager = scene.get_manager_mut::<BulletManager>();
        let mut alarm_manager = scene.get_manager_mut::<AlarmManager>();

        let mut bullet_transform = transform_manager.get_mut(bullet_entity);
        bullet_transform.set_position(position);
        bullet_transform.set_rotation(rotation);

        bullet_manager.assign(bullet_entity, Bullet {
            speed: 5.0
        });

        alarm_manager.assign(bullet_entity, 1.0, |scene, entity| {
            scene.destroy_entity(entity);
        });

        bullet_entity
    }
}

pub type BulletManager = StructComponentManager<Bullet>;

#[derive(Debug, Clone)]
pub struct BulletSystem;

impl System for BulletSystem {
    fn update(&mut self, scene: &Scene, delta: f32) {
        let bullet_manager = scene.get_manager::<BulletManager>();
        let transform_manager = scene.get_manager::<TransformManager>();

        for (bullet, entity) in bullet_manager.iter() {
            let mut transform = transform_manager.get_mut(entity);

            let forward  = transform.forward();
            transform.translate(forward * bullet.speed * delta);
        }
    }
}
