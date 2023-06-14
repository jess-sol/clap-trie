use std::{collections::HashMap, sync::{OnceLock, RwLock}};

use proc_macro::TokenStream;
use syn::parse_macro_input;

/// Generates a correct set of enums/structs for Clap given a trie of commands.
/// For example:
/// ```ignore
/// trie_command::build {
///   "list devices" -> devices::list_devices,
/// }
/// ```

#[macro_use] mod macros;
mod expand_subcommand;
mod expand_trie;
mod trie;
mod trie_key;

pub(crate) use trie::Trie;
pub(crate) use trie_key::TrieKey;

#[derive(Clone, Debug, Default)]
struct SubcommandEnumDefinition {
    mod_name: String, // Name of the module container command structs
    keys: HashMap<String, String>, // trie path -> command struct name
}

static ALL_DECLARATIONS: OnceLock<RwLock<HashMap<String, SubcommandEnumDefinition>>> = OnceLock::new();

// pub use command::build_subcommands;
// pub use trie::Trie;

#[proc_macro]
pub fn clap_subcommand(input: TokenStream) -> TokenStream {
    let result = match expand_subcommand::expand_subcommand(parse_macro_input!(input)) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error(),
    };
    TokenStream::from(result)
}

#[proc_macro]
pub fn clap_trie(input: TokenStream) -> TokenStream {
    let result = match expand_trie::expand_trie(parse_macro_input!(input)) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error(),
    };
    TokenStream::from(result)
}
