use clap::{error::ErrorKind, Parser};
use clap_trie::clap_trie;

mod sub {
    use clap_trie::clap_subcommand;
    clap_subcommand!{
        #[derive(Debug)]
        enum Thingies {
            #[derive(Debug)] "list thingy" => { pub(crate) id: String },
            #[derive(Debug)] "get thingy" => { pub(crate) id: String },
            #[derive(Debug)] "get thingy attributes" => { pub(crate) id: String },
        }
    }
}

mod sub2 {
    use clap_trie::clap_subcommand;
    clap_subcommand!{
        #[derive(Debug)]
        enum Other {
            #[derive(Debug)] "list other" => { pub(crate) id: String },
            #[derive(Debug)] "get other" => { pub(crate) id: String },
            #[derive(Debug)] "get other attributes" => { pub(crate) id: String },
        }
    }
}

clap_trie!{
    #[derive(Debug)]
    enum Subcommands {
        sub::Thingies,
        sub2::Other,
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
    matches!(x.unwrap().subcommand, Subcommands::Thingies(sub::Thingies::GetThingyAttributes(sub::thingies::GetThingyAttributes { id: _ })));

    let x = Cli::try_parse_from(vec!["test", "get", "other", "attributes", "ASDF"]);
    matches!(x.unwrap().subcommand, Subcommands::Other(sub2::Other::GetOtherAttributes(sub2::other::GetOtherAttributes { id: _ })));

    let x = Cli::try_parse_from(vec!["test", "get", "thingy", "ASDF"]);
    matches!(x.unwrap().subcommand, Subcommands::Thingies(sub::Thingies::GetThingy(sub::thingies::GetThingy { id: _ })));

    let x = Cli::try_parse_from(vec!["test", "get", "other", "ASDF"]);
    matches!(x.unwrap().subcommand, Subcommands::Other(sub2::Other::GetOther(sub2::other::GetOther { id: _ })));

    let x = Cli::try_parse_from(vec!["test"]);
    assert_eq!(x.unwrap_err().kind(), ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand);

    let x = Cli::try_parse_from(vec!["test", "asdf"]);
    assert_eq!(x.unwrap_err().kind(), ErrorKind::InvalidSubcommand);

    let x = Cli::try_parse_from(vec!["test", "get"]);
    assert_eq!(x.unwrap_err().kind(), ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand);
}
