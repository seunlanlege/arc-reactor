# arc reactor

> Asynchronous, Multithreaded, Minimal web framework

- **Asynchronous**. In arc reactor, route handlers are asynchronous by default.

- **Integration With futures-await**. The `#[service]` proc macro not only derives the `ArcService` trait for your route handler, but also marks it as `#[async]` so you can await on futures in  your route handlers with no extra stress.

- **Intuitive Middleware System**. arc reactor exposes a middleware system that is easy to reason about.

- **Minimalistic**. arc reactor is designed to be a very thin abstraction over tokio and hyper.

- **Nightly Rust**. arc reactor uses a lot of cool features, including `proc_macros` which are only available on the nightly channel.

## Installation

add this to your `cargo.toml`
```toml
arc-reactor = "0.1"
```
## Guides
check out the examples folder and the [api documentation](https://docs.rs/arc-reactor/~0.1) to get a feel for how `arc reactor` works, it's well documented. and i'm terrible at explaining things without using code.
