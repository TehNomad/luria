use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Help,
    Version,
    Q5Design {
        request: PathBuf,
        write_edited: Option<PathBuf>,
    },
}

/// A command line luria can't act on. `message` is one line, no trailing
/// newline and no `luria: ` prefix.
#[derive(Clone, Debug, PartialEq)]
pub struct UsageError {
    pub message: String,
}

/// The usage text `help` prints and every usage error repeats.
pub const USAGE: &str = "usage:
  luria q5 design REQUEST.json [--write-edited DIR]
  luria version
  luria help

Reads a Q5 site-directed mutagenesis request (JSON), runs the workflow and
writes one JSON result document to stdout. See docs/cli.md.
";

/// The reason `word` gives, as a `UsageError`.
fn reject(message: String) -> Result<Command, UsageError> {
    Err(UsageError { message })
}

/// Parse the arguments after the program name.
pub fn parse(arguments: &[String]) -> Result<Command, UsageError> {
    if arguments.is_empty() {
        return reject("no command given".to_string());
    }

    let word: &str = &arguments[0];
    let rest: &[String] = &arguments[1..];
    if word == "help" || word == "--help" || word == "-h" {
        if !rest.is_empty() {
            return reject("'help' takes no arguments".to_string());
        }
        return Ok(Command::Help);
    }
    if word == "version" || word == "--version" {
        if !rest.is_empty() {
            return reject("'version' takes no arguments".to_string());
        }
        return Ok(Command::Version);
    }
    if word == "q5" {
        return parse_q5(rest);
    }
    reject(format!("unknown command '{}'", word))
}

/// Read the words after `q5`: the subcommand, then `design`'s arguments.
fn parse_q5(words: &[String]) -> Result<Command, UsageError> {
    if words.is_empty() {
        return reject("'q5' needs a subcommand: design".to_string());
    }

    let subcommand: &str = &words[0];
    if subcommand != "design" {
        return reject(format!(
            "unknown q5 subcommand '{}'; the only one is 'design'",
            subcommand
        ));
    }
    parse_design(&words[1..])
}

/// Read `q5 design`'s arguments left to right: one request path and an
/// optional `--write-edited` directory. The first error found is the one
/// reported.
fn parse_design(words: &[String]) -> Result<Command, UsageError> {
    let mut request: Option<String> = None;
    let mut directory: Option<String> = None;
    let last: usize = words.len();
    let mut index: usize = 0;

    while index < last {
        let word: &str = &words[index];

        if word == "--write-edited" {
            match directory {
                Some(_) => return reject("--write-edited is given more than once".to_string()),
                None => {
                    index += 1;
                    if index == last {
                        return reject("--write-edited needs a directory".to_string());
                    }

                    let value: &str = &words[index];
                    if value.is_empty() {
                        return reject("an empty path is not a file".to_string());
                    }
                    directory = Some(value.to_string());
                }
            }
        } else if word.starts_with("-") {
            return reject(format!("unknown option '{}'", word));
        } else {
            match request {
                Some(_) => {
                    return reject(format!(
                        "'q5 design' takes one request file, got a second: '{}'",
                        word
                    ))
                }
                None => {
                    if word.is_empty() {
                        return reject("an empty path is not a file".to_string());
                    }
                    request = Some(word.to_string());
                }
            }
        }

        index += 1;
    }

    let write_edited: Option<PathBuf> = directory.map(PathBuf::from);

    match request {
        Some(path) => Ok(Command::Q5Design {
            request: PathBuf::from(path),
            write_edited,
        }),
        None => reject("'q5 design' needs a request file".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_words(words: &[&str]) -> Result<Command, UsageError> {
        let arguments: Vec<String> = words.iter().map(|word| word.to_string()).collect();
        parse(&arguments)
    }

    fn usage(words: &[&str]) -> String {
        match parse_words(words) {
            Err(error) => error.message,
            Ok(command) => panic!("{:?} parsed as {:?}", words, command),
        }
    }

    #[test]
    fn help_and_version_have_their_spellings() {
        for words in [&["help"][..], &["--help"][..], &["-h"][..]] {
            assert_eq!(parse_words(words), Ok(Command::Help), "{:?}", words);
        }
        for words in [&["version"][..], &["--version"][..]] {
            assert_eq!(parse_words(words), Ok(Command::Version), "{:?}", words);
        }
    }

    #[test]
    fn q5_design_takes_one_request_and_an_optional_output_directory() {
        assert_eq!(
            parse_words(&["q5", "design", "req.json"]),
            Ok(Command::Q5Design {
                request: PathBuf::from("req.json"),
                write_edited: None
            })
        );
        let with_dir = Ok(Command::Q5Design {
            request: PathBuf::from("req.json"),
            write_edited: Some(PathBuf::from("out")),
        });
        assert_eq!(
            parse_words(&["q5", "design", "req.json", "--write-edited", "out"]),
            with_dir
        );
        assert_eq!(
            parse_words(&["q5", "design", "--write-edited", "out", "req.json"]),
            with_dir
        );
        // The word after --write-edited is the directory, whatever it is.
        assert_eq!(
            parse_words(&["q5", "design", "req.json", "--write-edited", "-x"]),
            Ok(Command::Q5Design {
                request: PathBuf::from("req.json"),
                write_edited: Some(PathBuf::from("-x")),
            })
        );
    }

    #[test]
    fn each_usage_error_says_what_is_wrong() {
        let cases: [(&[&str], &str); 15] = [
            (&[], "no command given"),
            (&["frobnicate"], "unknown command 'frobnicate'"),
            (&["Help"], "unknown command 'Help'"),
            (&["help", "q5"], "'help' takes no arguments"),
            (&["version", "x"], "'version' takes no arguments"),
            (&["q5"], "'q5' needs a subcommand: design"),
            (
                &["q5", "plan"],
                "unknown q5 subcommand 'plan'; the only one is 'design'",
            ),
            (&["q5", "design"], "'q5 design' needs a request file"),
            (
                &["q5", "design", "a.json", "b.json"],
                "'q5 design' takes one request file, got a second: 'b.json'",
            ),
            (
                &["q5", "design", "a.json", "--write-edited"],
                "--write-edited needs a directory",
            ),
            (
                &[
                    "q5",
                    "design",
                    "a.json",
                    "--write-edited",
                    "o",
                    "--write-edited",
                    "p",
                ],
                "--write-edited is given more than once",
            ),
            (
                &["q5", "design", "a.json", "--write-edited=o"],
                "unknown option '--write-edited=o'",
            ),
            (&["q5", "design", "-v", "a.json"], "unknown option '-v'"),
            (&["q5", "design", ""], "an empty path is not a file"),
            (
                &["q5", "design", "a.json", "--write-edited", ""],
                "an empty path is not a file",
            ),
        ];
        for (words, message) in cases {
            assert_eq!(usage(words), message, "{:?}", words);
        }
    }
}
