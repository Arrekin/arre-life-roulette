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
        match &self.gd {
            Some(v) => Ok(v),
            None => Err(ArreError::NullGd(self.path.clone())),
        }
    }

    #[inline]
    pub fn ok_mut(&mut self) -> Result<&mut Gd<T>, ArreError> {
        match &mut self.gd {
            Some(v) => Ok(v),
            None => Err(ArreError::NullGd(self.path.clone())),
        }
    }

    #[inline]
    pub fn ok_shared(&self) -> Result<Gd<T>, ArreError> {
        match &self.gd {
            Some(v) => Ok(v.share()),
            None => Err(ArreError::NullGd(self.path.clone())),
        }
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
    pub fn from_gd(gd: Gd<T>) -> Self {
        let path = gd.share().upcast::<Node>().get_path();
        Self { gd: Some(gd), path: path.into() }
    }
    pub fn from_instance_id(instance_id: InstanceId) -> Self {
        let object = Gd::<T>::try_from_instance_id(instance_id);
        let gd = if let Some(object) = object { object.try_cast::<T>() } else { None };
        let path = if let Some(gd) = &gd { gd.share().upcast::<Node>().get_path().into() } else { String::new() };
        Self { gd, path: path.into() }
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
            gd: match &self.gd {
                Some(v) => Some(v.share()),
                None => None
            },
            path: self.path.clone()
        }
    }
}