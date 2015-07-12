use gunship::*;

#[derive(Debug, Clone, Copy)]
pub struct Magazine {
    pub capacity: u32,
    pub rounds: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Gun {
    magazine: Option<Magazine>,
    is_cocked: bool,
}

impl Gun {
    pub fn new() -> Gun {
        Gun {
            magazine: None,
            is_cocked: false,
        }
    }

    pub fn insert_magazine(&mut self, magazine: Magazine) {
        assert!(self.magazine.is_none());
        assert!(magazine.capacity > 0);
        assert!(magazine.rounds <= magazine.capacity);

        self.magazine = Some(magazine);
    }

    pub fn magazine(&self) -> &Option<Magazine> {
        &self.magazine
    }

    pub fn magazine_mut(&mut self) -> &mut Option<Magazine> {
        &mut self.magazine
    }

    pub fn fire(&mut self) {
        assert!(self.magazine.is_some());
        assert!(self.magazine.as_ref().unwrap().rounds > 0);
        assert!(self.is_cocked);

        self.magazine.as_mut().unwrap().rounds -= 1;
        self.is_cocked = false;
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

pub type GunManager = StructComponentManager<Gun>;
