use std::borrow::Borrow;
use std::hash::{BuildHasher, Hash};
use std::sync::{Arc, RwLock};

use hashbrown::HashMap;

use super::token::{Tok, WeakTok};
use crate::token::Internable;

/// An interner for any type that implements [Borrow]. Not many optimizations
/// are employed and the interner uses the default allocator. This and the use
/// of weak references means that a long-lived instance can be kept around with
/// regular calls to [TypedInterner::sweep].
pub struct TypedInterner<T: Internable> {
  tokens: RwLock<HashMap<Arc<T>, WeakTok<T>>>,
}
impl<T: Internable> TypedInterner<T> {
  /// Create a fresh interner instance
  #[must_use]
  pub fn new() -> Arc<Self> { Arc::new(Self { tokens: RwLock::new(HashMap::new()) }) }

  /// Get the number of stored values
  pub fn size(self: &Arc<Self>) -> usize { self.tokens.read().unwrap().len() }

  /// Remove entries which are no longer referenced anywhere else
  pub fn sweep(&self) -> usize {
    (self.tokens.write().unwrap()).extract_if(|_, v| v.upgrade().is_none()).count()
  }

  /// Intern an object, returning a token
  #[must_use]
  pub fn i<Q>(self: &Arc<Self>, q: &Q) -> Tok<T>
  where
    Q: ?Sized + Eq + Hash + ToOwned<Owned = T>,
    T: Borrow<Q>,
  {
    let mut tokens = self.tokens.write().unwrap();
    let hash = tokens.hasher().hash_one(q);
    let mut ret: Option<Tok<T>> = None;
    tokens
      .raw_entry_mut()
      .from_hash(hash, |k| <T as Borrow<Q>>::borrow(k) == q)
      .and_replace_entry_with(|_, v| {
        ret = Some((v.upgrade()?).clone());
        Some(v)
      })
      .or_insert_with(|| {
        let keyrc = Arc::new(q.to_owned());
        let token = Tok::<T>::new(keyrc.clone(), self.clone());
        ret = Some(token.clone());
        (keyrc, WeakTok::new(&token))
      });
    ret.expect("One of the above callbacks must have ran")
  }
}
