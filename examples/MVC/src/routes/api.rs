use controllers::api_controller as ApiController;
use arc_reactor::RouteGroup;
use arc_reactor::prelude::*;

/// Composes all the sub-routes and methods for `/api` route.
pub fn new(router: RouteGroup) -> RouteGroup {
    router
    .get("/", index)
    .get("/:username", ApiController::verify_username)
    .post("/auth", ApiController::auth)
    .delete("/verify/:username", ApiController::verify_username)
}

#[service]
fn index(_req: Request, res: Response) {
  
  // A redirect can also be done.
  res.redirect("https://google.com");
}
