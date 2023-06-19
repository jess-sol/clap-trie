use std::{collections::HashMap, sync::RwLock};

use convert_case::{Boundary, Case, Casing};
use quote::{quote, quote_spanned};
use syn::{LitStr, parse::{Result, Parse, ParseStream}, Token, punctuated::Punctuated, FieldsNamed, Attribute, braced};
use proc_macro2::{TokenStream, Ident, Span};

struct SubcommandData {
    name: Ident,
    attrs: Vec<Attribute>,
    subcommands: Punctuated<Declaration, Token![,]>
}

impl Parse for SubcommandData {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        input.parse::<Token![enum]>()?;
        let name: Ident = input.parse()?;
        let content;
        braced!(content in input);
        let subcommands = content.parse_terminated(Declaration::parse, Token![,])?;

        Ok(Self { name, attrs, subcommands })
    }
}

impl Parse for Declaration {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let name = input.parse::<LitStr>()?;
        input.parse::<Token![=>]>()?;
        let fields = input.parse::<FieldsNamed>()?;

        Ok(Self { name, attrs, fields })
    }
}

#[derive(Clone)]
struct Declaration {
    name: LitStr,
    attrs: Vec<Attribute>,
    fields: FieldsNamed,
}

pub(crate) fn expand_subcommand(input: TokenStream) -> Result<TokenStream> {
    let data: SubcommandData = syn::parse2(input)?;

    // Output command structs
    let mut cmd_structs = Vec::new();
    let mut cmd_enums = Vec::new();

    let mut cmd_names = HashMap::new();

    for subcommand in data.subcommands {
        let mut struct_name = subcommand.name.value().with_boundaries(Boundary::all().as_slice()).to_case(Case::Pascal);
        let enum_variant_name = ident!(&struct_name);
        struct_name.push_str("Cmd");
        cmd_names.insert(subcommand.name.value(), (enum_variant_name.to_string(), struct_name.clone()));
        let struct_name = ident!(&struct_name);

        cmd_enums.push(quote!{
            #enum_variant_name(#struct_name)
        });

        let atts = subcommand.attrs;
        let fields = subcommand.fields;
        cmd_structs.push(quote! {
            #[derive(clap::Parser)]
            #(#atts)*
            pub struct #struct_name #fields
        });
    }


    let enum_name = data.name;

    // Add data to ALL_DECLARATIONS static, so it can be used in clap-trie proc macro, which needs
    // more information than is provided to expand.
    let res = crate::ALL_DECLARATIONS.get_or_init(|| RwLock::new(HashMap::new()))
        .write().expect("Unable to wrap shared declarations in clap-trie proc macro")
        .insert(enum_name.to_string(), crate::SubcommandEnumDefinition {
            keys: cmd_names
        });
    if res.is_some() {
        return Ok(quote_spanned!{
            enum_name.span() => compile_error!("trie_subcommand names must be globally unique")
        });
    }

    // Get list of trie keys to commands
    let attrs = data.attrs;
    Ok(quote! {
        #(#attrs)*
        pub enum #enum_name {
            #(#cmd_enums,)*
        }

        #(#cmd_structs)*
    })
}
