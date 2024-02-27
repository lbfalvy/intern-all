//! A type-agnostic interner
//!
//! Can be used to deduplicate various structures for fast equality comparisons.
//! The design favours flexibility over efficiency, ideally you can throw
//! any type into [i] and it'll just work.
//!
//! ```
//! use std::env;
//! use std::path::PathBuf;
//!
//! use intern_all::{i, Tok};
//!
//! // Intern a value
//! let a: Tok<String> = i("foo");
//! // Intern a path
//! let b: Tok<PathBuf> = i(&env::current_dir().unwrap());
//! ```
//!
//! Some convenience methods are also provided to make working with lists
//! easier.
//!
//! ```
//! use intern_all::{i, ibv, iv, Tok};
//!
//! // Intern a list as a slice of tokens
//! let v1: Tok<Vec<Tok<String>>> = i(&[i("bar"), i("quz"), i("quux")][..]);
//! // Intern a list of internable values
//! let v2: Tok<Vec<Tok<String>>> = iv(["bar".to_string(), "quz".to_string(), "quux".to_string()]);
//! // Intern a list of the borrowed form of internable values
//! let v3: Tok<Vec<Tok<String>>> = ibv(["bar", "quz", "quux"]);
//! assert!(v1 == v2 && v2 == v3)
//! ```
//!
//! The interner uses weak references but the unreferenced values still take up
//! space in the token table. To avoid a memory leak, you can periodically
//! sremove entries referring to unreferenced values from the interner with
//! [sweep] or [sweep_t].
//!
//! ```
//! use intern_all::{sweep, sweep_t};
//!
//! // use this for general housekeeping
//! sweep();
//! // use this if a lot of temporary values of a particular interned type
//! // had been dropped recently
//! sweep_t::<String>();
//! ```
//!
//! The functions exposed by this crate have short and not very descriptive
//! names, which may seem like a red flag. In a typical use case, these
//! functions would appear everywhere in the codebase, so this is not a concern.
mod global;
#[warn(unsafe_code)]
mod interner;
mod token;
mod typed_interner;

pub use global::{ev, i, ibv, iv, sweep, sweep_t, get_global, set_global};
pub use token::{Internable, Tok};

pub mod instance {
  //! The interner uses weak references and can be cleared with [sweep], but if
  //! you want to avoid using a global singleton, you can construct an instance
  //! too. Comparing tokens from different interners causes a panic.
  //!
  //! ```
  //! use intern_all::instance::Interner;
  //!
  //! // But you can also create an instance which has methods corresponding to
  //! // all the above functions:
  //! let int = Interner::new();
  //! let k = int.i("foobar");
  //! ```

  pub use super::interner::Interner;
  pub use super::typed_interner::TypedInterner;
}
