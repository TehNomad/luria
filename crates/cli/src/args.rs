use std::path::PathBuf;

use crate::interpret::run::TextSource;
use luria_nl::endpoint::Endpoint;

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Help,
    Version,
    Q5Design {
        request: PathBuf,
        write_edited: Option<PathBuf>,
    },
    Q5Interpret {
        template: PathBuf,
        text: TextSource,
        design: Option<PathBuf>,
        endpoint: String,
        model: String,
        allow_remote: bool,
        write_request: Option<PathBuf>,
    },
    Review {
        request: PathBuf,
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
  luria q5 interpret --template FILE.gb (--text TEXT | --text-file FILE)
      --backend openai-compatible --endpoint URL --model NAME
      [--design PROFILE.json] [--allow-remote] [--write-request FILE.json]
  luria review REQUEST.json
  luria version
  luria help

'q5 design' reads a Q5 site-directed mutagenesis request (JSON), runs the
workflow and writes one JSON result document to stdout. 'q5 interpret'
asks a language model what a request in words means and writes the
proposed request, or its questions, as one JSON document. 'review' runs a
request and shows its result in the terminal. See docs/cli.md.
";

/// The reason `word` gives, as a `UsageError`.
fn reject(message: String) -> Result<Command, UsageError> {
    Err(UsageError { message })
}

/// The command line's words as text. A word that isn't valid UTF-8 is a
/// usage error naming its 1-based position after the program name.
pub fn utf8_arguments(arguments: Vec<std::ffi::OsString>) -> Result<Vec<String>, UsageError> {
    let mut words: Vec<String> = [].to_vec();
    let mut position: usize = 0;

    for word in arguments {
        position += 1;
        match word.into_string() {
            Ok(text) => words.push(text),
            Err(_) => {
                return Err(UsageError {
                    message: format!("argument {} is not valid UTF-8", position),
                })
            }
        }
    }

    Ok(words)
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
    if word == "review" {
        return parse_review(rest);
    }
    reject(format!("unknown command '{}'", word))
}

/// Read the words after `q5`: the subcommand, then that subcommand's
/// arguments.
fn parse_q5(words: &[String]) -> Result<Command, UsageError> {
    if words.is_empty() {
        return reject("'q5' needs a subcommand: design or interpret".to_string());
    }

    let subcommand: &str = &words[0];
    if subcommand == "interpret" {
        return parse_interpret(&words[1..]);
    }
    if subcommand != "design" {
        return reject(format!(
            "unknown q5 subcommand '{}'; they are 'design' and 'interpret'",
            subcommand
        ));
    }
    parse_design(&words[1..])
}

/// Read `q5 design`'s arguments left to right: one request path and an
/// optional `--write-edited` directory. A word exactly `--` ends the options,
/// so every later word is the request path, even one that starts with `-`,
/// and `--` itself is not a path. The first error found is the one reported.
fn parse_design(words: &[String]) -> Result<Command, UsageError> {
    let mut request: Option<String> = None;
    let mut directory: Option<String> = None;
    let mut options_end: bool = false;
    let last: usize = words.len();
    let mut index: usize = 0;

    while index < last {
        let word: &str = &words[index];

        if word == "--" && !options_end {
            options_end = true;
            index += 1;
            continue;
        }

        if word == "--write-edited" && !options_end {
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
        } else if !options_end && word.starts_with("-") {
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

/// Read `review`'s arguments left to right: one request path. A word exactly
/// `--`, the first time, ends the options, so every later word is the request
/// path, even one that starts with `-`, and `--` itself is not a path. Before
/// `--`, a word starting with `-` is an unknown option. The first error found
/// is the one reported (milestone 7a).
fn parse_review(words: &[String]) -> Result<Command, UsageError> {
    let mut request: Option<String> = None;
    let mut options_end: bool = false;
    let last: usize = words.len();
    let mut index: usize = 0;

    while index < last {
        let word: &str = &words[index];

        if word == "--" && !options_end {
            options_end = true;
            index += 1;
            continue;
        }

        if !options_end && word.starts_with("-") {
            return reject(format!("unknown option '{}'", word));
        }

        match request {
            Some(_) => {
                return reject(format!(
                    "'review' takes one request file, got a second: '{}'",
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

        index += 1;
    }

    match request {
        Some(path) => Ok(Command::Review {
            request: PathBuf::from(path),
        }),
        None => reject("'review' needs a request file".to_string()),
    }
}

/// Whether `option` names a file or directory, whose value cannot be empty.
fn path_option(option: &str) -> bool {
    matches!(
        option,
        "--template" | "--text-file" | "--design" | "--write-request"
    )
}

/// The value `given` holds for `option`, when it was given at all.
fn given_value(given: &[(String, String)], option: &str) -> Option<String> {
    given
        .iter()
        .find(|pair| pair.0 == option)
        .map(|pair| pair.1.clone())
}

/// Read `q5 interpret`'s words left to right: each value option takes the
/// next word as its value, whatever it is, and `--allow-remote` takes none.
/// The first error found is the one reported. An empty path and an empty
/// model name are refused with the words; an empty `--text` is not, because
/// `interpret` reports it as `text_empty`. After the words, the checks run
/// in the order plan §5.7 and §3 name: what must be present, and which
/// backend and endpoint are known.
fn parse_interpret(words: &[String]) -> Result<Command, UsageError> {
    let mut given: Vec<(String, String)> = [].to_vec();
    let mut allow_remote: bool = false;
    let last: usize = words.len();
    let mut index: usize = 0;

    while index < last {
        let word: &str = &words[index];

        if word == "--allow-remote" {
            if allow_remote {
                return reject("--allow-remote is given more than once".to_string());
            }
            allow_remote = true;
            index += 1;
            continue;
        }

        match word {
            "--template" | "--text" | "--text-file" | "--design" | "--backend" | "--endpoint"
            | "--model" | "--write-request" => {
                if given.iter().any(|pair| pair.0 == word) {
                    return reject(format!("{} is given more than once", word));
                }

                index += 1;
                if index == last {
                    return reject(format!("{} needs a value", word));
                }

                let value: String = words[index].to_string();
                if value.is_empty() && word == "--model" {
                    return reject("the model name is empty".to_string());
                }
                if value.is_empty() && path_option(word) {
                    return reject("an empty path is not a file".to_string());
                }
                given.push((word.to_string(), value));
            }
            _ => {
                if word.starts_with("-") {
                    return reject(format!("unknown option '{}'", word));
                }
                return reject(format!("'q5 interpret' takes options only, got '{}'", word));
            }
        }

        index += 1;
    }

    let template: PathBuf = match given_value(&given, "--template") {
        None => return reject("'q5 interpret' needs --template".to_string()),
        Some(path) => PathBuf::from(path),
    };
    let text: TextSource = match (
        given_value(&given, "--text"),
        given_value(&given, "--text-file"),
    ) {
        (None, None) => return reject("'q5 interpret' needs --text or --text-file".to_string()),
        (Some(_), Some(_)) => return reject("give --text or --text-file, not both".to_string()),
        (Some(value), None) => TextSource::Inline(value.to_string()),
        (None, Some(path)) => TextSource::File(PathBuf::from(path)),
    };
    let backend: String = match given_value(&given, "--backend") {
        None => return reject("'q5 interpret' needs --backend".to_string()),
        Some(kind) => kind,
    };
    if backend != "openai-compatible" {
        return reject(format!(
            "unknown backend '{}'; this build has only 'openai-compatible' (M6-18)",
            backend
        ));
    }
    let endpoint: String = match given_value(&given, "--endpoint") {
        None => return reject("'q5 interpret' needs --endpoint".to_string()),
        Some(url) => url,
    };
    let model: String = match given_value(&given, "--model") {
        None => return reject("'q5 interpret' needs --model".to_string()),
        Some(name) => name,
    };
    let design: Option<PathBuf> = given_value(&given, "--design").map(PathBuf::from);
    let write_request: Option<PathBuf> = given_value(&given, "--write-request").map(PathBuf::from);

    Ok(Command::Q5Interpret {
        template,
        text,
        design,
        endpoint,
        model,
        allow_remote,
        write_request,
    })
}

/// The endpoint a `q5 interpret` command line may call, or why not (plan §3,
/// M6-18). Decided from the text alone, before anything is read or sent.
pub fn checked_endpoint(url: &str, allow_remote: bool) -> Result<Endpoint, UsageError> {
    let endpoint: Endpoint = match Endpoint::parse(url) {
        Err(problem) => {
            return Err(UsageError {
                message: format!("the endpoint is not usable: {}", problem.message),
            })
        }
        Ok(found) => found,
    };

    if !allow_remote && !endpoint.is_loopback() {
        return Err(
            UsageError {
                message: format!(
                    "the endpoint {} is not on this machine; sending the request there needs --allow-remote, and plain HTTP to another machine is unencrypted",
                    endpoint.url()
                ),
            },
        );
    }
    Ok(endpoint)
}

/// The bearer key to send (D045): `key` unchanged, unless it is non-empty
/// and the endpoint is not loopback, which is refused, because this build
/// speaks plain HTTP only (M6-18) and the key would cross the network
/// unencrypted. An empty or absent key is never refused.
pub fn checked_key(endpoint: &Endpoint, key: Option<String>) -> Result<Option<String>, UsageError> {
    match key {
        None => Ok(None),
        Some(value) => {
            if value.is_empty() || endpoint.is_loopback() {
                return Ok(Some(value));
            }
            Err(
                UsageError {
                    message: format!(
                        "LURIA_OPENAI_API_KEY is set, and the endpoint {} is not on this machine; luria does not send a key over plain HTTP to another machine (D045)",
                        endpoint.url()
                    ),
                },
            )
        }
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
            (&["q5"], "'q5' needs a subcommand: design or interpret"),
            (
                &["q5", "plan"],
                "unknown q5 subcommand 'plan'; they are 'design' and 'interpret'",
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

    /// The word as an argument the operating system could pass.
    fn os_word(word: &str) -> std::ffi::OsString {
        PathBuf::from(word).into_os_string()
    }

    #[test]
    fn utf8_arguments_passes_through_the_words_it_was_given() {
        let expected: Vec<String> = ["q5", "design", "a.json"]
            .iter()
            .map(|word| word.to_string())
            .collect();
        let arguments: Vec<std::ffi::OsString> =
            [os_word("q5"), os_word("design"), os_word("a.json")].to_vec();
        assert_eq!(utf8_arguments(arguments), Ok(expected));
    }

    #[cfg(unix)]
    #[test]
    fn a_word_that_is_not_utf8_names_its_position() {
        use std::os::unix::ffi::OsStringExt;

        let word: std::ffi::OsString = OsStringExt::from_vec(b"r\xff.json".to_vec());
        let arguments: Vec<std::ffi::OsString> = [os_word("q5"), os_word("design"), word].to_vec();
        assert_eq!(
            utf8_arguments(arguments),
            Err(UsageError {
                message: "argument 3 is not valid UTF-8".to_string(),
            })
        );
    }

    #[test]
    fn a_double_dash_ends_the_options_of_q5_design() {
        assert_eq!(
            parse_words(&["q5", "design", "--", "-x.json"]),
            Ok(Command::Q5Design {
                request: PathBuf::from("-x.json"),
                write_edited: None,
            })
        );
        assert_eq!(
            parse_words(&["q5", "design", "--write-edited", "out", "--", "-x.json"]),
            Ok(Command::Q5Design {
                request: PathBuf::from("-x.json"),
                write_edited: Some(PathBuf::from("out")),
            })
        );
        assert_eq!(
            usage(&["q5", "design", "--", "a.json", "b.json"]),
            "'q5 design' takes one request file, got a second: 'b.json'"
        );
        assert_eq!(
            usage(&["q5", "design", "--"]),
            "'q5 design' needs a request file"
        );
        assert_eq!(
            parse_words(&["q5", "design", "--", "--write-edited"]),
            Ok(Command::Q5Design {
                request: PathBuf::from("--write-edited"),
                write_edited: None,
            })
        );
    }

    #[test]
    fn review_takes_one_request_file() {
        assert_eq!(
            parse_words(&["review", "r.json"]),
            Ok(Command::Review {
                request: PathBuf::from("r.json")
            })
        );
        // The word after -- is the request path, whatever it is, and -- is not
        // a path.
        assert_eq!(
            parse_words(&["review", "--", "-r.json"]),
            Ok(Command::Review {
                request: PathBuf::from("-r.json")
            })
        );
        let cases: [(&[&str], &str); 4] = [
            (&["review"], "'review' needs a request file"),
            (
                &["review", "a.json", "b.json"],
                "'review' takes one request file, got a second: 'b.json'",
            ),
            (
                &["review", "--write-edited", "d", "a.json"],
                "unknown option '--write-edited'",
            ),
            (&["review", ""], "an empty path is not a file"),
        ];
        for (words, message) in cases {
            assert_eq!(usage(words), message, "{:?}", words);
        }
    }

    /// One usage case: the words, and the message they must give.
    type UsageCase<'a> = (Vec<&'a str>, &'a str);

    fn interpret_words<'a>(extra: &[&'a str]) -> Vec<&'a str> {
        let mut words: Vec<&'a str> = vec![
            "q5",
            "interpret",
            "--template",
            "r.gb",
            "--text",
            "make A42G",
            "--backend",
            "openai-compatible",
            "--endpoint",
            "http://127.0.0.1:8080/v1",
            "--model",
            "qwen",
        ];
        words.extend_from_slice(extra);
        words
    }

    /// `base` without `option` and the value after it.
    fn without<'a>(base: &[&'a str], option: &str) -> Vec<&'a str> {
        let at: usize = base
            .iter()
            .position(|word| *word == option)
            .expect("an option");
        let mut words: Vec<&'a str> = base.to_vec();
        words.drain(at..at + 2);
        words
    }

    #[test]
    fn q5_interpret_reads_every_option() {
        assert_eq!(
            parse_words(&interpret_words(&[
                "--design",
                "p.json",
                "--allow-remote",
                "--write-request",
                "out.json"
            ])),
            Ok(Command::Q5Interpret {
                template: PathBuf::from("r.gb"),
                text: TextSource::Inline("make A42G".to_string()),
                design: Some(PathBuf::from("p.json")),
                endpoint: "http://127.0.0.1:8080/v1".to_string(),
                model: "qwen".to_string(),
                allow_remote: true,
                write_request: Some(PathBuf::from("out.json")),
            })
        );
        let file: Vec<&str> = vec![
            "q5",
            "interpret",
            "--model",
            "m",
            "--text-file",
            "t.txt",
            "--endpoint",
            "e",
            "--backend",
            "openai-compatible",
            "--template",
            "r.gb",
        ];
        assert_eq!(
            parse_words(&file),
            Ok(Command::Q5Interpret {
                template: PathBuf::from("r.gb"),
                text: TextSource::File(PathBuf::from("t.txt")),
                design: None,
                endpoint: "e".to_string(),
                model: "m".to_string(),
                allow_remote: false,
                write_request: None,
            })
        );
        // A value is the next word, whatever it is.
        assert!(matches!(
            parse_words(&interpret_words(&["--design", "--allow-remote"])),
            Ok(Command::Q5Interpret {
                design: Some(_),
                allow_remote: false,
                ..
            })
        ));
    }

    #[test]
    fn each_q5_interpret_usage_error_says_what_is_wrong() {
        let base: Vec<&str> = interpret_words(&[]);
        let cases: Vec<UsageCase> = vec![
            (
                without(&base, "--template"),
                "'q5 interpret' needs --template",
            ),
            (
                without(&base, "--text"),
                "'q5 interpret' needs --text or --text-file",
            ),
            (
                without(&base, "--backend"),
                "'q5 interpret' needs --backend",
            ),
            (
                without(&base, "--endpoint"),
                "'q5 interpret' needs --endpoint",
            ),
            (without(&base, "--model"), "'q5 interpret' needs --model"),
            (
                interpret_words(&["--text-file", "t.txt"]),
                "give --text or --text-file, not both",
            ),
            (
                interpret_words(&["--model", "n"]),
                "--model is given more than once",
            ),
            (
                interpret_words(&["--allow-remote", "--allow-remote"]),
                "--allow-remote is given more than once",
            ),
            (interpret_words(&["--design"]), "--design needs a value"),
            (
                interpret_words(&["--design", ""]),
                "an empty path is not a file",
            ),
            (
                interpret_words(&["--write-request", ""]),
                "an empty path is not a file",
            ),
            (
                interpret_words(&["--verbose"]),
                "unknown option '--verbose'",
            ),
            (
                interpret_words(&["stray"]),
                "'q5 interpret' takes options only, got 'stray'",
            ),
            (
                vec!["q5", "interpret", "--template", "", "--text", "t"],
                "an empty path is not a file",
            ),
            (
                vec!["q5", "interpret", "--model", ""],
                "the model name is empty",
            ),
        ];
        for case in cases {
            assert_eq!(usage(&case.0), case.1, "{:?}", case.0);
        }
        let mut other: Vec<&str> = base.clone();
        let at: usize = other
            .iter()
            .position(|word| *word == "openai-compatible")
            .expect("the backend");
        other[at] = "anthropic";
        assert_eq!(
            usage(&other),
            "unknown backend 'anthropic'; this build has only 'openai-compatible' (M6-18)"
        );
        assert_eq!(
            usage(&["q5", "interpret", "--text"]),
            "--text needs a value"
        );
    }

    #[test]
    fn an_endpoint_leaves_this_machine_only_when_allowed() {
        assert!(checked_endpoint("http://127.0.0.1:8080/v1", false).is_ok());
        assert!(checked_endpoint("http://localhost:11434/v1", false).is_ok());
        assert_eq!(
            checked_endpoint("http://gpu.lan:8080/v1/", false).map_err(|error| error.message),
            Err("the endpoint http://gpu.lan:8080/v1 is not on this machine; sending the request there needs --allow-remote, and plain HTTP to another machine is unencrypted".to_string())
        );
        assert!(checked_endpoint("http://gpu.lan:8080/v1", true).is_ok());
        assert_eq!(
            checked_endpoint("https://api.example.com/v1", true).map_err(|error| error.message),
            Err("the endpoint is not usable: https endpoints are not supported in this build; use http:// (M6-18)".to_string())
        );
    }

    /// The endpoint a command line may call, with `--allow-remote`.
    fn allowed_endpoint(url: &str) -> Endpoint {
        match checked_endpoint(url, true) {
            Err(problem) => panic!("{:?}: {}", url, problem.message),
            Ok(endpoint) => endpoint,
        }
    }

    #[test]
    fn a_key_is_sent_to_this_machine_only() {
        assert_eq!(
            checked_key(
                &allowed_endpoint("http://127.0.0.1:8080/v1"),
                Some("sk-1".to_string())
            ),
            Ok(Some("sk-1".to_string()))
        );
        assert_eq!(
            checked_key(&allowed_endpoint("http://gpu.lan:8080/v1"), None),
            Ok(None)
        );
        assert_eq!(
            checked_key(
                &allowed_endpoint("http://gpu.lan:8080/v1"),
                Some("".to_string())
            ),
            Ok(Some("".to_string()))
        );
        assert_eq!(
            checked_key(&allowed_endpoint("http://gpu.lan:8080/v1"), Some("sk-1".to_string()))
                .map_err(|problem| problem.message),
            Err(
                "LURIA_OPENAI_API_KEY is set, and the endpoint http://gpu.lan:8080/v1 is not on this machine; luria does not send a key over plain HTTP to another machine (D045)".to_string()
            )
        );
    }
}
