use crate::{
    atlas::{Atlas, AtlasBuilder},
    geom::Point,
    sprite::{Animation, Frame, Sprite},
};
use notify::{Event, EventKind, Watcher};
use std::{collections::HashMap, ffi::OsStr, path::PathBuf, time::Duration};

pub struct AssetWatcher {}

pub struct Assets {
    pub atlas: Atlas,
    pub base_path: PathBuf,
    sprite_files: HashMap<PathBuf, asefile::AsepriteFile>,
    sprite_indices: HashMap<String, usize>,
    sprites: Vec<Sprite>,
    watcher: Option<Box<dyn Watcher>>,
    recv: Option<std::sync::mpsc::Receiver<notify::Event>>,
    last_id: usize,
}

impl Assets {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        let base_path: PathBuf = base_path.into();

        let (send, recv) = std::sync::mpsc::channel();
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(
                evt @ notify::Event {
                    kind:
                        notify::EventKind::Access(notify::event::AccessKind::Close(
                            notify::event::AccessMode::Write,
                        )),
                    ..
                },
            ) = res
            {
                send.send(evt).unwrap();
            }
        })
        .unwrap();
        watcher
            .watch(base_path.as_path(), notify::RecursiveMode::Recursive)
            .unwrap();

        let sprites_path = base_path.join("sprites");
        let sprite_files = sprites_path
            .read_dir()
            .unwrap()
            .filter_map(Result::ok)
            .filter_map(|e| {
                let path = e.path();
                if path.extension() != Some(OsStr::new("aseprite")) {
                    return None;
                }
                let a = asefile::AsepriteFile::read_file(&path)
                    .unwrap_or_else(|_| panic!("invalid aseprite file {:?}", path));
                Some((path, a))
            })
            .collect();

        let mut a = Self {
            base_path,
            recv: Some(recv),
            atlas: Default::default(),
            sprite_files,
            sprite_indices: Default::default(),
            sprites: Default::default(),
            watcher: Some(Box::new(watcher)),
            last_id: 0,
        };
        a.rebuild_atlas().unwrap();
        a
    }

    pub fn check_for_updates(&mut self) -> bool {
        let Some(ref recv) = &mut self.recv else {
            return false;
        };
        let needs_rebuild = loop {
            match recv.try_recv() {
                Ok(Event { kind, paths, .. }) => match kind {
                    EventKind::Create(notify::event::CreateKind::File)
                    | EventKind::Access(notify::event::AccessKind::Close(
                        notify::event::AccessMode::Write,
                    )) => {
                        paths
                            .into_iter()
                            .filter(|p| {
                                p.components()
                                    .any(|c| c.as_os_str() == OsStr::new("sprites"))
                                    && p.extension() == Some(OsStr::new("aseprite"))
                            })
                            .for_each(|p| {
                                let a = asefile::AsepriteFile::read_file(&p).unwrap();
                                self.sprite_files.insert(p, a);
                            });
                        break true;
                    }
                    _ => {}
                },
                Err(std::sync::mpsc::TryRecvError::Empty) => break false,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.recv.take();
                    return false;
                }
            }
        };
        if needs_rebuild {
            self.rebuild_atlas()
                .unwrap_or_else(|e| println!("atlas rebuild failed: {:?}", e));
        }
        needs_rebuild
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assets() {
        let assets = Assets::new("./res");
        println!("testing {:?}", assets.sprites);
    }
}
