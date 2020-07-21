/**
- cmd line tools
    - cmds:
        - init - todo: toml config?
        - new - advanced/not needed yet
        - build - flags:optimise, -input and output
        - clean - advanced maybe? depends on how much we define things
        - server ADVANCED

*/
use argh::FromArgs;
use log::{info, error};
use std::path::{Path, PathBuf};

use mole;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum SubCommands {
    INIT(InitCommand),
    BUILD(BuildCommand),
    // CLEAN(CleanCommand),
    // NEW(NewCommand),
    // SERVE(ServeCommand)
}

impl SubCommands {
    pub fn run(self) {
        match self {
            SubCommands::INIT(x) => x.run(),
            SubCommands::BUILD(x) => x.run(),
        }
    }
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "init", description = "setups project")]
pub struct InitCommand {}

impl InitCommand {
    pub fn run(self) {
        unimplemented!("just working on build for now")
    }
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(
    subcommand,
    name = "build",
    description = "builds the static site from source"
)]
pub struct BuildCommand {
    #[argh(positional, default = "String::from(\"./\")")]
    /// path to output too
    current: String,

    #[argh(option, default = "PathBuf::from(\"_output/\")")]
    /// path to output too
    dest: PathBuf,

    #[argh(option, default = "PathBuf::from(\"_source/\")")]
    /// path to build from
    source: PathBuf,

    #[argh(option, default = "PathBuf::from(\"_include/\")")]
    /// path from 'source' to include folder
    include: PathBuf,

    #[argh(option, default = "PathBuf::from(\"_layouts/\")")]
    /// path from 'source' to layouts folder
    layouts: PathBuf,

    #[argh(option, default = "PathBuf::from(\"_articles/\")")]
    /// path from 'source' to articles folder
    articles: PathBuf,

    #[argh(option, default = "PathBuf::from(\"_css/\")")]
    /// path from 'source' to articles folder
    scss: PathBuf,
}

impl BuildCommand {
    pub fn run(self) {
        let current = Path::new(&self.current);
        if current.is_dir(){
            info!("building");
            mole::Build::new(current.join(self.dest))
                .includes(&vec![current.join(self.include)], false)
                .includes(&vec![current.join(self.layouts)], true)
                .articles(&vec![
                    current.join(self.articles),
                    current.join(self.source),
                    PathBuf::from(current),
                ])
                .sass(&vec![current.join(self.scss)])
                .run();
        }else{
            error!("{:?} is not a directory so could not find any files to build from", current);
        }
        
    }
}
