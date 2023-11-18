use std::cmp::PartialEq;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::ops::Deref;
use std::sync::{Arc, Weak};

#[allow(unused)] // for doc
use super::Interner;
use super::TypedInterner;

/// A number representing an object of type `T` stored in some interner.
/// Equality comparison costs two pointer comparisons. Ordering is by pointer
/// value.
///
/// # Panics
///
/// Comparison panics if values from different interners are involved. Since a
/// given [Interner] uses a single [TypedInterner] for each type, this is only
/// possible if either multiple [Interner] or [TypedInterner] instances are
/// constructed.
#[derive(Clone)]
pub struct Tok<T: Eq + Hash + Clone + 'static> {
  data: Arc<T>,
  interner: Arc<TypedInterner<T>>,
}
impl<T: Eq + Hash + Clone + 'static> Tok<T> {
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
    assert_eq!(
      self.interner_id(),
      other.interner_id(),
      "Tokens must come from the same interner"
    );
  }
  /// Get the interner that owns this token. It is safe to use this even if the
  /// token was created with [Interner].
  pub fn interner(&self) -> Arc<TypedInterner<T>> { self.interner.clone() }
}

impl<T: Eq + Hash + Clone + 'static> Tok<Vec<Tok<T>>> {
  /// Extern all elements of the vector in a new vector. If the vector itself
  /// isn't interned, use [Interner::ev]
  pub fn ev(&self) -> Vec<T> { Interner::ev(&self[..]) }
}

impl<T: Eq + Hash + Clone + Send + Sync + 'static> Tok<Vec<Tok<T>>> {
  /// Add a suffix to the interned vector
  pub fn append<'a>(
    &self,
    i: &Interner,
    suffix: impl IntoIterator<Item = &'a T>,
  ) -> Self {
    let v = (self.iter().cloned())
      .chain(suffix.into_iter().map(|t| i.i(t)))
      .collect::<Vec<_>>();
    i.i(&v)
  }

  /// Add a prefix to the interned vector
  pub fn prepend<'a>(
    &self,
    i: &Interner,
    prefix: impl IntoIterator<Item = &'a T>,
  ) -> Self {
    let v = (prefix.into_iter().map(|t| i.i(t)))
      .chain(self.iter().cloned())
      .collect::<Vec<_>>();
    i.i(&v)
  }
}

impl<T: Eq + Hash + Clone + 'static> Deref for Tok<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target { self.data.as_ref() }
}

impl<T: Eq + Hash + Clone + 'static + Debug> Debug for Tok<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Token({} -> {:?})", self.id(), self.data.as_ref())
  }
}

impl<T: Eq + Hash + Clone + Display + 'static> Display for Tok<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", **self)
  }
}

impl<T: Eq + Hash + Clone + 'static> Eq for Tok<T> {}
impl<T: Eq + Hash + Clone + 'static> PartialEq for Tok<T> {
  fn eq(&self, other: &Self) -> bool {
    self.assert_comparable(other);
    self.id() == other.id()
  }
}

impl<T: Eq + Hash + Clone + 'static> Ord for Tok<T> {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.assert_comparable(other);
    self.id().cmp(&other.id())
  }
}
impl<T: Eq + Hash + Clone + 'static> PartialOrd for Tok<T> {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl<T: Eq + Hash + Clone + 'static> Hash for Tok<T> {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    state.write_usize(self.usize())
  }
}

pub struct WeakTok<T: Eq + Hash + Clone + 'static> {
  data: Weak<T>,
  interner: Weak<TypedInterner<T>>,
}
impl<T: Eq + Hash + Clone + 'static> WeakTok<T> {
  pub fn new(tok: &Tok<T>) -> Self {
    Self {
      data: Arc::downgrade(&tok.data),
      interner: Arc::downgrade(&tok.interner),
    }
  }
  pub fn upgrade(&self) -> Option<Tok<T>> {
    Some(Tok { data: self.data.upgrade()?, interner: self.interner.upgrade()? })
  }
}
