use bullet::Bullet;
use physics::Rigidbody;
use gunship::*;
use gunship::math::*;
use gunship::mesh_renderer::MeshRenderer;
use gunship::resource::Mesh;
use gunship::transform::Transform;

#[derive(Debug, Clone, Copy)]
pub struct Magazine {
    pub capacity: u32,
    pub rounds: u32,
}

#[derive(Debug)]
pub struct Gun {
    pub transform: Transform,
    pub mesh_renderer: MeshRenderer,
    pub rigidbody: Rigidbody,

    bullet_offset: Vector3, // TODO: Configure based on gun mesh.
    magazine: Option<Magazine>,
    is_cocked: bool,

    bullet_mesh: Mesh,
}

impl Gun {
    pub fn new(
        mesh: &Mesh,
        bullet_mesh: Mesh,
        start_pos: Point,
        start_orientation: Orientation,
    ) -> Gun {
        let mut transform = Transform::new();
        transform.set_position(start_pos);
        transform.set_orientation(start_orientation);
        let mesh_renderer = MeshRenderer::new(&mesh, &transform);
        let rigidbody = Rigidbody::new();

        Gun {
            transform: transform,
            mesh_renderer: mesh_renderer,
            rigidbody: rigidbody,

            bullet_offset: Vector3::new(0.0, 0.04, 0.2),
            magazine: None,
            is_cocked: false,

            bullet_mesh: bullet_mesh,
        }
    }

    pub fn insert_magazine(&mut self, magazine: Magazine) {
        debug_assert!(self.magazine.is_none());
        debug_assert!(magazine.capacity > 0);
        debug_assert!(magazine.rounds <= magazine.capacity);

        self.magazine = Some(magazine);
    }

    pub fn magazine(&self) -> &Option<Magazine> {
        &self.magazine
    }

    pub fn magazine_mut(&mut self) -> &mut Option<Magazine> {
        &mut self.magazine
    }

    pub fn fire(&mut self) {
        self.magazine.as_mut().expect("Can't fire without a magazine").rounds -= 1;
        self.is_cocked = false;

        // TODO: Play audio on gunshot.
        // let mut audio_source = audio_manager.get_mut(player.gun_entity).unwrap();
        // audio_source.reset();
        // audio_source.play();

        let bullet_pos = self.transform.position()
                       + (self.bullet_offset.x * self.transform.right())
                       + (self.bullet_offset.y * self.transform.up())
                       + (self.bullet_offset.z * self.transform.forward());
        let mut bullet = Bullet::new(&self.bullet_mesh, bullet_pos, self.transform.orientation());
        engine::run_each_frame(move || {
            bullet.update();
        });
    }

    pub fn pull_hammer(&mut self) {
        self.is_cocked = true;
    }

    pub fn can_fire(&self) -> bool {
        self.magazine.is_some() &&
        self.magazine.as_ref().unwrap().rounds > 0 &&
        self.is_cocked
    }
}
