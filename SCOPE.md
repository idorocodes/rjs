# SCOPE.md — rjs v1 Language Subset

This document is the contract for what `rjs` v1 supports. If something isn't
listed here, it doesn't go into v1 — no exceptions, until v1 ships and this
document gets revised for v2.

Its purpose is to stop scope creep before it starts: every feature added
outside this list delays the point at which the core execution model
(bounded specialization → native codegen → dynamic fallback) is actually
proven.

---

## 1. Values

| Type | Supported | Notes |
|---|---|---|
| Number | ✅ | IEEE 754 double, as in standard JS |
| String | ✅ | UTF-8 backed; no regex methods in v1 |
| Boolean | ✅ | |
| BigInt | ✅ | Backing representation may be simplified (e.g. i128) in v1 |
| null | ✅ | |
| undefined | ✅ | |
| Symbol | ❌ | Deferred to v2 |
| Object | ✅ (fixed-shape only) | See §4 |
| Array | ✅ (basic only) | See §4 |
| Function | ✅ | See §3 |

## 2. Syntax — statements

| Feature | Supported | Notes |
|---|---|---|
| `let` / `const` | ✅ | |
| `var` | ❌ | Use `let`/`const` only in v1 |
| `if` / `else` | ✅ | |
| `while` | ✅ | |
| `for` (C-style) | ✅ | |
| `for...of` / `for...in` | ❌ | Deferred |
| `function` declarations | ✅ | |
| Arrow functions | ❌ | Deferred — adds closure-capture edge cases early |
| `return` | ✅ | |
| `break` / `continue` | ✅ | |
| `switch` | ❌ | Deferred — `if`/`else` covers v1 test programs |
| `try` / `catch` / `throw` | ❌ | Deferred |
| `class` | ❌ | Deferred |
| `import` / `export` | ❌ | Single-file programs only in v1 |

## 3. Syntax — expressions

| Feature | Supported | Notes |
|---|---|---|
| Arithmetic: `+ - * /` | ✅ | `+` includes bounded specialization (§5) |
| Comparison: `== === < > <= >=` | ✅ (`===`/`!==` preferred; `==` may skip coercion edge cases in v1) | |
| Logical: `&& \|\| !` | ✅ | |
| Function calls | ✅ | |
| Closures | ✅ | Required — appears in the fibonacci/recursion test programs |
| Object literals | ✅ | Fixed property set only |
| Array literals | ✅ | Fixed-size access patterns only |
| Property access `obj.prop` | ✅ | |
| Computed access `obj[expr]` | ❌ | Deferred — complicates shape tracking (§4) |
| Template literals | ❌ | Deferred |
| Destructuring | ❌ | Deferred |
| Spread / rest | ❌ | Deferred |
| Optional chaining `?.` | ❌ | Deferred |

## 4. Objects and arrays

- Objects are supported only when their property set is fixed at creation
  and never mutated afterward (no adding/deleting keys post-construction).
  This is what makes shape-based fast-path access (roadmap step 7)
  tractable in v1.
- Arrays support literal construction, indexed read/write, and `.length`.
  Methods like `.map`/`.filter`/`.push` are deferred to v2.
- Prototype chains, `Object.create`, and manual prototype mutation are
  out of scope — programs relying on them fall outside v1 entirely
  rather than being partially supported.

## 5. Bounded specialization signatures (v1)

Per the execution model in the README, each specialized operator gets a
small, fixed set of signatures. For v1, `+` is the only operator with
dedicated fast paths:

- `Number × Number`
- `String × String`
- `Dynamic × Dynamic` (fallback — covers everything else, including
  mixed-type `+`)

Other operators (`- * /` and comparisons) run through the interpreter
path only in v1; they're candidates for their own fast paths in v2 once
the `+` pipeline (steps 5–6 of the roadmap) is proven end to end.

## 6. Explicitly out of scope for v1

These are not "coming later this version" — they are cut, and any
program using them is simply not a v1 program:

- `eval()`
- `Proxy`
- Dynamic property access via variable keys (`obj[key]`)
- Prototype mutation
- `async` / `await`, Promises, the event loop
- `import` / `export`, CommonJS, npm package resolution
- Cross-compilation (`--target` flags) — build-for-host only
- Regular expressions
- Garbage collection beyond reference counting

If a program needs any of the above, it runs in Node, not `rjs`, until a
later version says otherwise.

## 7. v1 acceptance programs

A build is considered a valid v1 if it correctly runs all of the
following, both via `rjs run` and as a binary from `rjs build`:

```js
// 1. Recursion + closures
function fibonacci(n) {
  if (n <= 1) return n;
  return fibonacci(n - 1) + fibonacci(n - 2);
}
console.log(fibonacci(30));

// 2. Mixed-type arithmetic (exercises the Dynamic fallback)
console.log(1 + "1");

// 3. Fixed-shape object access
const user = { name: "Amos", age: 21 };
console.log(user.name);

// 4. Loops + arrays
const nums = [1, 2, 3, 4, 5];
let sum = 0;
for (let i = 0; i < nums.length; i = i + 1) {
  sum = sum + nums[i];
}
console.log(sum);
```

If any of these fail, v1 is not done — regardless of how much of §1–§5
is implemented.

## 8. Changing this document

This file may only be edited between versions (v1 → v2), not mid-build.
If a feature turns out to be genuinely required mid-implementation, that's
a signal to simplify the test program triggering it, not to expand scope.
