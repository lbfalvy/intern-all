use std::any::{Any, TypeId};
use std::borrow::Borrow;
use std::hash::Hash;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

use hashbrown::HashMap;

use super::token::Tok;
use super::typed_interner::TypedInterner;
use crate::token::Internable;

/// Operations that can be executed on [TypedInterner] without knowing its
/// concrete type
pub trait AnyInterner: Send + Sync {
  fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
  fn sweep(&self) -> usize;
}

impl<T: Internable> AnyInterner for TypedInterner<T> {
  fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> { self }
  fn sweep(&self) -> usize { TypedInterner::sweep(self) }
}

/// A collection of interners based on their type. Can be used to intern any
/// object that implements [ToOwned]. Objects of the same type are stored
/// together in a [TypedInterner]
pub struct Interner {
  interners: Mutex<HashMap<TypeId, Arc<dyn AnyInterner>>>,
}
impl Interner {
  /// Create a new interner
  #[must_use]
  pub fn new() -> Self { Self { interners: Mutex::new(HashMap::new()) } }

  /// Intern something
  #[must_use]
  pub fn i<Q>(&self, q: &Q) -> Tok<Q::Owned>
  where
    Q: ?Sized + Eq + Hash + ToOwned,
    Q::Owned: Internable + Borrow<Q>,
  {
    let mut interners = self.interners.lock().unwrap();
    let interner = get_interner(&mut interners);
    interner.i(q)
  }

  /// Sweep values of a specific type. Useful if you just
  /// constructed a large number of values of a specific type, otherwise use
  /// [Interner::sweep]
  pub fn sweep_t<T: Internable>(&self) -> usize {
    match self.interners.lock().unwrap().get(&TypeId::of::<T>()) {
      None => 0,
      Some(interner) => interner.sweep(),
    }
  }

  /// Sweep all values not referenced by anything other than the interner.
  pub fn sweep(&self) -> usize {
    self.interners.lock().unwrap().values().map(|v| v.sweep()).sum()
  }

  /// Intern a list and its elements. See also [Interner::ibv]
  pub fn iv<T: Internable>(
    &self,
    s: impl IntoIterator<Item = T>,
  ) -> Tok<Vec<Tok<T>>> {
    self.i(&s.into_iter().map(|t| self.i(&t)).collect::<Vec<_>>())
  }

  /// Intern a list of borrowed items. See also [Interner::iv]
  pub fn ibv<'a, Q>(
    &self,
    s: impl IntoIterator<Item = &'a Q>,
  ) -> Tok<Vec<Tok<Q::Owned>>>
  where
    Q: ?Sized + Eq + Hash + ToOwned + 'a,
    Q::Owned: Internable,
  {
    self.i(&s.into_iter().map(|t| self.i(t)).collect::<Vec<_>>())
  }
}

impl Default for Interner {
  fn default() -> Self { Self::new() }
}

/// Get or create an interner for a given type
#[must_use]
fn get_interner<T: Internable>(
  interners: &mut impl DerefMut<Target = HashMap<TypeId, Arc<dyn AnyInterner>>>,
) -> Arc<TypedInterner<T>> {
  let boxed = interners
    .raw_entry_mut()
    .from_key(&TypeId::of::<T>())
    .or_insert_with(|| (TypeId::of::<T>(), TypedInterner::<T>::new()))
    .1
    .clone();
  (Arc::downcast(boxed.as_any_arc()))
    .expect("the typeid is supposed to protect from this")
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  pub fn test_string() {
    let interner = Interner::new();
    let key1 = interner.i("foo");
    let key2 = interner.i(&"foo".to_string());
    assert_eq!(key1, key2)
  }

  #[test]
  pub fn test_slice() {
    let interner = Interner::new();
    let key1 = interner.i(&vec![1, 2, 3]);
    let key2 = interner.i(&[1, 2, 3][..]);
    assert_eq!(key1, key2);
  }

  #[test]
  pub fn test_str_slice() {
    let interner = Interner::new();
    let key1 = interner.iv(["a".to_string(), "b".to_string(), "c".to_string()]);
    let key2 = interner.ibv(vec!["a", "b", "c"]);
    assert_eq!(key1, key2);
  }
}
