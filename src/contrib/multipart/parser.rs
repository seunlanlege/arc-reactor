use bytes::Bytes;
use core::file;
use futures::prelude::*;
use hyper::{self, Body};
use hyperx::header::{ContentDisposition, DispositionParam, DispositionType, Header};
use mime::Mime;
use regex::{
	self,
	bytes::{Regex, RegexBuilder},
};
use std::{collections::HashMap, io, path::PathBuf};
use tokio::fs::File;

#[derive(Debug)]
enum DecodeState {
	Boundary,
	Header,
	Read(CharSet),
}
#[derive(Clone, Debug)]
enum CharSet {
	Utf8(String),
	Binary(String, String),
	None,
}
#[derive(Debug)]
pub enum ParseError {
	Io(io::Error),
	Regex(regex::Error),
	Hyper(hyper::Error),
	InvalidMime,
}

impl From<hyper::Error> for ParseError {
	fn from(err: hyper::Error) -> ParseError {
		ParseError::Hyper(err)
	}
}

#[async]
pub fn parse(
	body: Body,
	boundary: String,
	dir: PathBuf,
	mimes: Option<Vec<Mime>>,
) -> Result<HashMap<String, String>, ParseError> {
	let boundary = format!("--{}", boundary);
	let re = format!("{}*", boundary);
	let mut boundary_regex = RegexBuilder::new(&re);
	let boundary_regex = match boundary_regex.size_limit(5000).build() {
		Ok(regex) => regex,
		Err(err) => {
			error!("[MultiPart Parser] Regex Error: {}", err);
			return Err(ParseError::Regex(err));
		}
	};
	let crlf_regex = Regex::new(r"\r\n").unwrap();
	let mut map = HashMap::new();
	let mut file: Option<File> = None;
	let mut state = DecodeState::Boundary;

	#[async]
	for chunk in body {
		let chunk = Bytes::from(chunk);
		let matches = boundary_regex
			.split(&chunk)
			.map(Bytes::from)
			.collect::<Vec<_>>();

		// invalid UTF8 bytes
		if matches.len() == 1 && matches[0].len() == chunk.len() {
			file = match await!(file::write(file.unwrap(), chunk)) {
				Ok(f) => Some(f),
				Err(err) => {
					error!("[MultiPart Parser] Error writing to file: {}", err);
					return Err(ParseError::Io(err));
				}
			};
			continue;
		}

		for matched in matches {
			let split = crlf_regex
				.split(&matched)
				.map(Bytes::from)
				.collect::<Vec<_>>();

			for parts in split {
				if parts.len() == 0 {
					match state {
						DecodeState::Header => {
							state = DecodeState::Read(CharSet::None);
							continue;
						}
						DecodeState::Read(_) => {
							state = DecodeState::Header;
							continue;
						}
						DecodeState::Boundary => continue,
					};
				}

				match state {
					DecodeState::Header | DecodeState::Boundary => {
						if let Ok(disp) = ContentDisposition::parse_header(&parts.to_vec().into()) {
							if let DispositionType::Ext(ref contentType) = disp.disposition {
								if contentType.to_lowercase().contains("content-type") {
									let c_t = contentType.get(14..);
									let mime = c_t.unwrap().to_string();
									// map.insert("content-type".into(), mime.clone());
									if let Some(ref mimes) = mimes {
										let mime = mime.parse::<Mime>();
										if !mime.is_ok() || !mimes.contains(&mime.unwrap()) {
											error!("[MultiPart Parser] Unsupported Mime!");
											return Err(ParseError::InvalidMime);
										}
									}
								}
							}

							for param in disp.parameters {
								match param {
									DispositionParam::Ext(_, name) => {
										map.insert("--param".into(), name);
									}
									DispositionParam::Filename(_, _, filename) => {
										let filename = String::from_utf8(filename).unwrap();
										map.insert("--file".into(), filename);
									}
								}
							}
						}
						state = DecodeState::Header;
					}

					DecodeState::Read(_) => {
						match (map.remove("--file"), map.remove("--param")) {
							(Some(filename), Some(param)) => {
								let mut path = dir.clone();
								path.push(&filename);
								file = match await!(File::create(path)) {
									Ok(f) => Some(f),
									Err(err) => {
										error!("[MultiPart Parser] Error creating file: {}", err);
										return Err(ParseError::Io(err));
									}
								};
								state = DecodeState::Read(CharSet::Binary(filename, param));
							}
							(None, Some(param)) => {
								state = DecodeState::Read(CharSet::Utf8(param));
							}
							_ => {}
						};

						let charset = match state {
							DecodeState::Read(ref r) => r.clone(),
							_ => unreachable!(),
						};

						if &parts[..] != b"--" {
							match charset {
								CharSet::Utf8(param) => {
									let val = String::from_utf8_lossy(&parts);
									map.entry(param.clone()).or_insert("".into()).push_str(&val);
								}
								CharSet::Binary(filename, param) => {
									file = match await!(file::write(file.unwrap(), parts)) {
										Ok(f) => Some(f),
										Err(err) => {
											error!(
												"[MultiPart Parser] Error writing to file: {}",
												err
											);
											return Err(ParseError::Io(err));
										}
									};

									if let None = map.get(&param) {
										map.insert(param, filename);
									}
								}
								_ => {}
							}
						}
					}
				}
			}
		}
	}

	Ok(map)
}
