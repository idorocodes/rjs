<div align="center">

# rjs

**A JavaScript compiler and runtime, written in Rust, that compiles standard JavaScript into standalone native executables.**

[![Status](https://img.shields.io/badge/status-pre--alpha-orange)]()
[![License](https://img.shields.io/badge/license-TBD-lightgrey)]()
[![Rust](https://img.shields.io/badge/built%20with-Rust-000000?logo=rust)]()

</div>

---

## Overview

`rjs` compiles ordinary JavaScript — no new syntax, no type annotations, no build configuration — into a standalone native binary.

```bash
rjs build app.js
./app
```

The resulting binary runs on its own. No Node.js, no runtime install, no dependencies at execution time.

`rjs` does not introduce a new language and does not require developers to change how they write JavaScript. The goal is to keep JavaScript fully dynamic at the language level while making its execution as statically optimized as possible underneath.

## How it works

JavaScript's dynamic typing means a function like

```js
function add(a, b) {
  return a + b;
}
```

has no single fixed type at compile time — `add` may be called with numbers, strings, bigints, or a mix, each with different semantics. Most JavaScript engines resolve this at runtime via a JIT: interpret, profile, speculatively compile, and deoptimize on mismatch.

`rjs` uses a narrower, ahead-of-time model built on five pieces:

| Mechanism | Description |
|---|---|
| **Bounded specialization** | Each function compiles to a small, fixed set of internal signatures (e.g. `Number × Number`, `String × String`, `Dynamic × Dynamic`) rather than unlimited runtime-generated variants. |
| **Compile-time type inference** | Types that are unambiguous from the source (e.g. `const x = 10`) are resolved before execution, not at runtime. |
| **Runtime type dispatch** | A tagged value representation (`JsValue`) makes runtime type checks cheap where types can't be known statically. |
| **Guards & deoptimization** | Specialized native code verifies its assumptions before running; a failed check redirects execution to a general-purpose interpreter path rather than producing incorrect results. |
| **Dynamic fallback** | Constructs that can't be reasoned about statically — `eval`, `Proxy`, dynamic property access, prototype mutation — always route through a general JavaScript execution path. |

```
     JavaScript source
             │
   compile-time analysis
             │
     ┌───────┴────────┐
     │                 │
 predictable         dynamic
     │                 │
     ▼                 ▼
 native code      JS interpreter
     │                 │
     └───────┬─────────┘
             ▼
         execution
```

The underlying question this project explores:

> How much of dynamically typed JavaScript can be compiled into efficient native code while fully preserving JavaScript semantics?

## Usage

```bash
rjs run app.js              # execute directly via the interpreter
rjs check app.js            # run compile-time analysis only, no execution
rjs build app.js            # produce a standalone native executable
rjs build app.js --release  # build with aggressive optimization
```

## Architecture

```
rjs/
├── rjs-parser/    lexer, AST, and parser for the supported JavaScript subset
├── rjs-runtime/   JsValue representation, interpreter, and bounded dispatch
└── rjs-cli/       the `rjs` binary (run / check / build)
```

`rjs-runtime` implements both the interpreter that every specialized native path falls back to, and the bounded dispatch logic that decides when a native fast path is used instead.

Native code generation is implemented on top of [Cranelift](https://cranelift.dev/), rather than a hand-written machine code emitter.

## Supported subset (v1)

- Values: number, string, boolean, bigint, null, undefined
- Functions and closures
- Control flow: `if`/`else`, `while`, `for`
- Object literals with fixed shapes, and arrays

**Not yet supported:** `eval`, `Proxy`, prototype mutation, `async`/`await`, module resolution (CommonJS/ESM/npm), cross-compilation targets. These are either routed through the dynamic fallback or unsupported until later versions.


## Contributing

This project is in active early development. Contribution guidelines will be published once the core interpreter is functional. Issues and design discussion are welcome in the meantime.
