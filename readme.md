Clap Trie
===

Build clap subcommands with Tries! Allows you to structure your apps subcommands separately from how you layout the enums for parsing those subcommands.

For example, if you structured your app as:

```rust
#[derive(clap::Parser)]
struct Cli {
    #[clap(Subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    Person(PersonCommand),
    Place(PlaceCommand),
}

mod people {
    #[derive(clap::Subcommand)]
    enum PersonCommand {
        List,
        Get { name: String }
    }
}

mod place {
    #[derive(clap::Subcommand)]
    enum PlaceCommand {
        List,
        Get { name: String },
        Create { name: String }
    }
}
```

You'd be stuck with your subcommands structured as:

```console
prog person get
prog person list
prog place create
```

With Clap trie, you instead can define your subcommands into a trie of space separated keys:

```rust
mod people {
    clap_subcommand! {
        enum PeopleCommand {
            "get person" => { name: String },
            "list people",
        }
    }
}

mod place {
    clap_subcommand! {
        enum PlaceCommand {
            "list places" => {},
            "get place" => { name: String },
            "create place" => { name: String },
        }
    }
}

clap_trie! {
    enum Command {
        mod::people::PeopleCommand,
        mod::place::PlaceCommand,
    }
}
```

Now you can use the commands as written in the macro!

```console
prog get person
prog list people
prog create place
```

While still parsing using the enum format:

```rust
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::PeopleCommand(command) => parse_people_command(command,
        Command::PlaceCommand(command) => parse_place_command(command),
    }
}

fn parse_people_command(command: PeopleCommand) {
    match command {
        PeopleCommand::GetPerson(GetPersonCmd { name }) => {},
        PeopleCommand::ListPeople(_) => {},
    }
}

fn parse_place_command(command: PlaceCommand) {
    match command {
        PeopleCommand::ListPlace(_) => {},
        PeopleCommand::GetPlace(GetPlaceCmd { name }) => {},
        PeopleCommand::CreatePlace(CreatePlaceCmd { name }) => {},
    }
}
```
