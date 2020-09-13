use std::{fmt::Display, path::PathBuf};

#[derive(Debug, PartialEq)]
pub enum CustomError {
    IOError(String),
    LiquidError(String),
}

impl Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomError::IOError(s) => write!(f, "IO error: {}\n", s),
            CustomError::LiquidError(s) => write!(f, "Liquid error: {}\n", s),
        }
    }
}

impl From<std::io::Error> for CustomError {
    fn from(e: std::io::Error) -> Self {
        CustomError::IOError(e.to_string())
    }
}

impl From<liquid::Error> for CustomError {
    fn from(e: liquid::Error) -> Self {
        CustomError::LiquidError(e.to_string())
    }
}

pub fn parse_error_message(
    message: &str,
    path: &PathBuf,
    line: &str,
    start: usize,
    end: usize,
    lineno: i8,
) -> String {
    let spacing = if lineno < 99 {
        "  "
    } else if lineno < 127 {
        "   "
    } else {
        "    "
    };

    let mut underline = String::new();
    for _i in 0..start {
        underline.push(' ');
    }

    for _i in start..end {
        underline.push('^');
    }

    let msg = format!(
        "\n{s   } --> {p} {n}:{start}\n{s   } |\n{n:w$} | {line}\n{s   } | {underline}\n{s   } |\n{s  }{m}",
        p = path.to_str().unwrap(),
        line = line,
        s = spacing,
        w = spacing.len(),
        underline = underline,
        n = lineno,
        start = start,
        m = message
    )
    .to_string();

    msg
}

/*


test.md --> Liquid error: liquid: Unknown variable 'dogs'
  |
2 | hello{{dogs}}
  |      ^^^^^^^^
help: available varaibles=global, layout, page


*/

// we need: to get the line no. and location of the {{varaible}} to be able to print out the line
// help text

// probably have a flag to parse in that allows for full error handling for errors occuring in an include
// this would be for includes and we have two seperate methods for handling this:
// parses along an Optional<HashMap<String, String>> or to just use the `include` varaible to get the path to file
// and just read all the data that we need. The benefit of that is we can even just do a BufReader and only read the
// lines that we need. However it would involve doing more IO......
// doing lots of reads from disk all over again would be clostly if the error was within like default for example
// in that case you would almost want to cache the error so that it didn't propergate everywhere

/*

global_cache



fn parse_liquid_err(error) -> Err{
    if global_cache.contains(error){
        // do some kind of transform here listing that it caused issues to happen multiple times
        return global_cache.get(error)
    }else{
        gloabl_cache.insert(errors::liquid_err_transform(error))
    }
}

Article{
    too_render() -> Result<(), Error> {
        foo.pre_render().map_err()?.pre_render().map_err()?.render().map_err()?;
    }
}



and then

fn render(&self) -> {
    let mut errors = Vec::new();
    for obj in self.articles {
        let mut output_path = self.output.clone();
        output_path.push(PathBuf::from(&obj.url));
        info!("writing to {:?}", output_path);

        match obj.true_render(&global, &parser, &LAYOUTS_THAT_HAVE_ERRORS_IN_THEM_THAT@S_NOT_TODO_WITH_PAGE) {
            Ok(output) => {
                let mut file = File::create(output_path).unwrap();
                file.write_all(output.as_bytes()).unwrap();
            }
            Err(e) => {
                let temp = format!("{}", e);
                if layout error {
                    LAYOUTS_THAT_HAVE_ERRORS_IN_THEM_THAT@S_NOT_TODO_WITH_PAGE.insert(e);
                }else{
                    // it's important for the error to be display here in the context of everything else
                    error!(temp);
                }
            },
        }
    }
    spew out all the errors that are to do with layouts and check for bracktrace here as well!!!
}


*/

// mole --build --backtrace
