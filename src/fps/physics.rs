use gunship::*;

#[derive(Debug, Clone, Copy)]
pub struct Rigidbody {
        /// Mass (in kilograms) of the rigidbody.
        pub mass: f32,

        /// Rotoational inertia (or moment of inertia) of the rigid body in whatever units are normally used ?_?
        pub rotational_inertia: f32,

        pub linear_drag: f32,
        pub angular_drag: f32,

        /// The current velocity of the simulation in meters per second.
        velocity: Vector3,

        /// The current angular velocity represented as euler angles.
        ///
        /// The angular velocity is represented using euler angles because quaternions can't
        /// represent rotations greater than 180 degrees. Euler angles allow for high rotation
        /// speeds to be represented in a
        angular_velocity: Vector3,

        /// The total force applied to the rigidbody for the frame.
        ///
        /// Every frame all forces applied to the rigidbody are summed and used to calculate
        /// the rigidbody's acceleration for the frame.
        force: Vector3,

        /// The total torque applied to the rigidbody for the frame.
        ///
        /// Every frame all torques applied to the rigidbody are summed and used to calculate
        /// the rigidbody's angular acceleration for the frame.
        torque: Vector3,
}

impl Rigidbody {
    pub fn new() -> Rigidbody {
        Rigidbody {
            mass: 1.0,
            rotational_inertia: 1.0,

            linear_drag: 0.0,
            angular_drag: 0.0,

            velocity: Vector3::zero(),
            angular_velocity: Vector3::zero(),

            force: Vector3::zero(),
            torque: Vector3::zero(),
        }
    }

    /// Retrieves the current velocity of the rigidbody.
    pub fn velocity(&self) -> Vector3 {
        self.velocity
    }

    /// Sets the current velocity of the rigidbody, overriding the current value.
    pub fn set_velocity(&mut self, velocity: Vector3) {
        self.velocity = velocity;
    }

    /// Adds the specified value to the current velocity of the rigidbody.
    pub fn add_velocity(&mut self, velocity: Vector3) {
        self.velocity = self.velocity + velocity;
    }

    /// Adds the specified value to the current angular velocity of the rigidbody.
    pub fn add_angular_velocity(&mut self, angular_velocity: Vector3) {
        self.angular_velocity = self.angular_velocity + angular_velocity;
    }

    /// Applies the specified force to the rigidbody.
    pub fn apply_force(&mut self, force: Vector3) {
        self.force = self.force + force;
    }

    /// Applies the specified torque to the rigidbody.
    pub fn apply_torque(&mut self, torque: Vector3) {
        self.torque = self.torque + torque;
    }
}

pub type RigidbodyManager = StructComponentManager<Rigidbody>;

#[derive(Debug, Clone, Copy)]
pub struct RigidbodyUpdateSystem;

impl System for RigidbodyUpdateSystem {
    fn update(&mut self, scene: &mut Scene, delta: f32) {
        let rigidbody_manager = scene.get_manager::<RigidbodyManager>();
        let transform_manager = scene.get_manager::<TransformManager>();

        for (mut rigidbody, entity) in rigidbody_manager.iter_mut() {
            let mut transform = transform_manager.get_mut(entity);

            // Calculate the acceleration from the forces, then use the acceleration to
            // update the velocity.
            let damping = -rigidbody.linear_drag * rigidbody.velocity;
            let force = rigidbody.force + damping;

            let acceleration = force / rigidbody.mass;
            rigidbody.velocity = rigidbody.velocity + acceleration * delta;

            transform.translate(rigidbody.velocity * delta);

            // Calculate angular acceleration from the torques, then use angular acceleration
            // to update the angular velocity.
            let damping_torque = -rigidbody.angular_drag * rigidbody.angular_velocity;
            let torque = rigidbody.torque + damping_torque;

            let angular_acceleration = torque / rigidbody.rotational_inertia;
            rigidbody.angular_velocity = rigidbody.angular_velocity + angular_acceleration * delta;

            let delta_angular = rigidbody.angular_velocity * delta;
            transform.rotate(Quaternion::from_eulers(delta_angular.x, delta_angular.y, delta_angular.z)); // TODO: Add Transform::rotate_eulers() for convenience.

            rigidbody.force = Vector3::zero();
            rigidbody.torque = Vector3::zero();
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GunPhysics {
    pub spring_constant: f32,
    pub angular_spring: f32,
}

pub type GunPhysicsManager = StructComponentManager<GunPhysics>;

pub struct GunPhysicsSystem;

impl System for GunPhysicsSystem {
    fn update(&mut self, scene: &mut Scene, _delta: f32) {

        let gun_animation_manager = scene.get_manager::<GunPhysicsManager>();
        let rigidbody_manager = scene.get_manager::<RigidbodyManager>();
        let transform_manager = scene.get_manager::<TransformManager>();

        for (mut gun_physics, entity) in gun_animation_manager.iter_mut() {
            let mut rigidbody = rigidbody_manager.get_mut(entity);
            let transform = transform_manager.get(entity);

            // Override values for debug purposes.
            gun_physics.spring_constant = 500.0;
            gun_physics.angular_spring = 400.0;

            rigidbody.linear_drag = 20.0;
            rigidbody.angular_drag = 20.0;

            // Calculate the force based on the offset from equilibrium (the origin).
            let offset = transform.position().as_vector3();
            let spring = -gun_physics.spring_constant * offset;
            rigidbody.apply_force(spring);

            // Calculate torque.
            let angular_offset = transform.rotation().as_eulers();
            let torque = -gun_physics.angular_spring * angular_offset;
            rigidbody.apply_torque(torque);
        }
    }
}
