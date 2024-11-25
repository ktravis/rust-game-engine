use crate::atlas::{Atlas, AtlasBuilder};
use crate::geom::Point;
use crate::sprite::{Animation, Frame, Sprite};
use image::RgbaImage;
use slotmap::{new_key_type, SlotMap};
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;

new_key_type! {
    pub struct SpriteRef;
}

#[derive(Default, Debug)]
pub struct SpriteManager {
    atlas: Atlas,
    sprites: SlotMap<SpriteRef, Sprite>,
    sprite_files: HashMap<PathBuf, asefile::AsepriteFile>,
    sprites_by_name: HashMap<String, SpriteRef>,
    dirty: bool,
}

impl SpriteManager {
    pub fn add_sprite_file_path(&mut self, path: impl Into<PathBuf>) {
        let path_buf = path.into();
        let f = File::open(&path_buf).unwrap();
        self.add_sprite_file(path_buf, f);
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
            let sprite_ref = self.sprites.insert(Sprite::default());
            self.sprites_by_name.insert(name, sprite_ref);
            raw_sprites.push((sprite_ref, a));
        }
        let mut idx = 0;
        self.atlas = atlas_builder.build();
        for (sprite_ref, a) in raw_sprites {
            let s = self.sprites.get_mut(sprite_ref).unwrap();
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

    pub fn get_sprite_ref(&self, key: impl AsRef<str>) -> Option<SpriteRef> {
        self.sprites_by_name.get(key.as_ref()).copied()
    }

    pub fn get_sprite(&self, r: SpriteRef) -> &Sprite {
        self.sprites.get(r).unwrap()
    }

    pub fn get_sprite_mut(&mut self, r: SpriteRef) -> &mut Sprite {
        self.sprites.get_mut(r).unwrap()
    }

    pub fn atlas_image<'a>(&'a mut self) -> &'a RgbaImage {
        self.maybe_rebuild();
        self.atlas.image()
    }
}
