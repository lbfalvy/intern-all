An interner for data of mixed / arbitrary type. It uses weak references and the default allocator so it can be used in long-running processes.

```rust
use std::env;
use std::path::PathBuf;

use intern_all::{i, Tok};

// Intern a value
let a: Tok<String> = i("foo");
// Intern a path
let b: Tok<PathBuf> = i(&env::current_dir().unwrap());
```

Some convenience methods are also provided to make working with lists easier

```rust
use intern_all::{i, ibv, iv, Tok};

// Intern a list as a slice of tokens
let v1: Tok<Vec<Tok<String>>> = i(&[i("bar"), i("quz"), i("quux")][..]);
// Intern a list of internable values
let v2: Tok<Vec<Tok<String>>> =
  iv(["bar".to_string(), "quz".to_string(), "quux".to_string()]);
// Intern a list of the borrowed form of internable values
let v3: Tok<Vec<Tok<String>>> = ibv(["bar", "quz", "quux"]);
assert!(v1 == v2 && v2 == v3)
```

The interner uses weak references but the unreferenced values still take up
space in the token table. To avoid a memory leak, you can periodically
sremove entries referring to unreferenced values from the interner with
`sweep` or `sweep_t`.

```rust
use intern_all::{sweep, sweep_t};

// use this for general housekeeping
sweep();
// use this if a lot of temporary values of a particular interned type
// had been dropped recently
sweep_t::<String>();
```