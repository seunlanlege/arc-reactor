# Design

When i began building arc-reactor, i was still learning the rust language.
Coming from a nodejs background, I wanted a web framework that closely mimiced the simplicity of expressjs
the most popular web framework for nodejs.

So I wrote Arc Reactor.

This document is split into three parts:

 - Client dispatch
 - Middlewares
 - proc_macros

## Client Dispatch
In order to achieve high throughput, Arc Reactor uses Multiple threads each running an event loop.
The main thread listens for incoming connections and dipatches them to worker threads sequentially over a futures aware mpsc channel.
Say there are 4 worker threads.

connection #1 goes to thread #1<br/>
connection #2 goes to thread #2<br/>
connection #3 goes to thread #3<br/>
connection #4 goes to thread #4<br/>
connection #5 goes to thread #1

etc.

The raw socket is handed off to hyper's Http::bind_service, as well as a Service trait object. RootService.
bind_service returns a future, that completes when the response returned from the service has been flushed to the client.
If the client drops the connection, the future is dropped as well.

In order to provide things like  a handle to the underlying event loop, and as well as access to the client IP, which hyper removed.
the RootService deconstructs the hyper::Request passed to it, constructs a core::Request and passes it to it's inner ArcHandler.

## ArcService
```rust
pub trait ArcService: ArcServiceClone + Send + Sync {
	fn call(&self, req: Request, res: Response) -> Box<Future<Item=Response, Error=Response>>;
}
```

All Service Handlers in ArcReactor implement this trait


it is a heavily simplified version of tokio's `Service` trait.

```rust
trait Service {
	type Request;
	type Response;
	type Error;
	type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

	fn call(&self, req: Self::Request) -> Self::Future;
}
```
This trait enables a generic way to define Service Handlers.


## ArcHandler
```rust
pub struct ArcHandler {
	pub before: Option<Box<MiddleWare<Request>>>,
	pub handler: Box<ArcService>,
	pub after: Option<Box<MiddleWare<Response>>>,
}

impl ArcService for ArcHandler {}
```
ArcHandler is one of the central pieces to arc reactor.

- [x] Becuase of it's structure, It becomes possible to nest ArcHandlers.
- [x] Nested ArcHandlers inherit their parent's middlewares.
- [x] A request is passed from the `before`(MiddleWare<Request>) to the `handler`(which is also possibly an ArcHandler) and finally to the `after`(MiddleWare<Response>).
- [x] If the MiddleWare<Request> returns an Err(Response), the `handler` is skipped, and the response is forwarded to the `after`(MiddleWare<Response>)*.
- [x] Returning an Err(Response) from a `handler` has no effect, if the `after`(MiddleWare<Response>) exists it would *always* recieve the response returned from the `handler`. Returning an Err(Response) should only be done for testability sake with `FakeReactor`.


## MiddleWare
```rust
pub trait MiddleWare<T: Sized>: MiddleWareClone<T> + Sync + Send {
	fn call(&self, param: T) -> Box<Future<Item = T, Error = Response>>;
}
```
Another one of the central pieces to arc reactor. The middleware trait enables a generic way of defining a MiddleWare.
It does so by providing two types of MiddleWares.

 - [x] `MiddleWare<Request>`
	- [x] This is run before the service handler and has time to do extra processing on the request, e.g the body parser, staticfilserver and multipart parser in contrib were all implemented as `MiddleWare<Request>`
	- [x] A vector of `Vec<Box<MiddleWare<Request>>>` can behave like a single `MiddleWare<Request>`.
	- [x] If a `MiddleWare<Request>` in a `Vec<Box<MiddleWare<Request>>>` returns an Err(Response), the rest of the Middlewares are skipped,
	- [x] Therefore the order matters when mounting MiddleWares.
 - [x] `MiddleWare<Response>`
	- [x] This is run after the service handler, for additional processing to be done on the response.
	- [x] A vector of `Vec<Box<MiddleWare<Request>>>` can behave like a single `MiddleWare<Request>`.
	- [x] If a `MiddleWare<Response>` in a `Vec<Box<MiddleWare<Response>>>` returns an `Err(Response)`, the rest of the Middlewares are skipped,

## Proc Macros
```rust
#[service]
fn ServiceHandler(req: Request, res: Response) {
	Ok(res)
}

#[middleware(Request)]
fn AuthMiddleWare(req: Request) {
	Ok(req)
}

#[middleware(Response)]
fn LoggerMiddleWare(res: Request) {
	Ok(res)
}
```

In order to provide ease of use, and integration with `futures-await`. the `service` and `middleware` proc_macros were introduced.
the `service` proc_macro, creates a zero-sized struct (i.e a struct with no fields, with the same name as the fn), and implements the `ArcService` trait for the struct wrapping its function body in an `async_block`.

the `middleware` proc_macro, creates a zero-sized struct (i.e a struct with no fields, with the same name as the fn), and implements the `MiddleWare<T>` trait for the struct wrapping its function body in an `async_block`.

```rust
pub struct ServiceHandler;

impl ArcService for ServiceHandler {
	fn call(&self, req: Request, res: Response) -> Box<Future<Item=Response, Error=Response> {
		let future = async_block! {
			Ok(req)
		};

		Box::new(future)
	}
}

pub struct AuthMiddleWare;

impl MiddleWare<Request> for AuthMiddleWare {
	fn call(&self, req: Request) -> Box<Future<Item=Request, Error=Response> {
		let future = async_block! {
			Ok(req)
		};

		Box::new(future)
	}
}

pub struct LoggerMiddleWare;

impl MiddleWare<Response> for LoggerMiddleWare {
	fn call(&self, res: Response) -> Box<Future<Item=Response, Error=Response> {
		let future = async_block! {
			Ok(res)
		};

		Box::new(future)
	}
}
```
                       