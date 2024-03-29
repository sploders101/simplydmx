# sacn-simplydmx

This folder contains a modified copy of the sacn crate for use in SimplyDMX.
The original crate used an outdated version of the uuid crate, which was not
compatible with the version used in SimplyDMX, so patches have been applied
to mitigate this, allowing SimplyDMX to maintain a consistent CID.

The plan is to eventually merge these changes into the new version of the
crate and use that, but this serves as a short-term stop-gap so I can
continue work on the software until I have an MVP.

# Project moved
This project has moved to https://github.com/RustLight/sacn

# Rust sACN implementation

[![Join the chat at https://gitter.im/rust_sacn/Lobby](https://badges.gitter.im/rust_sacn/Lobby.svg)](https://gitter.im/rust_sacn/Lobby?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)
[![Build Status](https://travis-ci.org/lschmierer/sacn.svg)](https://travis-ci.org/lschmierer/sacn)
[![Crates.io](https://img.shields.io/crates/v/sacn.svg)](https://crates.io/crates/sacn)

[Documentation](https://docs.rs/sacn/)

This is an implementation of the Streaming ACN (ANSI E1.31) network protocol.

Currently only the sending DMX data is implemented.

Parsing of the sACN network packets is allocation free and can work in `no_std`
environments.

## Usage

Add to `Cargo.toml`:

```toml
[dependencies]

sacn = "0.4.4"
```

Create a DmxSource and start sending DMX data to a universe.

```rust
extern crate sacn;
use sacn::DmxSource;

let mut dmx_source = DmxSource::new("Controller").unwrap();

dmx_source.send(1, &[0, 1, 2]);
// ...

// terminate the stream for a specific universe
dmx_source.terminate_stream(1);
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
