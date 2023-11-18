An interner for data of mixed / arbitrary type. It uses weak references and the default allocator so it can be used in long-running processes.

```rust
use std::env;
use std::path::PathBuf;

use intern_all::{Interner, Tok};

let i = Interner::new();

// Intern a value
let a: Tok<String> = i.i("foo");
// Intern a path
let b: Tok<PathBuf> = i.i(&env::current_dir().unwrap());
```

Some convenience methods are also provided to make working with lists easier

```rust
use intern_all::{Interner, Tok};

let i = Interner::new();
// Intern a list as a slice of tokens
let v1: Tok<Vec<Tok<String>>> =
  i.i(&[i.i("bar"), i.i("quz"), i.i("quux")][..]);
// Intern a list of internable values
let v2: Tok<Vec<Tok<String>>> =
  i.iv(["bar".to_string(), "quz".to_string(), "quux".to_string()]);
// Intern a list of the borrowed form of internable values
let v3: Tok<Vec<Tok<String>>> = i.ibv(["bar", "quz", "quux"]);
```