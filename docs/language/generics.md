# Generics (v0.4+)

v++ uses **monomorphization**  -  generic functions are specialized at each call site.

```vpp
fn first[T](items: array[T]) -> Option[T] {
    if len(items) == 0 {
        return None
    }
    return Some(items[0])
}
```

Call with explicit type args:

```vpp
let word = first[string](["a", "b"])
```

Example: [`examples/generics.vpp`](../../examples/generics.vpp).
