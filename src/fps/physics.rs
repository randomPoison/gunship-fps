use gunship::*;

#[derive(Debug, Clone, Copy)]
pub struct Rigidbody {
        /// The current velocity of the simulation in meters per second.
        velocity: Vector3,

        /// The current angular velocity represented as euler angles.
        ///
        /// The angular velocity is represented using euler angles because quaternions can't
        /// represent rotations greater than 180 degrees. Euler angles allow for high rotation
        /// speeds to be represented in a
        angular_velocity: Vector3,
}

impl Rigidbody {
    pub fn new() -> Rigidbody {
        Rigidbody {
            velocity: Vector3::zero(),
            angular_velocity: Vector3::zero(),
        }
    }

    pub fn add_velocity(&mut self, velocity: Vector3) {
        self.velocity = self.velocity + velocity;
    }

    pub fn add_angular_velocity(&mut self, angular_velocity: Vector3) {
        self.angular_velocity = self.angular_velocity + angular_velocity;
    }
}

pub type RigidbodyManager = StructComponentManager<Rigidbody>;

#[derive(Debug, Clone, Copy)]
pub struct RigidbodyUpdateSystem;

impl System for RigidbodyUpdateSystem {
    fn update(&mut self, scene: &mut Scene, delta: f32) {
        let rigidbody_manager = scene.get_manager::<RigidbodyManager>();
        let transform_manager = scene.get_manager::<TransformManager>();

        for (rigidbody, entity) in rigidbody_manager.iter() {
            let mut transform = transform_manager.get_mut(entity);

            // Update the position.
            transform.translate(rigidbody.velocity * delta);

            // Update the rotation.
            let temp = rigidbody.angular_velocity * delta;
            transform.rotate(Quaternion::from_eulers(temp.x, temp.y, temp.z));
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GunPhysics {
    /// Mass in kilograms. Not a measure of anything specific, just used for the simulation.
    pub mass: f32,
    pub rotational_inertia: f32,

    pub spring_constant: f32,
    pub angular_spring: f32,

    pub damping: f32,
    pub angular_damping: f32,
}

pub type GunPhysicsManager = StructComponentManager<GunPhysics>;

pub struct GunPhysicsSystem;

impl System for GunPhysicsSystem {
    fn update(&mut self, scene: &mut Scene, delta: f32) {
        let gun_physics = GunPhysics {
            mass: 1.0,
            rotational_inertia: 1.0,

            spring_constant: 500.0,
            angular_spring: 400.0,

            damping: 20.0,
            angular_damping: 20.0,
        };

        let gun_animation_manager = scene.get_manager::<GunPhysicsManager>();
        let rigidbody_manager = scene.get_manager::<RigidbodyManager>();
        let transform_manager = scene.get_manager::<TransformManager>();

        for (_, entity) in gun_animation_manager.iter_mut() {
            let mut rigidbody = rigidbody_manager.get_mut(entity);
            let transform = transform_manager.get(entity);

            // Calculate the force based on the offset from equilibrium (the origin).
            let offset = transform.position().as_vector3();
            let spring = -gun_physics.spring_constant * offset;
            let damping = -gun_physics.damping * rigidbody.velocity;
            let force = spring + damping;

            // Calculate the resulting acceleration using SCIENCE!
            let acceleration = force / gun_physics.mass;
            rigidbody.velocity = rigidbody.velocity + acceleration * delta;

            // Calculate torque.
            let angular_offset = transform.rotation().as_eulers();
            let spring_torque = -gun_physics.angular_spring * angular_offset;
            let damping_torque = -gun_physics.angular_damping * rigidbody.angular_velocity;
            let torque = spring_torque + damping_torque;

            let angular_acceleration = torque / gun_physics.rotational_inertia;
            rigidbody.angular_velocity = rigidbody.angular_velocity + angular_acceleration * delta;
        }
    }
}
