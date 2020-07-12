use argh::FromArgs;

pub mod cmds;

#[derive(FromArgs, PartialEq, Debug)]
/// Top-level command.
struct TopLevel {
    #[argh(subcommand)]
    nested: cmds::SubCommands,
}

fn main() {
    simple_logger::init().unwrap();
    argh::from_env::<TopLevel>().nested.run();
}
