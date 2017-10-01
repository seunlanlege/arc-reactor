use hyper;

pub struct ArcError(pub String);

impl Into<hyper::Body> for ArcError {
    fn into(self) -> hyper::Body {
        self.0.into()
    }
}

impl<'a> Into<ArcError> for &'a str {
	fn into(self) -> ArcError {
		ArcError(self.to_owned())
	}
}

pub type ArcResult = Result<hyper::Request, ArcError>;
