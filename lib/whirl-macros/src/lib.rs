use itertools::Itertools;
use pest_consume::Parser;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal, Span};
use quote::{ToTokens, format_ident};
use syn::{
    DeriveInput, Expr, LitStr, Token, parse::Parse, parse_macro_input, punctuated::Punctuated,
};

use crate::actor::{ActorParser, HandleDeclaration, Rule};

#[proc_macro_derive(Message)]
pub fn derive_message(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote::quote! {
        impl whirl::Message for #ident {}
    };
    output.into()
}

#[proc_macro_derive(Actor)]
pub fn derive_actor(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, attrs, .. } = parse_macro_input!(input);
    let output = quote::quote! {
        impl whirl::Actor for #ident { }
        impl whirl::ActorMarker for #ident { }
        impl whirl::ActorError for #ident { }
    };
    output.into()
}

mod actor;

#[proc_macro]
pub fn actor(input: TokenStream) -> TokenStream {
    let input_string = input.to_string();
    let tree = ActorParser::parse(Rule::program, &input_string)
        .unwrap()
        .single()
        .unwrap();
    let prog = ActorParser::program(tree).unwrap();
    let mut res = String::new();
    for decl in prog {
        let code = match decl {
            actor::Declaration::ActorDeclaration(actor_declaration) => {
                let actor_name = Ident::new(&actor_declaration.name, Span::call_site());
                if actor_declaration.is_pub {
                    quote::quote! {
                        #[derive(whirl_macros::Actor)]
                        pub struct #actor_name;
                    }
                } else {
                    quote::quote! {
                        #[derive(whirl_macros::Actor)]
                        struct #actor_name;
                    }
                }
            }
            actor::Declaration::HandleDeclaration(handle_declaration) => {
                let actor = Ident::new(&handle_declaration.actor, Span::call_site());
                let message = Ident::new(&handle_declaration.message, Span::call_site());
                let response = Ident::new(&handle_declaration.response, Span::call_site());
                let handler_code: syn::Expr =
                    syn::parse_str(&handle_declaration.handler_code).unwrap();
                quote::quote! {
                    #[async_trait::async_trait]
                    impl whirl::Handle<#message> for #actor {
                        type Reply = #response;
                        async fn handle_infallibly(&mut self, msg: #message) -> Self::Reply #handler_code
                    }
                }
            }
            _ => todo!(),
        };
        res.push_str(&code.to_string());
    }
    // panic!("{}", res);
    res.parse().unwrap()
}
