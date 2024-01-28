# Contributing

Contributions are welcome and help greatly improve the app.

## Local Run

To get started, you'll need:
- The Rust toolchain
- `cargo-shuttle`

To install the Rust toolchain [click here](https://rustup.rs) and follow the appropriate instructions for your operating system.
To install `cargo-shuttle`, run `cargo install cargo-shuttle`.

Once you have all the prerequisites, to build, simply run `cargo shuttle run`.
This will compile any necessary dependencies and run the site on [localhost port 8000](http://localhost:8000).
From there, there is one more step necessary to begin local development.
You must enter the `main.js` file and change out `wss://` with `ws://`.
This is because while the hosting service provides a secure connection, localhost does not.
Once you have done this you will be able to use the websocket without receiving an error.
