//! A type-agnostic interner
//!
//! Can be used to deduplicate various structures for fast equality comparisons.
//! The design favours flexibility over efficiency, ideally you can throw
//! any type into [Interner] and it'll just work. The main interface is
//! [Interner] and [Tok], [TypedInterner] is provided to cover less likely use
//! cases.
//!
//! ```
//! use std::env;
//! use std::path::PathBuf;
//!
//! use intern_all::{Interner, Tok};
//!
//! let i = Interner::new();
//!
//! // Intern a value
//! let a: Tok<String> = i.i("foo");
//! // Intern a path
//! let b: Tok<PathBuf> = i.i(&env::current_dir().unwrap());
//! ```
//!
//! Some convenience methods are also provided to make working with lists easier
//!
//! ```
//! use intern_all::{Interner, Tok};
//!
//! let i = Interner::new();
//! // Intern a list as a slice of tokens
//! let v1: Tok<Vec<Tok<String>>> =
//!   i.i(&[i.i("bar"), i.i("quz"), i.i("quux")][..]);
//! // Intern a list of internable values
//! let v2: Tok<Vec<Tok<String>>> =
//!   i.iv(["bar".to_string(), "quz".to_string(), "quux".to_string()]);
//! // Intern a list of the borrowed form of internable values
//! let v3: Tok<Vec<Tok<String>>> = i.ibv(["bar", "quz", "quux"]);
//! ```
mod interner;
mod token;
mod typed_interner;

pub use interner::Interner;
pub use token::Tok;
pub use typed_interner::TypedInterner;
