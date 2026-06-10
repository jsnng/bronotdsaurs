# Tabular Data Stream Protocol

- a span-based, `no_std + alloc`-compatible pure-rust implementation of Microsoft's tabular data stream protocol. Decoding is zero-alloc for result sets up to 32 columns (wider column metadata spills to the heap via a small-buffer optimization); owned values are allocated only on demand via the [Decode](./decoder/traits.rs) trait.
- Transport-agnostic; operates purely at the protocol layer. Implement the `AsyncTransport` trait to use it over TCP, TLS, MARS, or any async runtime.
- Intentionally kept as dependency-free as possible.

> [!WARNING]
> This crate uses `unsafe` in performance-critical paths (e.g. the streaming decode buffer). These are documented and minimal, but this crate is not `#![forbid(unsafe_code)]`.

# Example

### 1. Implement `AsyncTransport`
```rust
struct TcpTransport(TcpStream);

impl AsyncTransport for TcpTransport {
    type Error = std::io::Error;
    // ...
}
```
### 2. Connect
This implementation manages the session lifecycle via the **typestate pattern** to enforce transitions at compile time.
```rust
let tcp = TcpStream::connect_timeout(&addr, Duration::from_secs(10))?;
let session: Session<InitialState, TcpStream, _> = Session::new(tcp, observer);
// .. do perlogin, login7 and query
```

### 3. Pre-Login/Login

```rust
let prelogin = PreLoginPacketBuilder::default()
  .encryption(PreLoginEncryptionOptions::NotSupported as u8)
  .build()
  .unwrap();

match session.transition(prelogin).await? {
  InitialStateTransition::LoginReady(s) => {
    let login7 = Login7PacketBuilder::default()
      .user_name("sa".to_string())
      .password("password".to_string())
      .server_name("localhost".to_string())
      .build()?;
    match s.transition(login7).await? {
      LoginReadyStateTransition::LoggedIn { session } => { /* ready to query */ }
      LoginReadyStateTransition::AuthenticationRequired { session, errors } => { /* login failed */ }
    }
  }
  #[cfg(feature = "tls")]
  InitialStateTransition::TlsSslNegotiation(s) => {
    // TLS
  }
}
```

### 4. Query
Here is a `sql_batch` example.
```rust
let sql_batch = SQLBatchBuilder::default()
    .all_headers(AllHeaders::new(vec![/* DataStreamHeaderType */]))
    .sql_text("SELECT * FROM sys.tables".to_string()) 
    .build()?;

let transition = session.query(
  sql_batch,
  Attention::new(),
  |col_metadata| { /* column metadata received */ },
  |col_metadata, row| { /* col_metadata iterator to perform row decoding */ },
).await?;
```

or `rpc` example:

```rust
let rpc = SpExecuteSqlBuilder::default()
    .stmt("SELECT @p1".to_string())
    .into_rpc_batch(AllHeaders::new(vec![]));

let transition = session.execute(
  rpc,
  Attention::new(),
  |col_metadata| { /* column metadata received */ },
  |col_metadata, row| { /* col_metadata iterator to perform row decoding */ },
).await?;
```

# Features Flags
### TDS Version
Use the following features to target a specific TDS version.

| Flag | Version |
| --- | --- |
| `"tds7.0"` | 7.0 |
| `"tds7.1"` | 7.1 |
| `"tds7.2"` | 7.2 |
| `"tds7.3"` | 7.3 |
| `"tds7.3a"` | 7.3a |
| `"tds7.3b"` | 7.3b |
| `"tds7.4"` | 7.4 |
| `"tds8.0"` | 8.0 |

### MARS
Enable Session Multiplex Protocol (MARS) support:

| Flag | Description |
| --- | --- |
| `"smp"` | MARS support |
| `"smp2.2"` | MARS 2.2 (implies `"smp"`) |

### Chrono
Enable `chrono` integration for DateTime types:

  - `"chrono"`

### no-std
By default, this crate is `no_std`. To enable `std`, use the feature flag:

  - `"std"`

# Installation

Add to your `Cargo.toml`:
```toml
[dependencies]
bronotdsaurs = { git = "https://github.com/jsnng/bronotdsaurs.git", features = ["tds7.4"] }
```

# Building

```bash
cargo build
```

### Shared Library (with `std`)
Add to `Cargo.toml`:
```toml
[lib]
crate-type = ["cdylib"]
```
Then run the following command:
```bash
cargo build --release --features std
```

### Shared Library (with `no_std`)
```
#TODO
```
# Layout

```
 bronotdsaurs/
 |--- src/
 |    |--- interface/           # impl of unified database layer
 |    |--- smp/                 # "Session Multiplex Protocol" for MARS
 |    |--- tds/
 |    |    |--- session/        # session lifecycle and state machine
 |    |    |--- decoder/        # parses incoming bytes from a MSSQL server into types
 |    |    |--- encoder/        # serialises types into bytes for transport
 |    |    |--- fmt/            # value formatters (datetime, money, guid, decimal)
 |    |    |--- plugins/        # federated auth plugin interface
 |    |    |--- types/          # TDS type definitions
 |    |    |    |--- tokens/    # streaming token definitions
 |    |    |    |--- sp/        # stored procedures (for rpc.rs)
 |    |--- lib.rs
```
