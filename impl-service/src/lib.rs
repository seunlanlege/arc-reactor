#![feature(proc_macro, box_syntax)]
#![allow(unused_extern_crates)]
#![allow(non_snake_case)]
#![recursion_limit = "128"]
extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;

use quote::{ToTokens, quote_spanned};
use proc_macro2::Span;
use proc_macro::TokenStream;
use syn::*;

#[proc_macro_attribute]
pub fn service(_attribute: TokenStream, function: TokenStream) -> TokenStream {
	let item = syn::parse(function)
		.expect("Well, that didn't work. Must be a syntax error.");
	let ItemFn {
		ident,
		block,
		decl,
		..
	} = match item {
		Item::Fn(item) => item,
		_ => panic!("#[service]: Whoops!, try again. This time, with a function."),
	};

	let block = block.stmts.iter();
	let inputs = decl.inputs.into_tokens();
	let span = Span::call_site();

	let output = quote_spanned! {span=>
		#[derive(Clone)]
		pub struct #ident;
		
		impl ArcService for #ident {
			fn call(&self, #inputs) -> Box<Future<Item = Response, Error = Response>> {
				box async_block! (#(#block)*)
			}
		}
	};

	output.into()
}

#[proc_macro_attribute]
pub fn middleware(attribute: TokenStream, function: TokenStream) -> TokenStream {
	let attribute = attribute.to_string();
	let attribute = attribute.trim_matches(&[' ', '(', ')', '"'][..]);
	if attribute != "Request" && attribute != "Response" {
		panic!("#[Middleware] attribute must be one of 'Request' or 'Response'")
	}
	let attribute = Ident::new(attribute, Span::call_site());
	let item = syn::parse(function)
		.expect("Well, that didn't work. Must be a syntax error");
	let ItemFn {
		ident,
		block,
		decl,
		..
	} = match item {
		Item::Fn(item) => item,
		_ => panic!("#[middleware]: Whoops!, try again. This time, with a function."),
	};


	let block = block.stmts.iter();
	let inputs = decl.inputs.into_tokens();
	let span = Span::call_site();

	let output = quote_spanned! {span=>
		#[derive(Clone)]
		pub struct #ident;

		impl MiddleWare<#attribute> for #ident {
			fn call(&self, #inputs) -> Box<Future<Item=#attribute, Error=Response>> {
				box async_block!(#(#block)*)
			}
		}
	};

	output.into()
}
