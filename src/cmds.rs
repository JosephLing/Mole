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
use log::{error, info};
use notify::{Watcher, RecursiveMode, watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

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
pub struct InitCommand {
    #[argh(positional, default = "String::from(\"./\")")]
    /// directory to initailize all the site
    current: String,
}

impl InitCommand {
    pub fn run(self) {
        let current = Path::new(&self.current);
        if current.is_dir() {
            info!("init");
        /* we need to write:
            - _layouts
            - _includes
            - _articles
            - _sources (although what is this actually meant to be for)
            - _scss
            - _output
            - .mole.toml -> going to be used to identify the project (just so that clean is safer)
        */
        } else {
            error!("{:?} is not a directory so could not initailize", current);
        }
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

    #[argh(option, default = "false")]
    /// whether or not to check the project for changes and if changed rebuild
    watch: bool,

    #[argh(option, default = "false")]
    /// whether or not to spawn a server to show the website on
    serve: bool,
}

impl BuildCommand {
    pub fn run(mut self) {
        error!("{:?} {:?}", self.watch, self.serve);
        let current = Path::new(&self.current);
        self.dest = current.join(self.dest);
        self.include = current.join(self.include);
        self.layouts = current.join(self.layouts);
        self.articles = current.join(self.articles);
        self.scss = current.join(self.scss);
        if current.is_dir() {
            info!("building");
            mole::Build::new(&self.dest)
                .includes(&self.include, false)
                .includes(&self.layouts, true)
                .articles(&vec![&self.articles, &PathBuf::from(current)])
                .sass(&self.scss)
                .run();

            if self.serve {}

            if self.watch {
                // Create a channel to receive the events.
                let (tx, rx) = channel();

                // Create a watcher object, delivering debounced events.
                // The notification back-end is selected based on the platform.
                let mut watcher = watcher(tx, Duration::from_secs(60)).unwrap();

                // Add a path to be watched. All files and directories at that path and
                // below will be monitored for changes.
                watcher.watch(current, RecursiveMode::Recursive).unwrap();

                loop {
                    match rx.recv() {
                        Ok(event) => {
                            info!("{:?}", event);
                            info!("re-building");
                            mole::Build::new(&self.dest)
                                .includes(&self.include, false)
                                .includes(&self.layouts, true)
                                .articles(&vec![&self.articles, &PathBuf::from(current)])
                                .sass(&self.scss)
                                .run();
                        }
                        Err(e) => error!("watch error: {:?}", e),
                    }
                }
            }
        } else {
            error!(
                "{:?} is not a directory so could not find any files to build from",
                current
            );
        }
    }
}
