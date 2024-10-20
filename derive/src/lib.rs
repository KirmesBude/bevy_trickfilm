/// Derive macros for bevy_trickfilm
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, DeriveInput};

extern crate proc_macro;

/// Derive macro for AnimationEvent
#[proc_macro_derive(AnimationEvent)]
pub fn derive_animation_event(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);

    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: bevy::ecs::event::Event + bevy::reflect::GetTypeRegistration + bevy::reflect::FromReflect });

    let struct_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics bevy_trickfilm::animation::event::AnimationEvent for #struct_name #type_generics #where_clause {
            fn set_entity(&mut self, entity: Entity) {
                self.entity = entity;
            }
        }
    })
}
