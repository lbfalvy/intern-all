use std::borrow::Borrow;
use std::hash::Hash;
use std::sync::OnceLock;

use crate::interner::Interner;
use crate::token::Internable;
use crate::Tok;

static SINGLETON: OnceLock<&'static Interner> = OnceLock::new();

/// Obtain a reference to the singleton
pub fn get_global() -> &'static Interner {
  SINGLETON.get_or_init(|| Box::leak(Box::new(Interner::new())))
}

/// Set the global interner if it hasn't been used yet. 
pub fn set_global(i: impl Fn() -> &'static Interner) -> bool {
  let mut done = false;
  SINGLETON.get_or_init(|| {
    done = true;
    i()
  });
  done
}

/// Create a thread-local token instance and copy it. This ensures that the
/// interner will only be called the first time the expresion is executed,
/// and subsequent calls will just copy the token. Accepts a single static
/// expression (i.e. a literal).
#[macro_export]
macro_rules! i {
  ($ty:ty : $expr:expr) => {{
    thread_local! {
      static VALUE: $crate::Tok<<$ty as ToOwned>::Owned> = $crate::i($expr as &$ty);
    }
    VALUE.with(|v| v.clone())
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
  get_global().i(q)
}

/// Fully resolve a list of interned things. If the list is interned, use
/// [Tok::ev]
#[must_use]
pub fn ev<'a, T: Internable>(s: impl IntoIterator<Item = &'a Tok<T>>) -> Vec<T> {
  s.into_iter().map(|t| (**t).clone()).collect()
}

/// Sweep values of a specific type from the global interner. Useful if you just
/// constructed a large number of values of a specific type, otherwise use
/// [sweep]
pub fn sweep_t<T: Internable>() -> usize { get_global().sweep_t::<T>() }

/// Sweep the global interner. If you want to sweep a specific type, try
/// [sweep_t]
pub fn sweep() -> usize { get_global().sweep() }

/// Intern a list and its elements in the global interner. See also [ibv]
pub fn iv<T: Internable>(s: impl IntoIterator<Item = T>) -> Tok<Vec<Tok<T>>> { get_global().iv(s) }

/// Intern a list of borrowed items in the global interner. See also [iv]
pub fn ibv<'a, Q>(s: impl IntoIterator<Item = &'a Q>) -> Tok<Vec<Tok<Q::Owned>>>
where
  Q: ?Sized + Eq + Hash + ToOwned + 'a,
  Q::Owned: Internable,
{
  get_global().ibv(s)
}

#[cfg(test)]
mod test {
  use std::any::{type_name, type_name_of_val};

  use super::i;
  use crate::Tok;

  #[test]
  pub fn statics() {
    let a = i!(str: "foo");
    let b = i!(str: "foo");
    let c = i("foo");
    assert_eq!(a, b);
    assert_eq!(a, c);
    let v = i!([Tok<String>]: &[i("foo"), i("bar"), i("baz")]);
    assert_eq!(type_name_of_val(&v), type_name::<Tok<Vec<Tok<String>>>>());
  }

  #[test]
  pub fn basics() {
    let a1 = i!(str: "foo");
    let a2 = i!(str: "foo");
    let b = i!(str: "bar");
    assert_eq!(a1, a2);
    assert_ne!(a1, b);
  }
}
