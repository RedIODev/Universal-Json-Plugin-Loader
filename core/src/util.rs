use std::{ops::Deref, sync::Arc};

use arc_swap::{ArcSwap, Guard, RefCnt};
use finance_together_api::cbindings::ServiceError;
use im::HashMap;
use ouroboros::self_referencing;
use std::hash::Hash;

use crate::runtime::{endpoint::Endpoint, event::Event};




pub type LockedMap<K, V> = ArcSwap<HashMap<K, V>>;

pub trait ArcMapExt<K, V> {
    type Inner;
    type Error;
    fn rcu_alter<F>(&self, key: impl Into<K> + Clone, f: F) -> Result<(), ServiceError>
    where
        F: Fn(&mut V) -> Result<(), ServiceError>;

    fn try_rcu<F>(&self, f: F) -> Result<(), Self::Error>
    where
        F: Fn(&Arc<Self::Inner>) -> Result<Self::Inner, Self::Error>;
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
        let mut error = ServiceError::Success;
        self.rcu(|map_inner| {
            map_inner.alter(
                |value_opt| {
                    let Some(mut value) = value_opt else {
                        error = ServiceError::NotFound;
                        return value_opt;
                    };
                    if let Err(err) = f(&mut value) {
                        error = err;
                    }
                    return Some(value);
                },
                key.clone().into(),
            )
        });
        if error == ServiceError::Success {
            Ok(())
        } else {
            Err(error)
        }
    }

    type Inner = HashMap<K, V>;
    type Error = ServiceError;

    fn try_rcu<F>(&self, f: F) -> Result<(), Self::Error>
    where
        F: Fn(&Arc<Self::Inner>) -> Result<Self::Inner, Self::Error>,
    {
        let mut error = ServiceError::Success;
        self.rcu(|map_inner| match f(map_inner) {
            Ok(map) => map,
            Err(err) => {
                error = err;
                (**map_inner).clone()
            }
        });
        if error == ServiceError::Success {
            Ok(())
        } else {
            Err(error)
        }
    }
}

pub trait TrueOrErr {
    fn or_error<E>(self, error: E) -> Result<(), E>;
}

impl TrueOrErr for bool {
    fn or_error<E>(self, error: E) -> Result<(), E> {
        match self {
            true => Ok(()),
            false => Err(error),
        }
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
    fn map<U, F>(self, f: F) -> MappedGuard<T, U>
    where
        F: FnOnce(&T) -> U;

    fn try_map<U, F, E>(self, f: F) -> Result<MappedGuard<T, U>, E>
    where
        F: FnOnce(&T) -> Result<U, E>;
}

impl<T> GuardExt<T> for Guard<T>
where
    T: RefCnt,
{
    fn map<U, F>(self, f: F) -> MappedGuard<T, U>
    where
        F: FnOnce(&T) -> U,
    {
        MappedGuard::new(self, |g| MappedGuardInner { mapped: f(g) })
    }

    fn try_map<U, F, E>(self, f: F) -> Result<MappedGuard<T, U>, E>
    where
        F: FnOnce(&T) -> Result<U, E>,
    {
        MappedGuard::try_new(self, |guard| Ok(MappedGuardInner { mapped: f(guard)? }))
    }
}
