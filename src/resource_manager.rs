use std::{fmt::Debug, hash::Hash, marker::PhantomData};

pub struct ResourceRef<R> {
    pub index: usize,
    pub version: u64,
    _resource_type: PhantomData<R>,
}

impl<R> Copy for ResourceRef<R> {}

impl<R> Clone for ResourceRef<R> {
    fn clone(&self) -> Self {
        Self {
            index: self.index.clone(),
            version: self.version.clone(),
            _resource_type: self._resource_type.clone(),
        }
    }
}
impl<R> Hash for ResourceRef<R> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.version.hash(state);
        self._resource_type.hash(state);
    }
}

impl<R> Debug for ResourceRef<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceRef")
            .field("index", &self.index)
            .field("version", &self.version)
            .field("_resource_type", &self._resource_type)
            .finish()
    }
}

impl<R> PartialEq for ResourceRef<R> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
            && self.version == other.version
            && self._resource_type == other._resource_type
    }
}

impl<R> Eq for ResourceRef<R> {}

impl<R> ResourceRef<R> {
    // todo remove pub
    pub fn new(version: usize, index: usize) -> Self {
        Self {
            version,
            index,
            _resource_type: PhantomData,
        }
    }
}

pub trait ResourceManager<R> {
    fn get(&self, resource_ref: ResourceRef<R>) -> Option<&R>;
    fn get_default<'a>(&'a self, resource_ref: ResourceRef<R>, default: &'a R) -> &'a R {
        self.get(resource_ref).unwrap_or(default)
    }
}

struct VersionedResource<R> {
    version: u64,
    value: R,
}

pub struct BaseResourceManager<R> {
    resources: Vec<VersionedResource<R>>,
}

impl<R> ResourceManager<R> for BaseResourceManager<R> {
    fn get(&self, resource_ref: ResourceRef<R>) -> Option<&R> {
        // todo: check id
        self.resources.get(resource_ref.index).map(|v| &v.value)
    }
}

impl<R> BaseResourceManager<R> {
    pub fn new() -> Self {
        Self { resources: vec![] }
    }

    pub fn add(&mut self, value: R) -> ResourceRef<R> {
        let r = ResourceRef::new(0, self.resources.len());
        self.resources.push(VersionedResource {
            version: r.version,
            value,
        });
        r
    }
}
