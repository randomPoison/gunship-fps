use gunship::*;
use gunship::math::*;
use gunship::mesh_renderer::MeshRenderer;
use gunship::resource::Mesh;
use gunship::transform::Transform;

#[derive(Debug)]
pub struct Bullet {
    transform: Transform,
    mesh_renderer: MeshRenderer,
    pub speed: f32,
}

impl Bullet {
    pub fn new(mesh: &Mesh, position: Point, orientation: Orientation) -> Bullet {
        let mut transform = Transform::new();
        transform.set_position(position);
        transform.set_orientation(orientation);

        // TODO: Remove this once we have a proper bullet mesh.
        transform.set_scale(Vector3::new(0.1, 0.1, 1.0));

        let mesh_renderer = MeshRenderer::new(mesh, &transform);

        Bullet {
            transform: transform,
            mesh_renderer: mesh_renderer,
            speed: 100.0
        }
    }

    pub fn update(&mut self) {
        let forward = self.transform.forward();
        self.transform.translate(forward * self.speed * time::delta_f32());
    }
}
