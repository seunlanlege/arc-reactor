#![feature(proc_macro)]
#![allow(unused_extern_crates)]
#![allow(non_snake_case)]
#![recursion_limit="128"]
extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;
use quote::ToTokens;

use proc_macro::TokenStream;
use syn::*;

#[proc_macro_attribute]
pub fn service(_attribute: TokenStream, function: TokenStream) -> TokenStream {
	let Item { node, .. } = syn::parse(function)
		.expect("failed to parse tokens");
	let ItemFn {
		ident,
		block,
		decl,
		..
	} = match node {
		ItemKind::Fn(item) => item,
		_ => panic!("#[service]: Whoops!, try again. This time, with a function."),
	};
	
	let block = block.stmts.iter();
	let inputs = decl.inputs.into_tokens();
	
	let output = quote! {
		struct #ident;
		
		impl ArcService for #ident {
			fn call(&self, #inputs) -> Box<Future<Item = Response, Error = Error>> {
				Box::new(
					async_block!{
						#(
							#block
						)*
					}
				)
			}
		}
	};

	output.into()
}

#[proc_macro_attribute]
pub fn middleware(_attribute: TokenStream, function: TokenStream) -> TokenStream {
	let Item { node, .. } = syn::parse(function)
		.expect("failed to parse tokens");
	let ItemFn {
		ident,
		block,
		decl,
		..
	} = match node {
		ItemKind::Fn(item) => item,
		_ => panic!("#[middleware]: Whoops!, try again. This time, with a function."),
	};
	
	let block = block.stmts.iter();
	let inputs = decl.inputs.into_tokens();
	
	let output = quote! {
		struct #ident;
		
		impl MiddleWare for #ident {
			fn call(&self, #inputs) -> ArcResult {
				#(
					#block
				)*
			}
		}
	};

	output.into()
}
