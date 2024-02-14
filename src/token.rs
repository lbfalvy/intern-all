use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::ops::Deref;
use std::sync::{Arc, Weak};
use std::{cmp, fmt};

use trait_set::trait_set;

#[allow(unused)] // for doc
use super::interner::Interner;
use super::typed_interner::TypedInterner;
use crate::global::{self, ev};

trait_set! {
  pub trait Internable = Eq + Hash + Clone + Send + Sync + 'static;
}

/// A shared instance. Equality comparison costs two pointer comparisons.
/// Ordering is by pointer value.
///
/// # Panics
///
/// If an interner was manually constructed, tokens from different interners
/// cannot be compared and attempting to do so causes a panic.
///
/// Since a given [Interner] uses a single [TypedInterner] for each type, this
/// is only possible if an [Interner] or [TypedInterner] was constructed besides
/// the singleton.
#[derive(Clone)]
pub struct Tok<T: Internable> {
  data: Arc<T>,
  interner: Arc<TypedInterner<T>>,
}
impl<T: Internable> Tok<T> {
  /// Create a new token. Used exclusively by the interner
  #[must_use]
  pub(crate) fn new(data: Arc<T>, interner: Arc<TypedInterner<T>>) -> Self {
    Self { data, interner }
  }
  /// The pointer value of the token. If this is equal, equality comparison
  /// succeeds.
  #[must_use]
  pub fn id(&self) -> NonZeroUsize {
    ((self.data.as_ref() as *const T as usize).try_into())
      .expect("Pointer can always be cast to nonzero")
  }
  /// The pointer value of the interner. If this is different, comparison
  /// panics.
  pub fn interner_id(&self) -> NonZeroUsize {
    ((self.interner.as_ref() as *const _ as usize).try_into())
      .expect("Pointer can always be cast to nonzero")
  }
  /// Cast into usize
  #[must_use]
  pub fn usize(&self) -> usize { self.id().into() }
  /// Panic if the two tokens weren't created with the same interner
  pub fn assert_comparable(&self, other: &Self) {
    assert_eq!(self.interner_id(), other.interner_id(), "Tokens must come from the same interner");
  }
  /// Get the typed interner that owns this token.
  pub fn interner(&self) -> Arc<TypedInterner<T>> { self.interner.clone() }

  pub fn i<Q>(q: &Q) -> Self
  where
    Q: ?Sized + Eq + Hash + ToOwned<Owned = T>,
    T: Borrow<Q>,
  {
    global::i(q)
  }
}

impl<T: Internable> Tok<Vec<Tok<T>>> {
  /// Extern all elements of the vector in a new vector. If the vector itself
  /// isn't interned, use [ev]
  pub fn ev(&self) -> Vec<T> { ev(&self[..]) }
}

impl<T: Internable> Tok<Vec<Tok<T>>> {
  /// Add a suffix to the interned vector
  pub fn append(&self, suffix: impl IntoIterator<Item = Tok<T>>) -> Self {
    let i = self.interner();
    i.i(&self.iter().cloned().chain(suffix).collect::<Vec<_>>())
  }

  /// Add a prefix to the interned vector
  pub fn prepend(&self, prefix: impl IntoIterator<Item = Tok<T>>) -> Self {
    let i = self.interner();
    i.i(&prefix.into_iter().chain(self.iter().cloned()).collect::<Vec<_>>())
  }
}

impl<T: Internable> Deref for Tok<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target { self.data.as_ref() }
}

impl<T: Internable + fmt::Debug> fmt::Debug for Tok<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Token({} -> {:?})", self.id(), self.data.as_ref())
  }
}

impl<T: Internable + fmt::Display> fmt::Display for Tok<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", **self) }
}

impl<T: Internable> Eq for Tok<T> {}
impl<T: Internable> cmp::PartialEq for Tok<T> {
  fn eq(&self, other: &Self) -> bool {
    self.assert_comparable(other);
    self.id() == other.id()
  }
}

impl<T: Internable> cmp::Ord for Tok<T> {
  fn cmp(&self, other: &Self) -> cmp::Ordering {
    self.assert_comparable(other);
    self.id().cmp(&other.id())
  }
}
impl<T: Internable> cmp::PartialOrd for Tok<T> {
  fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> { Some(self.cmp(other)) }
}

impl<T: Internable> Hash for Tok<T> {
  fn hash<H: Hasher>(&self, state: &mut H) { state.write_usize(self.usize()) }
}

pub struct WeakTok<T: Internable> {
  data: Weak<T>,
  interner: Weak<TypedInterner<T>>,
}
impl<T: Internable> WeakTok<T> {
  pub fn new(tok: &Tok<T>) -> Self {
    Self { data: Arc::downgrade(&tok.data), interner: Arc::downgrade(&tok.interner) }
  }
  pub fn upgrade(&self) -> Option<Tok<T>> {
    Some(Tok { data: self.data.upgrade()?, interner: self.interner.upgrade()? })
  }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize + Internable> serde::Serialize for Tok<T> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    self.data.serialize(serializer)
  }
}

#[cfg(feature = "serde")]
impl<'a, T: serde::Deserialize<'a> + Internable> serde::Deserialize<'a> for Tok<T> {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'a>,
  {
    T::deserialize(deserializer).map(|t| crate::i(&t))
  }
}
