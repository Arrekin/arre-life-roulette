use godot::engine::{Engine, NodeExt};
use godot::obj::dom;
use godot::prelude::*;
use crate::errors::ArreError;


pub fn get_singleton<T>(name: impl Into<StringName>) -> Gd<T>
where
    T: GodotClass<Declarer = dom::UserDomain> + Inherits<Object>
{
    let name = name.into();
    Engine::singleton().get_singleton(name).unwrap().cast::<T>()
}

pub struct GdHolder<T>
    where T: GodotClass
{
    pub gd: Option<Gd<T>>,
    pub path: String,
}

impl<T> GdHolder<T>
    where T: GodotClass
{
    pub fn new(gd: Gd<T>, path: impl Into<String>) -> Self {
        Self { gd: Some(gd), path: path.into() }
    }

    #[inline]
    pub fn ok(&self) -> Result<&Gd<T>, ArreError> {
        self.gd.as_ref().ok_or(ArreError::NullGd(self.path.clone()))
    }

    #[inline]
    pub fn ok_mut(&mut self) -> Result<&mut Gd<T>, ArreError> {
        self.gd.as_mut().ok_or(ArreError::NullGd(self.path.clone()))
    }

    #[inline]
    pub fn ok_shared(&self) -> Result<Gd<T>, ArreError> {
        self.gd.as_ref().map(|gd| gd.share()).ok_or(ArreError::NullGd(self.path.clone()))
    }
}

impl<T> GdHolder<T>
    where T: GodotClass + Inherits<Node>
{
    pub fn from_path<B: GodotClass<Declarer = dom::EngineDomain> + Inherits<Node>>(base: &Base<B>, path: impl Into<String>) -> Self {
        let path = path.into();
        Self {
            gd: base.try_get_node_as::<T>(path.clone()),
            path: format!("{}::{}", base.to_string(), path),
        }
    }
    pub fn from_gd<F: GodotClass + Inherits<Node>>(gd: Gd<F>) -> Self {
        let gd = gd.upcast();
        let path = gd.get_path();
        Self { gd: gd.try_cast::<T>(), path: path.into() }
    }
    pub fn from_node(node: Gd<Node>) -> Self {
        let path = node.get_path();
        Self { gd: node.try_cast::<T>(), path: path.into() }
    }
    pub fn from_instance_id(instance_id: InstanceId) -> Self {
        let gd = Gd::<Node>::try_from_instance_id(instance_id);
        let (path, gd) = if let Some(gd) = gd {
            (gd.get_path().into(), gd.try_cast::<T>())
        } else {
            (String::new(), None)
        };
        Self { gd, path }
    }
}

impl<T> From<Gd<T>> for GdHolder<T> where T: GodotClass {
    fn from(value: Gd<T>) -> Self {
        Self { gd: Some(value), path: String::new() }
    }
}

impl<T> Default for GdHolder<T>
    where T: GodotClass
{
    fn default() -> Self {
        Self { gd: None, path: String::new() }
    }
}

impl<T> Clone for GdHolder<T>
    where T: GodotClass
{
    fn clone(&self) -> Self {
        Self {
            gd: self.gd.as_ref().map(|t| t.share()),
            path: self.path.clone()
        }
    }
}