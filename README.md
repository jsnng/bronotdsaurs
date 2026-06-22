# bronotdsaurs

- It is currently scoped for multi-protocol support.

> [!WARNING]
> The public API is unstable and will change frequently. Expect breaking changes between any two commits until v1.0.0. Pin to a specific git revision if you wish to depend on it.

> [!NOTE]
> This project is functional in that the core TDS flows are implemented but it should be considered "early development".

# Layout

```
 bronotdsaurs/
 |--- crates/
 |    |--- bronotdsaurs/          # TDS protocol implementation
 |    |--- interface/             # unified database traits (Connection, Rows, Row)
 |    |--- fedauth/               # federated authentication
 |    |--- derive_proc_macros/    # procedural macros for type conversions
 |--- foundation/
 |    |--- collections/           # hybrid stack/heap buffer (SmallBytes<N>)
 |    |--- traits/                # core Encoder/Decode traits
 |    |--- transport/             # network I/O and TLS abstraction
 |    |--- plugins/               # extensions interface for capabilities requiring external libraries
```