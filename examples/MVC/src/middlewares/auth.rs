use arc_reactor::prelude::*;

/// some fake future.
fn get_isauth() -> impl Future<Item=bool, Error=()> {
  futures::future::ok(true)
}

/// middlewares must return Result<Request, Response>
/// if a request middleware returns Ok(request)
/// the returned request is passed on to the next middleware in the chain (if there is one)
/// or the route handler.
///
/// If a middleware returns Err(response)
/// that response is forwarded directly to the client
///
///
#[middleware(Request)]
pub fn verify_auth(req: Request) {
  if let Ok(is_auth) = await!(get_isauth()) { // await! is exported in the arc_reactor::prelude.
    if is_auth {
      return Ok(req)
    }
  }

  Err((401, "Unauthorized!").into()) // <T: Serialize> From<(i16, T)> is implemented for Response
}
