# std.process

Run external processes (native codegen supported).

```vpp
import std.process

let code = process.run("echo hello")
```

Use with care  -  sandbox and validate commands in user-facing apps.
