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

## Demo

```rust
#![feature(conservative_impl_trait, proc_macro, generators, box_syntax)] // <== need to add this.
extern crate arc_reactor;
#[macro_use]
extern crate serde_json;
use arc_reactor::prelude::*;
use arc_reactor::{Router, ArcReactor, StatusCode};

fn main() {
  ArcReactor::new()
    .routes(rootRoutes())
     .port(3000)
     .initiate()
     .unwrap()
}

fn rootRoutes() -> Router {
  Router::new()
    .get("/", IndexRoute)
}


#[service]
fn IndexRoute(_req: Rrequest, res: Response) {
  let isAuth = await!(fakeFuture());
  if isAuth {
    let payload = json!({
      "data": "hello world"
    });
      
    return Ok(payload.into()) // convert json to json response.
  }
  
  res.with_status(StatusCode::UnAuthorized)
}

fn fakeFuture() -> impl Future<Item=bool, Error=()> {
  futures::future::ok(true)
}

```

## Guides
check out the examples folder and the [api documentation](https://docs.rs/arc-reactor/~0.1) to get a feel for how `arc reactor` works, it's well documented. and i'm terrible at explaining things without using code.
