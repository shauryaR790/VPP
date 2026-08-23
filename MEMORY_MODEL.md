# v++ Memory Model (v0.2 bootstrap)

## Goals

- Predictable behavior for beginners
- Efficient native executables
- Clear C ABI between compiler and runtime
- Extensible toward v1.0 production model

v0.2 does **not** implement a borrow checker or garbage collector.

## Representation

### Stack (by value in LLVM)

| Type | Native representation | Notes |
|------|----------------------|-------|
| `int` | `i64` | Signed 64-bit |
| `float` | `double` | IEEE 64-bit |
| `bool` | `i1` / zero-extended | |
| fixed-size struct | LLVM struct | Field layout computed by codegen |

### Heap (ARC)

| Type | C type | Header |
|------|--------|--------|
| `string` | `VppString*` | `{ char* data; int64_t ref_count; }` |
| `array[T]` | `VppArray*` | `{ void* data; int64_t len; int64_t elem_size; int64_t ref_count; }` |

## String ABI (v0.2)

1. String **literals**  -  compiler emits a nul-terminated `i8*` constant, calls `vpp_string_new(cstr)` → `VppString*`
2. String **locals**  -  stored as `VppString*` (pointer to heap object)
3. **`print(s)`**  -  calls `vpp_print_str(VppString* s)`
4. **`len(s)`**  -  calls `vpp_strlen(VppString* s)`
5. **Concatenation**  -  `vpp_string_concat(VppString* a, VppString* b)` → new `VppString*` (retain inputs during call)

Never pass raw `i8*` to `vpp_print_str`.

## Array ABI (v0.2)

1. **Literals**  -  `vpp_make_array(len, elem_size)` then fill element slots via typed GEP
2. **Value type**  -  `VppArray*` everywhere (locals, parameters, returns)
3. **Elements**  -  inline in buffer; `string` elements store `VppString*` (8 bytes); `bool` uses 1 byte
4. **`len(a)`**  -  `vpp_array_len(VppArray*)`
5. **`a[i]`**  -  `vpp_array_index_ptr(arr, i)` with bounds check (abort on OOB, matches interpreter error)
6. **Ownership**  -  `vpp_array_retain` on pass to functions; `vpp_array_release` at scope exit
7. **String elements**  -  `vpp_string_retain` when stored into array literal

## ARC rules (v0.2 bootstrap)

- `vpp_string_new` / `vpp_make_array`  -  ref_count = 1
- `vpp_*_retain`  -  increment
- `vpp_*_release`  -  decrement; free at zero
- Function arguments: retain on pass (caller keeps ownership)
- Scope exit: release heap locals (v0.2: strings in nested scopes)

Full ARC at scope boundaries is implemented incrementally. Leaks are acceptable in v0.2 for top-level-only programs; crashes are not.

## Interpreter mapping

| Native | Interpreter |
|--------|-------------|
| `VppString*` | `Rc<String>` |
| `VppArray*` | `Rc<Vec<Value>>` |

Semantics must match observable behavior (print output, len, indexing).

## Future (v1.0)

- Move semantics at assignment for heap types
- Struct/enums with explicit layout in spec
- Optional `unsafe` blocks
- No hidden GC
