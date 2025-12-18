# Architecture

This document describes the internal architecture of the `dbc-rs` library.

## Design Principles

1. **`no_std` First** - The library is designed to work without the standard library, enabling use on embedded targets (Cortex-M, RISC-V, etc.)

2. **Zero Unsafe Code** - The crate uses `#![forbid(unsafe_code)]` to guarantee memory safety at compile time

3. **Minimal Dependencies** - Zero dependencies with `alloc`/`std` features; only `heapless` when using that feature

4. **Immutability** - All data structures are immutable after creation; modifications require builders

5. **Validation at Construction** - All inputs are validated when structures are created, not when accessed

## Feature Flags

| Feature | Default | Requires | Provides |
|---------|---------|----------|----------|
| `std` | ✅ | — | `alloc` + builders, serialization, I/O |
| `alloc` | ❌ | Global allocator | Heap-allocated `Vec`/`String` |
| `heapless` | ❌ | — | Stack-allocated bounded collections |
| `embedded-can` | ❌ | — | `decode_frame()` method |

**Dependency graph:**
```
std ───────► alloc ───────► Heap collections (zero deps)
                  
heapless ─────────────────► Stack collections (one dep: heapless)

embedded-can ─────────────► Frame decoding (one dep: embedded-can)
```

**Rules:**
- You MUST enable exactly one of: `std`, `alloc`, or `heapless`
- `alloc` and `heapless` are **mutually exclusive**
- `std` implicitly enables `alloc`
- `embedded-can` is independent and can combine with any allocation strategy

## Module Structure

```
src/
├── lib.rs              # Crate root, public API exports
├── compat/                 # Abstraction layer for alloc/heapless
│   ├── mod.rs    
│   ├── string.rs           # String<N> wrapper
│   └── vec.rs              # Vec<T, N> wrapper
├── parser/                 # Hand-written zero-copy parser
│   ├── mod.rs              # Parser struct definition
│   ├── impls.rs            # Core parsing methods
│   ├── expect.rs           # Token expectation utilities
│   ├── keyword.rs          # DBC keyword parsing
│   ├── parse.rs            # Parsing trait implementations
│   ├── skip.rs             # Whitespace/comment skipping
│   └── take.rs             # Token extraction
├── dbc/                    # Top-level DBC structure
│   ├── mod.rs              # Dbc struct definition
│   ├── impls.rs            # Core methods (accessors)
│   ├── parse.rs            # Dbc::parse() implementation
│   ├── decode.rs           # CAN message decoding
│   ├── std.rs              # std only features
│   ├── validate.rs         # Validation logic
│   ├── messages.rs         # Messages collection with indexing
│   └── builder/            # DbcBuilder [std only]
├── message/                # CAN message entity
├── signal/                 # Signal entity
├── nodes/                  # Network nodes (ECUs)
├── version/                # VERSION string
├── receivers/              # Signal receivers
├── extended_multiplexing/  # SG_MUL_VAL_ entries
├── value_descriptions/     # VAL_ entries [std only]
├── error/                  # Error types and messages
│   └── lang/               # Localized error strings
└── byte_order.rs           # BigEndian/LittleEndian enum
```

## Entity Module Pattern

Each DBC entity (message, signal, nodes, etc.) follows a consistent module structure:

```
entity/
├── mod.rs          # Entity struct definition, re-exports, constants
├── impls.rs        # Accessor methods (getters), constructors
├── parse.rs        # Parser::parse_entity() implementation
├── std.rs          # std only features
├── validate.rs     # Validation rules [if applicable]
└── builder/        # EntityBuilder [std only]
    └── mod.rs
```

This pattern provides:
- **Separation of concerns** - Each file has a single responsibility
- **Feature isolation** - `std`-only code lives in dedicated files
- **Consistent navigation** - Same structure across all entities

## Compatibility Layer (`compat/`)

The compat module provides type aliases that abstract over `alloc` and `heapless`:

```rust
// With alloc feature:
type Vec<T, const N: usize> = alloc::vec::Vec<T>;
type String<const N: usize> = alloc::string::String;

// With heapless feature:
type Vec<T, const N: usize> = heapless::Vec<T, N>;
type String<const N: usize> = heapless::String<N>;
```

**Key design decisions:**

1. **Unified API** - Both implementations expose the same methods
2. **Capacity parameter** - `N` is always required, even for `alloc` (enables limit enforcement)
3. **Result-based push** - `push()` returns `Result<()>` to handle capacity limits uniformly
4. **Limit enforcement** - Even `alloc::Vec` enforces `N` as a maximum size (DoS protection)

## Parser Architecture

The parser is hand-written (not using parser combinators like `nom`) for several reasons:

1. **Zero dependencies** - No external parser crate needed
2. **Zero-copy** - Returns `&str` slices into the input, no allocations during parsing
3. **`no_std` compatible** - Works without allocator during the parsing phase
4. **Simple error messages** - Direct control over error reporting

```rust
pub struct Parser<'a> {
    input: &'a [u8],  // Original input bytes
    pos: usize,       // Current position
    line: usize,      // Current line number (for errors)
}
```

**Parsing flow:**
```
Input &str
    │
    ▼
Parser::new(input.as_bytes())
    │
    ▼
parse_version() ──► Version
parse_nodes()   ──► Nodes  
parse_messages()──► Vec<Message>
    │                  │
    │                  └──► parse_signals() ──► Vec<Signal>
    ▼
Dbc::new(version, nodes, messages, ...)
    │
    ▼
Validate::validate() ──► Ok(Dbc) or Err(Error)
```

## Build-Time Configuration

Capacity limits are configurable via environment variables at build time:

```bash
DBC_MAX_MESSAGES=512 cargo build ...
```

The `build.rs` script:
1. Reads environment variables
2. Validates values (power of 2 required for `heapless`)
3. Generates `limits.rs` with `const` definitions
4. Included via `include!(concat!(env!("OUT_DIR"), "/limits.rs"))`

| Variable | Default | Description |
|----------|---------|-------------|
| `DBC_MAX_MESSAGES` | 8192 | Maximum messages per DBC |
| `DBC_MAX_SIGNALS_PER_MESSAGE` | 256 | Maximum signals per message |
| `DBC_MAX_NODES` | 256 | Maximum network nodes |
| `DBC_MAX_VALUE_DESCRIPTIONS` | 64 | Maximum value descriptions |
| `DBC_MAX_NAME_SIZE` | 32 | Maximum identifier length |
| `DBC_MAX_EXTENDED_MULTIPLEXING` | 512 | Maximum SG_MUL_VAL_ entries |

## Message Lookup Optimization

The `Messages` struct includes optional indexing for fast ID lookups:

```rust
pub struct Messages {
    messages: Vec<Message, MAX_MESSAGES>,
    
    #[cfg(feature = "heapless")]
    id_index: Option<FnvIndexMap<u32, usize, MAX_MESSAGES>>,  // O(1)
    
    #[cfg(feature = "alloc")]
    sorted_indices: Option<Vec<(u32, usize)>>,  // O(log n) binary search
}
```

- **heapless**: `FnvIndexMap` provides O(1) hash-based lookup
- **alloc**: Sorted vector with binary search provides O(log n) lookup
- **Fallback**: Linear scan O(n) if index building fails

## Decoding Architecture

CAN message decoding supports both basic and extended multiplexing:

```
decode(id, payload, is_extended)
    │
    ├──► Find message by ID (optimized lookup)
    │
    ├──► Validate payload length ≥ DLC
    │
    ├──► Decode multiplexer switches first
    │         │
    │         └──► Store (name, raw_value) pairs
    │
    └──► For each signal:
              │
              ├──► Check multiplexing rules
              │         │
              │         ├──► Extended (SG_MUL_VAL_): Check value ranges
              │         │
              │         └──► Basic (m0, m1...): Check switch == value
              │
              └──► If should_decode:
                        │
                        └──► signal.decode(payload) ──► DecodedSignal
```

**Extended CAN ID handling:**
- DBC stores extended IDs with bit 31 set: `0x80000000 | raw_id`
- `decode(id, payload, is_extended)` adds the flag when `is_extended=true`
- `decode_frame(frame)` (embedded-can) extracts ID type automatically

## Error Handling

Errors use a single enum with string messages for `no_std` compatibility. Parsing errors include optional line number information:

```rust
pub enum Error {
    UnexpectedEof { line: Option<usize> },
    Expected { msg: &'static str, line: Option<usize> },
    InvalidChar { char: char, line: Option<usize> },
    MaxStrLength { max: usize, line: Option<usize> },
    Version { msg: &'static str, line: Option<usize> },
    Message { msg: &'static str, line: Option<usize> },
    Signal { msg: &'static str, line: Option<usize> },
    Nodes { msg: &'static str, line: Option<usize> },
    Receivers { msg: &'static str, line: Option<usize> },
    Decoding(&'static str),      // Runtime decode error (no line info)
    Validation(&'static str),    // Post-parse validation (no line info)
}
```

Error messages are defined as `const` strings in `error/lang/`:
- `en_no_std.rs` - Minimal messages for embedded
- `en_std.rs` - Detailed messages with context

## Testing Strategy

```
tests/
├── integration_tests.rs   # Full DBC parsing scenarios
├── real_world_tests.rs    # Tests with actual DBC files
├── edge_cases.rs          # Boundary conditions
├── proptest_tests.rs      # Property-based testing
└── data/                  # Sample DBC files
    ├── simple.dbc
    ├── complete.dbc
    ├── j1939.dbc
    └── ...
```

Unit tests are co-located with implementation (`#[cfg(test)] mod tests`).

## Performance Considerations

1. **Zero-copy parsing** - Parser returns references to input, no string allocations
2. **Indexed lookups** - O(1) or O(log n) message lookup by ID
3. **Inlined hot paths** - `#[inline]` on frequently called accessors
4. **Early validation** - Fail fast before expensive operations
5. **Pre-allocated collections** - Vec capacity hints reduce reallocations

