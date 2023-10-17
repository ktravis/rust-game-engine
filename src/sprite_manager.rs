use crate::atlas::{Atlas, AtlasBuilder};
use crate::geom::Point;
use crate::sprite::{Animation, Frame, Sprite};
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Default, Debug)]
pub struct SpriteManager {
    pub atlas: Atlas,
    last_id: usize,
    sprite_files: HashMap<PathBuf, asefile::AsepriteFile>,
    sprite_indices: HashMap<String, usize>,
    sprites: Vec<Sprite>,
    dirty: bool,
}

impl SpriteManager {
    pub fn add_sprite_file_path(&mut self, path: PathBuf) {
        let f = File::open(&path).unwrap();
        self.add_sprite_file(path, f);
    }

    pub fn add_sprite_file(&mut self, path: PathBuf, file: File) {
        let a = asefile::AsepriteFile::read(file).unwrap();
        self.sprite_files.insert(path, a);
        self.dirty = true;
    }

    pub fn maybe_rebuild(&mut self) -> bool {
        if !self.dirty {
            return false;
        }
        println!("rebuilding");
        self.rebuild_atlas()
            .unwrap_or_else(|e| println!("atlas rebuild failed: {:?}", e));
        self.dirty = false;
        true
    }

    pub fn rebuild_atlas(&mut self) -> image::ImageResult<()> {
        let mut atlas_builder = AtlasBuilder::default();
        let mut raw_sprites = vec![];
        for (path, a) in &self.sprite_files {
            let name = path.file_stem().unwrap().to_owned().into_string().unwrap();
            for f in 0..a.num_frames() {
                atlas_builder.add(&a.frame(f).image()).unwrap();
            }
            let sprite_ref = self.get_sprite_ref(&name).unwrap_or_else(|| {
                let index = self.sprites.len();
                self.sprites.push(Sprite::default());
                self.sprite_indices.insert(name.clone(), index);
                let id = self.last_id;
                self.last_id += 1;
                SpriteRef::new(id, index)
            });
            raw_sprites.push((sprite_ref, a));
        }
        let mut idx = 0;
        self.atlas = atlas_builder.build();
        for (sprite_ref, a) in raw_sprites {
            let mut s = &mut self.sprites[sprite_ref.index];
            s.frames = (0..a.num_frames())
                .map(|i| {
                    let f = a.frame(i);
                    idx += 1;
                    Frame {
                        index: i as _,
                        duration: Duration::from_millis(f.duration() as _),
                        region: self.atlas.entry_rect(idx - 1),
                    }
                })
                .collect();
            s.size = Point::new(a.width() as u32, a.height() as u32);
            s.pivot = a
                .slices()
                .first()
                .and_then(|s| s.keys.first())
                .and_then(|k| k.pivot.map(|(a, b)| Point::new(a as _, b as _)));
            s.animations = (0..a.num_tags())
                .map(|i| a.tag(i))
                .map(|t| Animation {
                    name: t.name().to_string(),
                    frames: (t.from_frame() as usize)..(t.to_frame() + 1) as usize,
                })
                .collect();
        }
        Ok(())
    }

    pub fn get_sprite_ref(&self, r: impl AsRef<str>) -> Option<SpriteRef> {
        let index = *self.sprite_indices.get(r.as_ref())?;

        Some(SpriteRef::new(self.sprites[index].id, index))
    }

    pub fn get_sprite(&self, r: SpriteRef) -> &Sprite {
        let s = &self.sprites[r.index];
        if r.id != s.id {
            panic!(
                "referenced sprite had the wrong id: {} (expected {})",
                s.id, r.id
            );
        }
        s
    }

    pub fn get_sprite_mut(&mut self, r: SpriteRef) -> &mut Sprite {
        let s = &mut self.sprites[r.index];
        if r.id != s.id {
            panic!(
                "referenced sprite had the wrong id: {} (expected {})",
                s.id, r.id
            );
        }
        s
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SpriteRef {
    index: usize,
    id: usize,
}

impl SpriteRef {
    fn new(id: usize, index: usize) -> Self {
        Self { id, index }
    }
}
