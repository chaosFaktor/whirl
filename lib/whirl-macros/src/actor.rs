use itertools::Itertools;
use pest_consume::{Node, match_nodes};
use pest_derive::Parser;

use crate::Actor;

type PResult<T> = Result<T, pest_consume::Error<Rule>>;
type PNode<'i> = pest_consume::Node<'i, Rule, ()>;

#[derive(Debug)]
pub(crate) struct ActorDeclaration{pub(crate) is_pub: bool, pub(crate) name: String}
#[derive(Debug)]
pub(crate) struct DeriveDeclaration{pub(crate) derive_tags: Vec<String>, actor_name: String }
#[derive(Debug)]
pub(crate) struct HandleDeclaration {
    pub(crate) actor: String,
    pub(crate) message: String,
    pub(crate) response: String,
    pub(crate) handler_code: String,
}
#[derive(Debug)]
pub(crate) enum Declaration {
    ActorDeclaration(ActorDeclaration),
    HandleDeclaration(HandleDeclaration),
    DeriveDeclaration(DeriveDeclaration),
}

#[derive(Parser)]
#[grammar = "src/grammars/actor.pest"]
pub(crate) struct ActorParser;
#[pest_consume::parser]
impl ActorParser {
    fn EOI(_input: PNode) -> PResult<()> {
        Ok(())
    }
    fn ident(input: PNode) -> PResult<String> {
        Ok(input.to_string())
    }
    fn access_specifier_public(_input: PNode) -> PResult<()> {
        Ok(())
    }
    fn actor_decl(input: PNode) -> PResult<ActorDeclaration> {
        Ok(match_nodes!(input.into_children();
            [ident(i)] => ActorDeclaration{name: i, is_pub: false},
            [access_specifier_public(_),ident(i)] => ActorDeclaration{name: i, is_pub: true}
        ))
    }
    fn rust_like(input: PNode) -> PResult<String> {
        Ok(input.to_string())
    }
    fn handle_decl(input: PNode) -> PResult<HandleDeclaration> {
        Ok(match_nodes!(input.into_children();
        [ident(actor), ident(message), ident(response), rust_like(handler_code)] => HandleDeclaration { actor, message, response, handler_code }
            ))
    }
    fn derive_decl(input: PNode) -> PResult<DeriveDeclaration> {
        Ok(match_nodes!(input.into_children();
            [ident(actor_name),ident(derive_tags)..] => DeriveDeclaration{derive_tags: derive_tags.collect(), actor_name},
        ))
    }

    fn decl(input: PNode) -> PResult<Declaration> {
        Ok(match_nodes!(input.into_children();
            [actor_decl(a)] => Declaration::ActorDeclaration(a),
            [handle_decl(h)] => Declaration::HandleDeclaration(h),
            [derive_decl(d)] => Declaration::DeriveDeclaration(d),
        ))
    }
    pub(crate) fn program(input: PNode) -> PResult<Vec<Declaration>> {
        Ok(match_nodes!(input.into_children();
            [decl(r)..,_] => r.collect(),
        ))
    }
}

// use itertools::Itertools;
// use proc_macro::TokenStream;
//
//
// pub struct Name(pub String);
// pub enum Noun {
//     Actor,
//     Message,
// }
// impl Noun {
//     fn from_str(input: &str) -> Self {
//         match input.to_lowercase().as_str() {
//             "a"|"act"|"actor" => {
//                 Self::Actor
//             }
//             "m"|"msg"|"message" => {
//                 Self::Message
//             }
//             _ => panic!("invalid noun: {:?}", input),
//         }
//     }
// }
// pub enum Verb {
//
// }
// pub struct TypeDefinition(Noun,Name);
//
// fn type_definition(type_name: &str, name: &str) -> TypeDefinition {
//         let noun = Noun::from_str(type_name);
//         let name = Name(name.to_string());
//         TypeDefinition(noun, name)
// }
// fn triple() {
//
// }
//
// pub enum Statement{
//     TypeDefinition(TypeDefinition),
// }
// pub fn statement(line: &str) -> Statement {
//     let sentence = line.split(' ');
//     if let Some([type_name, name]) = sentence.clone().collect_array() {
//         let type_definition = type_definition(type_name, name);
//         Statement::TypeDefinition(type_definition)
//     } else if let Some([subject, predicate, object]) = sentence.clone().collect_array() {
//         let triple =
//         todo!()
//     } else {
//         panic!("invalid sentence: {:?}", sentence.collect_vec())
//     }
// }
//
// pub fn actor(input: TokenStream) -> TokenStream {
//     for line in input.to_string().split(";") {
//         let statement = statement(line);
//     }
//     todo!()
// }
