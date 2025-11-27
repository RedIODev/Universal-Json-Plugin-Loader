
pub use mapped_guard::*;
mod mapped_guard {
    #![allow(clippy::field_scoped_visibility_modifiers, 
        clippy::future_not_send, clippy::mem_forget, 
        clippy::shadow_same, clippy::shadow_reuse, reason = "result from macro")]
    use core::ops::Deref;

    use arc_swap::{Guard, RefCnt};
    use ouroboros::self_referencing;

    #[self_referencing]
    pub struct MappedGuard<T: arc_swap::RefCnt + 'static, U: 'static> {
        
        guard: Guard<T>,
        #[borrows(guard)]
        mapped: MappedGuardInner<U>,
    }
    
    impl<T: RefCnt, U> Deref for MappedGuard<T, U> {
        type Target = U;
    
        fn deref(&self) -> &Self::Target {
            &self.borrow_mapped().mapped
        }
    }
    
    struct MappedGuardInner<U> {
        pub mapped: U,
    }

    pub trait GuardExt<T>: Sized
    where
        T: RefCnt,
    {
    
        fn try_map<U, F, E>(self, func: F) -> Result<MappedGuard<T, U>, E>
        where
            F: FnOnce(&T) -> Result<U, E>;
    }
    
    impl<T> GuardExt<T> for Guard<T>
    where
        T: RefCnt,
    {
    
        fn try_map<U, F, E>(self, func: F) -> Result<MappedGuard<T, U>, E>
        where
            F: FnOnce(&T) -> Result<U, E>,
        {
            MappedGuard::try_new(self, |guard| Ok(MappedGuardInner { mapped: func(guard)? }))
        }
    }
}


use arc_swap::{ArcSwap};
use plugin_loader_api::ServiceError;
use im::{HashMap, HashSet, Vector};
use lazy_init::LazyTransform;
use core::hash::Hash;
use std::collections;
use toml::map::Map as TomlMap;


pub type LockedMap<K, V> = ArcSwap<HashMap<K, V>>;
pub type LockedVec<T> = ArcSwap<Vector<T>>;

pub trait ArcMapExt<K, V> {
    type Error;
    type Inner;
    fn rcu_alter<F>(&self, key: impl Into<K> + Clone, func: F) -> Result<(), ServiceError>
    where
        F: Fn(&mut V) -> Result<(), ServiceError>;
}

impl<K, V> ArcMapExt<K, V> for ArcSwap<HashMap<K, V>>
where
    K: Hash + Eq + Clone,
    V: Clone,
    {
    type Error = ServiceError;
    type Inner = HashMap<K, V>;

    fn rcu_alter<F>(&self, key: impl Into<K> + Clone, func: F) -> Result<(), ServiceError>
    where
        F: Fn(&mut V) -> Result<(), ServiceError>,
    {
        let mut error = Result::Ok(());
        self.rcu(|map_inner| {
            map_inner.alter(
                |value_opt| {
                    let Some(mut value) = value_opt else {
                        error = Err(ServiceError::NotFound);
                        return value_opt;
                    };
                    if let Err(err) = func(&mut value) {
                        error = Err(err);
                    }
                    Some(value)
                },
                key.clone().into(),
            )
        });
        error
    }

}

pub trait TrueOrErr {
    fn or_error<E>(self, error: E) -> Result<(), E>;
}

impl TrueOrErr for bool {
    fn or_error<E>(self, error: E) -> Result<(), E> {
        if self {
            Ok(())
        } else {
            Err(error)
        }
    }
}

pub trait MapExt<K,V> {
    fn join_merge<F>(self, other: Self, func: F) -> Self where F: Fn(&K,V,V) -> V; 
}

impl<K: Hash + Eq + Clone,V> MapExt<K,V> for collections::HashMap<K,V> {
    fn join_merge<F>(mut self, mut other: Self, func: F) -> Self where F: Fn(&K,V,V) -> V {
        self.keys()
            .chain(other.keys())
            .cloned()
            .collect::<HashSet<K>>()
            .into_iter()
            .map(|key| {
                let left_opt = self.remove(&key);
                let right_opt = other.remove(&key);
                match (left_opt, right_opt) {
                    (Some(left), Some(right)) => {
                        let value = func(&key, left, right);
                        (key, value)
                    },
                    (Some(left), None) => (key, left),
                    (None, Some(right)) => (key, right),
                    (None, None) => unreachable!("We source from keys of both maps. At least 1 must contain it.")
                }
            }).collect()
    }
}

impl<K: Hash + Eq + Clone + Ord,V> MapExt<K,V> for TomlMap<K,V> {
    fn join_merge<F>(mut self, mut other: Self, func: F) -> Self where F: Fn(&K,V,V) -> V {
        self.keys()
            .chain(other.keys())
            .cloned()
            .collect::<HashSet<K>>()
            .into_iter()
            .map(|key| {
                let left_opt = self.remove(&key);
                let right_opt = other.remove(&key);
                match (left_opt, right_opt) {
                    (Some(left), Some(right)) => {
                        let value = func(&key, left, right);
                        (key, value)
                    },
                    (Some(left), None) => (key, left),
                    (None, Some(right)) => (key, right),
                    (None, None) => unreachable!("We source from keys of both maps. At least 1 must contain it.")
                }
            }).collect()
    }
}



pub trait ResultFlatten<T,E> {
    fn flatten_(self) -> Result<T,E>;
}

impl<T,E> ResultFlatten<T,E> for Result<Result<T, E>, E> {
    fn flatten_(self) -> Result<T,E> {
        match self {
            Ok(Ok(ok)) => Ok(ok),
            Ok(Err(err)) | Err(err) => Err(err)
        }
    }
}

pub struct LazyInit<T, F: FnOnce() -> T = fn() -> T>(LazyTransform<F,T>);

impl<T, F: FnOnce() -> T> LazyInit<T, F> {
    pub fn get(&self) -> &T {
        self.0.get_or_create(|func| func())
    }

    #[allow(clippy::single_call_fn, clippy::allow_attributes, reason = "just a coincidence. May be used more often later")]
    pub fn new(init: F) -> Self {
        Self(LazyTransform::new(init))
    }

}