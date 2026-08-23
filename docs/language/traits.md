# Traits and impls (v0.4+)

Static dispatch  -  no virtual tables at runtime.

```vpp
trait Display {
    fn to_text(self) -> string
}

struct User {
    name: string,
}

impl Display for User {
    fn to_text(self) -> string {
        return self.name
    }
}

let u = User { name: "Alex" }
print(u.to_text())
```

Example: [`examples/traits.vpp`](../../examples/traits.vpp).
