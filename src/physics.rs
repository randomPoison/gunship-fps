use gunship::*;
use gunship::math::*;
use gunship::transform::Transform;

#[derive(Debug)]
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
        self.torque += torque;
    }

    pub fn update(&mut self, transform: &mut Transform) {
        // Calculate the acceleration from the forces, then use the acceleration to
        // update the velocity.
        let damping = -self.linear_drag * self.velocity;
        let force = self.force + damping;

        let acceleration = force / self.mass;
        self.velocity = self.velocity + acceleration * time::delta_f32();

        transform.translate(self.velocity * time::delta_f32());

        // Calculate angular acceleration from the torques, then use angular acceleration
        // to update the angular velocity.
        let damping_torque = -self.angular_drag * self.angular_velocity;
        let torque = self.torque + damping_torque;

        let angular_acceleration = torque / self.rotational_inertia;
        self.angular_velocity = self.angular_velocity + angular_acceleration * time::delta_f32();

        let Vector3 { x, y, z } = self.angular_velocity * time::delta_f32();
        transform.rotate(Orientation::from_eulers(x, y, z));

        // Force and torque are both instantaneous. They're accumlated by all forces acting on the
        // rigidbody during the frame, applied all at once, and then the total is reset for the
        // next frame.
        self.force = Vector3::zero();
        self.torque = Vector3::zero();
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GunPhysics {
    pub linear_spring: f32,
    pub angular_spring: f32,

    pub position_offset: Vector3,

    pub target_position: Point,
    pub target_orientation: Orientation,
}

impl GunPhysics {
    pub fn update_target(&mut self, target_transform: &Transform) {
        self.target_position = target_transform.position() + target_transform.orientation() * self.position_offset;
        self.target_orientation = target_transform.orientation();
    }

    pub fn update(&mut self, rigidbody: &mut Rigidbody, transform: &Transform) {
        // Override values for debug purposes.
        self.linear_spring = 500.0;
        self.angular_spring = 400.0;

        rigidbody.linear_drag = 20.0;
        rigidbody.angular_drag = 20.0;

        // Calculate the force based on the offset from equilibrium (the origin).
        let offset = transform.position() - self.target_position;
        let spring = -self.linear_spring * offset;
        rigidbody.apply_force(spring);

        // Calculate torque.
        let offset = (transform.orientation() - self.target_orientation).as_eulers();
        let torque = -self.angular_spring * offset;
        rigidbody.apply_torque(torque);
    }
}
