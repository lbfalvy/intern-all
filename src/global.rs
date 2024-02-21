use std::borrow::Borrow;
use std::hash::Hash;

use lazy_static::lazy_static;

use crate::interner::Interner;
use crate::token::Internable;
use crate::Tok;

lazy_static! {
  static ref SINGLETON: Interner = Interner::new();
}

/// Create a thread-local token instance and copy it. This ensures that the
/// interner will only be called the first time the expresion is executed,
/// and subsequent calls will just copy the token. Accepts a single static
/// expression (i.e. a literal).
#[macro_export]
macro_rules! i {
  ($name:expr) => {{
    thread_local! {
      static TOKEN: $crate::Tok<String> = $crate::i($name);
    };
    TOKEN.with(|v| v.clone())
  }};
}

/// Intern something with the global interner. If Q is static, you should
/// consider using the macro version of this function.
#[must_use]
pub fn i<Q>(q: &Q) -> Tok<Q::Owned>
where
  Q: ?Sized + Eq + Hash + ToOwned,
  Q::Owned: Borrow<Q> + Internable,
{
  SINGLETON.i(q)
}

/// Fully resolve a list of interned things. If the list is interned, use
/// [Tok#ev]
#[must_use]
pub fn ev<'a, T: Internable>(s: impl IntoIterator<Item = &'a Tok<T>>) -> Vec<T> {
  s.into_iter().map(|t| (**t).clone()).collect()
}

/// Sweep values of a specific type from the global interner. Useful if you just
/// constructed a large number of values of a specific type, otherwise use
/// [sweep]
pub fn sweep_t<T: Internable>() -> usize { SINGLETON.sweep_t::<T>() }

/// Sweep the global interner. If you want to sweep a specific type, try
/// [sweep_t]
pub fn sweep() -> usize { SINGLETON.sweep() }

/// Intern a list and its elements in the global interner. See also [ibv]
pub fn iv<T: Internable>(s: impl IntoIterator<Item = T>) -> Tok<Vec<Tok<T>>> { SINGLETON.iv(s) }

/// Intern a list of borrowed items in the global interner. See also [iv]
pub fn ibv<'a, Q>(s: impl IntoIterator<Item = &'a Q>) -> Tok<Vec<Tok<Q::Owned>>>
where
  Q: ?Sized + Eq + Hash + ToOwned + 'a,
  Q::Owned: Internable,
{
  SINGLETON.ibv(s)
}

#[cfg(test)]
mod test {
  #[test]
  pub fn statics() {
    let a = i!("foo");
    let b = i!("foo");
    assert!(a == b);
  }
}
