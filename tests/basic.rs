use clap_trie::clap_trie;
use clap::{Parser, error::ErrorKind};

mod sub {
    use clap_trie::clap_subcommand;
    clap_subcommand!{
        #[derive(Debug)]
        enum Thingies {
            // #[group(required=true)]
            #[derive(Debug)]
            "list thingy" => {
                pub(crate) id: String,
            },
            #[derive(Debug)]
            "get thingy" => {
                pub(crate) id: String,
            },
            #[derive(Debug)]
            "get thingy attributes" => {
                pub(crate) id: String,
            },
        }
    }

}

clap_trie!{
    #[derive(Debug)]
    enum Subcommands {
        sub::Thingies
    }
}

#[derive(Debug, clap::Parser)]
#[command(name="test")]
struct Cli {
    #[clap(subcommand)]
    subcommand: Subcommands,
}

#[test]
fn basic() {
    let x = Cli::try_parse_from(vec!["test", "get", "thingy", "attributes", "ASDF"]);
    matches!(x.unwrap().subcommand, Subcommands::Thingies(sub::Thingies::GetThingyAttributes(sub::GetThingyAttributesCmd { id: _ })));

    let x = Cli::try_parse_from(vec!["test", "get", "thingy", "ASDF"]);
    matches!(x.unwrap().subcommand, Subcommands::Thingies(sub::Thingies::GetThingy(sub::GetThingyCmd { id: _ })));

    let x = Cli::try_parse_from(vec!["test"]);
    assert_eq!(x.unwrap_err().kind(), ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand);

    let x = Cli::try_parse_from(vec!["test", "asdf"]);
    assert_eq!(x.unwrap_err().kind(), ErrorKind::InvalidSubcommand);

    let x = Cli::try_parse_from(vec!["test", "get"]);
    assert_eq!(x.unwrap_err().kind(), ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand);
}
