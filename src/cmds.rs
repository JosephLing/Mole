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
use mole;
use notify::{watcher, RecursiveMode, Watcher};
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use tiny_http::{Request, Response, Server};

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

    #[argh(switch)]
    /// version of the tool
    version: bool,
}

impl InitCommand {
    pub fn run(self) {
        if self.version {
            info!("version: {:?}", env!("CARGO_PKG_VERSION"));
        }

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

// largely copied from cobalt-org/cobalt.rs/src/bin/serve.rs as it's under MIT
fn static_file_handler(dest: &Path, req: Request) -> Result<(), mole::error::CustomError> {
    // grab the requested path
    let mut req_path = req.url().to_string();

    // strip off any querystrings so path.is_file() matches and doesn't stick index.html on the end
    // of the path (querystrings often used for cachebusting)
    if let Some(position) = req_path.rfind('?') {
        req_path.truncate(position);
    }

    // find the path of the file in the local system
    // (this gets rid of the '/' in `p`, so the `join()` will not replace the path)
    let path = dest.to_path_buf().join(Path::new(&req_path[1..]));

    let serve_path = if path.is_file() {
        // try to point the serve path to `path` if it corresponds to a file
        path
    } else {
        // try to point the serve path into a "index.html" file in the requested
        // path
        path.join("index.html")
    };

    if serve_path.exists() {
        let file = fs::File::open(&serve_path)?;
        let content_type =
            if let Some(mime) = mime_guess::MimeGuess::from_path(&serve_path).first_raw() {
                mime.as_bytes()
            } else {
                &b"text/html;"[..]
            };
        req.respond(
            Response::from_file(file).with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], content_type)
                    .expect("Invalid mime type for content found"),
            ),
        )?;
    } else {
        req.respond(
            Response::from_string("<h1>404 page, couldn't find the anything...</h1>")
                .with_status_code(404)
                .with_header(
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html;"[..])
                        .unwrap(),
                ),
        )?;
    }

    Ok(())
}

// largely copied from cobalt-org/cobalt.rs/src/bin/serve.rs as it's under MIT
fn serve(dest: &Path, ip: &str) -> Result<(), mole::error::CustomError> {
    info!("Serving {:?} through static file server", dest);
    info!("Server Listening on http://{}", &ip);
    info!("Ctrl-c to stop the server");

    // attempts to create a server
    let server = Server::http(ip).map_err(|e| mole::error::CustomError(e.to_string()))?;

    for request in server.incoming_requests() {
        if let Err(e) = static_file_handler(&dest, request) {
            error!("{:?}", e);
        }
    }
    Ok(())
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

    #[argh(switch)]
    /// whether or not to check the project for changes and if changed rebuild
    watch: bool,

    #[argh(switch)]
    /// whether or not to spawn a server to show the website on
    serve: bool,

    #[argh(switch)]
    /// version of the tool
    version: bool,
}

impl BuildCommand {
    pub fn run(mut self) {
        if self.version {
            info!("version: {:?}", env!("CARGO_PKG_VERSION"));
        }
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

            if self.serve {
                let dest = Path::new("").join(&self.dest);
                if self.watch {
                    thread::spawn(move || {
                        if let Err(e) = serve(&dest, "127.0.0.1:4000") {
                            error!("{:?}", e);
                        }
                        process::exit(1);
                    });
                } else {
                    if let Err(e) = serve(&dest, "127.0.0.1:4000") {
                        error!("{:?}", e);
                    }
                }
            }

            if self.watch {
                info!("watching for changes in {:?}", self.current);
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
