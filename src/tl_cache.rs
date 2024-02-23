/// Cache any expression while staying in expression scope. The expression gets
/// put in a type-erased thread-local box, and it's downcast and cloned every
/// time the expression is called at runtime.
///
/// # Panics
///
/// If the enclosed expression can have multiple possible types and the
/// point-of-use unifies to a different type than the default. I'm not aware of
/// a way to statically detect this so it's just a panic.
///
/// The most obvious way in which this can happen is with integer literals which
/// default to `i32` but can unify seamlessly to any other integer type, but it
/// could also be caused by autoref/autoderef. In either case, rust-analyzer can
/// tell you the type of both the nested expression and the
#[macro_export]
macro_rules! tl_cache {
  ($data:expr) => {{
    use std::any::Any;
    thread_local! {
      static MY_VALUE: (Box<dyn Any>, &'static str) = {
        let data = Box::new($data);
        let name = std::any::type_name_of_val(&*data);
        (data, name)
      }
    }
    fn failed_downcast_to<T>(found_type: &str) -> T {
      let data_str = stringify!($data);
      let expected = std::any::type_name::<T>();
      panic!("tl_cache called on ambiguous expression!\n{{{data_str}}} defaults to {found_type} but accessed as {expected}")
    }
    MY_VALUE.with(|(v, name)| {
      // the if/else unifies to Option<T> which is the return type of the correct
      // downcast_ref.
      #[allow(dead_code)]
      if false { Some($data) } else { v.downcast_ref().cloned() }
        .unwrap_or_else(|| failed_downcast_to(name))
    })
  }};
}

#[cfg(test)]
mod test {
  #[test]
  fn tl_cache_correct() {
    let my_cached_value = tl_cache!(1);
    assert_eq!(my_cached_value, 1)
  }

  #[test]
  #[should_panic = "tl_cache called on ambiguous expression!\n{1} defaults to i32 but accessed as u64"]
  fn tl_cache_incorrect() { let _: u64 = tl_cache!(1); }
}
