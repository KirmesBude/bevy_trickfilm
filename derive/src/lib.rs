/// Derive macros for bevy_trickfilm
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, DeriveInput, Ident};

extern crate proc_macro;

/// Derive macro for AnimationEvent
#[proc_macro_derive(AnimationEvent, attributes(animationevent))]
pub fn derive_animation_event(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);

    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: bevy::ecs::event::Event + bevy::reflect::GetTypeRegistration + bevy::reflect::FromReflect });

    let struct_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    let mut target = None;

    // Only process structs
    if let syn::Data::Struct(ref data_struct) = ast.data {
        // Check the kind of fields the struct contains
        // Structs with named fields
        if let syn::Fields::Named(ref fields_named) = data_struct.fields {
            // Iterate over the fields
            for field in fields_named.named.iter() {
                // Get attributes #[..] on each field
                for attr in field.attrs.iter() {
                    // Parse the attribute
                    if let syn::Meta::List(ref list) = attr.meta {
                        if let Some(ident) = list.path.get_ident() {
                            if ident == "animationevent" {
                                if let Ok(arg) = attr.parse_args::<Ident>() {
                                    if arg == "target" {
                                        if target.is_some() {
                                            panic!("Multiple `#[target] attributes. Only a single target is supported.")
                                        };

                                        target = Some(field.ident.clone());
                                    } else {
                                        panic!("Unknown argument {}", arg);
                                    }
                                } else {
                                    panic!("animationevent attribute needs target arg");
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    match target {
        Some(target) => TokenStream::from(quote! {
            impl #impl_generics bevy_trickfilm::animation::event::AnimationEvent for #struct_name #type_generics #where_clause {
                fn set_target(&mut self, target: EventTarget) {
                    self.#target = target;
                }
            }
        }),
        None => TokenStream::from(quote! {
            impl #impl_generics bevy_trickfilm::animation::event::AnimationEvent for #struct_name #type_generics #where_clause {}
        }),
    }
}
