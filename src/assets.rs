use glob::Pattern;
use notify::{Event, EventKind, Watcher};
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::{collections::HashMap, path::PathBuf};

type AssetChangedCallback<S> = fn(&mut S, &Path, File);

pub struct AssetManager<S> {
    state: S,
    base_path: PathBuf,
    watcher: Option<Box<dyn Watcher>>,
    recv: Option<std::sync::mpsc::Receiver<Event>>,
    file_callbacks: HashMap<PathBuf, AssetChangedCallback<S>>,
    glob_callbacks: HashMap<Pattern, AssetChangedCallback<S>>,
}

// TODO: error handling
impl<S> AssetManager<S> {
    const WRITE_EVENT: EventKind = EventKind::Access(notify::event::AccessKind::Close(
        notify::event::AccessMode::Write,
    ));
    const CREATE_EVENT: EventKind = EventKind::Create(notify::event::CreateKind::File);

    pub fn new(state: S, base_path: impl Into<PathBuf>) -> Self {
        let base_path: PathBuf = base_path.into();

        let (send, recv) = std::sync::mpsc::channel();
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(
                evt @ Event {
                    kind:
                        EventKind::Access(notify::event::AccessKind::Close(
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
        Self {
            state,
            base_path,
            recv: Some(recv),
            watcher: Some(Box::new(watcher)),
            file_callbacks: Default::default(),
            glob_callbacks: Default::default(),
        }
    }

    pub fn track_file(
        &mut self,
        path: impl Into<PathBuf>,
        callback: AssetChangedCallback<S>,
    ) -> &mut Self {
        let path = path.into().canonicalize().unwrap();
        if !path.exists() {
            panic!("path {:?} does not exist", path);
        }
        if !path.is_file() {
            panic!("path {:?} is not a file", path);
        }
        let f = File::open(path.as_path()).unwrap();
        callback(self, path.as_path(), f);
        self.file_callbacks.insert(path, callback);
        self
    }

    pub fn track_glob(
        &mut self,
        pattern: impl AsRef<str>,
        callback: AssetChangedCallback<S>,
    ) -> &mut Self {
        let pattern = pattern.as_ref();
        for entry in glob::glob(pattern).unwrap() {
            if let Ok(path_buf) = entry {
                println!("Glob matched: {:?}", &path_buf);
                let f = File::open(path_buf.as_path()).unwrap();
                callback(self, path_buf.as_path(), f);
            }
        }
        let pattern = Pattern::new(pattern).unwrap();
        self.glob_callbacks.insert(pattern, callback);
        self
    }

    pub fn check_for_updates(&mut self) -> bool {
        let Some(ref recv) = &mut self.recv else {
            return false;
        };
        let needs_rebuild = loop {
            match recv.try_recv() {
                Ok(Event { kind, paths, .. }) => match kind {
                    Self::CREATE_EVENT | Self::WRITE_EVENT => {
                        self.process_event_paths(paths);
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
        needs_rebuild
    }

    fn process_event_paths(&mut self, paths: Vec<PathBuf>) {
        for modified_path in paths {
            let Ok(modified_path) = modified_path.canonicalize() else {
                continue;
            };
            self.file_callbacks.get(&modified_path).map(|callback| {
                println!("file is tracked: {:?}", modified_path);
                let f = File::open(modified_path.as_path()).unwrap();
                callback(&mut self.state, modified_path.as_path(), f);
            });
            for (pattern, callback) in &self.glob_callbacks {
                if pattern.matches_path(modified_path.as_path()) {
                    println!(
                        "file matches pattern {}: {:?}",
                        pattern.as_str(),
                        modified_path
                    );
                    let f = File::open(modified_path.as_path()).unwrap();
                    callback(&mut self.state, modified_path.as_path(), f);
                }
            }
        }
    }
}

impl<S> Deref for AssetManager<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<S> DerefMut for AssetManager<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}
