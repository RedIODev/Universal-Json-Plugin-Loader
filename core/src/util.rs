use core::ops::Deref;

use arc_swap::{ArcSwap, Guard, RefCnt};
use plugin_loader_api::ServiceError;
use im::{HashMap, HashSet, Vector};
use lazy_init::LazyTransform;
use ouroboros::self_referencing;
use core::hash::Hash;
use std::collections;

pub type LockedMap<K, V> = ArcSwap<HashMap<K, V>>;
pub type LockedVec<T> = ArcSwap<Vector<T>>;

pub trait ArcMapExt<K, V> {
    type Inner;
    type Error;
    fn rcu_alter<F>(&self, key: impl Into<K> + Clone, f: F) -> Result<(), ServiceError>
    where
        F: Fn(&mut V) -> Result<(), ServiceError>;

    // fn try_rcu<F>(&self, f: F) -> Result<(), Self::Error>
    // where
    //     F: Fn(&Arc<Self::Inner>) -> Result<Self::Inner, Self::Error>;
}

impl<K, V> ArcMapExt<K, V> for ArcSwap<HashMap<K, V>>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    fn rcu_alter<F>(&self, key: impl Into<K> + Clone, f: F) -> Result<(), ServiceError>
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
                    if let Err(err) = f(&mut value) {
                        error = Err(err);
                    }
                    Some(value)
                },
                key.clone().into(),
            )
        });
        error
    }

    type Inner = HashMap<K, V>;
    type Error = ServiceError;

    // fn try_rcu<F>(&self, f: F) -> Result<(), Self::Error>
    // where
    //     F: Fn(&Arc<Self::Inner>) -> Result<Self::Inner, Self::Error>,
    // {
    //     let mut error = Result::Ok(());
    //     self.rcu(|map_inner| match f(map_inner) {
    //         Ok(map) => map,
    //         Err(err) => {
    //             error = Err(err);
    //             (**map_inner).clone()
    //         }
    //     });
    //     error
    // }
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
    fn join_merge<F>(self, other: Self, f: F) -> Self where F: Fn(&K,V,V) -> V; 
}

impl<K: Hash + Eq + Clone,V> MapExt<K,V> for collections::HashMap<K,V> {
    fn join_merge<F>(mut self, mut other: Self, f: F) -> Self where F: Fn(&K,V,V) -> V {
        self.keys()
            .chain(other.keys())
            .cloned()
            .collect::<HashSet<K>>()
            .into_iter()
            .map(|key| {
                let left = self.remove(&key);
                let right = other.remove(&key);
                match (left, right) {
                    (Some(left), Some(right)) => {
                        let value = f(&key, left, right);
                        (key, value)
                    },
                    (Some(left), None) => (key, left),
                    (None, Some(right)) => (key, right),
                    (None, None) => unreachable!("We source from keys of both maps. At least 1 must contain it.")
                }
            }).collect()
    }
}

impl<K: Hash + Eq + Clone + Ord,V> MapExt<K,V> for toml::map::Map<K,V> {
    fn join_merge<F>(mut self, mut other: Self, f: F) -> Self where F: Fn(&K,V,V) -> V {
        self.keys()
            .chain(other.keys())
            .cloned()
            .collect::<HashSet<K>>()
            .into_iter()
            .map(|key| {
                let left = self.remove(&key);
                let right = other.remove(&key);
                match (left, right) {
                    (Some(left), Some(right)) => {
                        let value = f(&key, left, right);
                        (key, value)
                    },
                    (Some(left), None) => (key, left),
                    (None, Some(right)) => (key, right),
                    (None, None) => unreachable!("We source from keys of both maps. At least 1 must contain it.")
                }
            }).collect()
    }
}



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
    mapped: U,
}

pub trait GuardExt<T>: Sized
where
    T: RefCnt,
{
    // fn map<U, F>(self, f: F) -> MappedGuard<T, U>
    // where
    //     F: FnOnce(&T) -> U;

    fn try_map<U, F, E>(self, f: F) -> Result<MappedGuard<T, U>, E>
    where
        F: FnOnce(&T) -> Result<U, E>;
}

impl<T> GuardExt<T> for Guard<T>
where
    T: RefCnt,
{
    // fn map<U, F>(self, f: F) -> MappedGuard<T, U>
    // where
    //     F: FnOnce(&T) -> U,
    // {
    //     MappedGuard::new(self, |g| MappedGuardInner { mapped: f(g) })
    // }

    fn try_map<U, F, E>(self, f: F) -> Result<MappedGuard<T, U>, E>
    where
        F: FnOnce(&T) -> Result<U, E>,
    {
        MappedGuard::try_new(self, |guard| Ok(MappedGuardInner { mapped: f(guard)? }))
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
    pub fn new(init: F) -> Self {
        Self(LazyTransform::new(init))
    }

    pub fn get(&self) -> &T {
        self.0.get_or_create(|f| f())
    }
}