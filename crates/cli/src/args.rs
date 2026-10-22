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
        export: Option<PathBuf>,
    },
    ReviewInterpret {
        template: PathBuf,
        text: TextSource,
        design: Option<PathBuf>,
        endpoint: String,
        model: String,
        allow_remote: bool,
        request: PathBuf,
        export: Option<PathBuf>,
    },
    New {
        design: Option<PathBuf>,
        export: Option<PathBuf>,
        backend: Option<NewBackend>,
    },
}

/// The model `luria new`'s `In words` box sends its text to (7k7, M7-40): the
/// endpoint and the model name as the command line gave them, and whether an
/// endpoint off this machine was allowed, the same three the interpret form
/// reads, so `new` gates them exactly as `q5 interpret` gates them.
#[derive(Clone, Debug, PartialEq)]
pub struct NewBackend {
    pub endpoint: String,
    pub model: String,
    pub allow_remote: bool,
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
  luria review REQUEST.json [--export DIR]
  luria review --template FILE.gb (--text TEXT | --text-file FILE)
      --backend openai-compatible --endpoint URL --model NAME
      --request FILE.json [--design PROFILE.json] [--allow-remote] [--export DIR]
  luria new [--design PROFILE.json] [--export DIR]
      [--backend openai-compatible --endpoint URL --model NAME [--allow-remote]]
  luria version
  luria help

'q5 design' reads a Q5 site-directed mutagenesis request (JSON), runs the
workflow and writes one JSON result document to stdout. 'q5 interpret'
asks a language model what a request in words means and writes the
proposed request, or its questions, as one JSON document. 'review' runs a
request, or interprets a request in words and runs it once you approve, and
shows the result in the terminal. 'new' takes a template and mutations in a
form and runs them once you approve. See docs/cli.md.
";

/// The reason `word` gives, as a `UsageError`. The value type is the command
/// the caller was building, so `reject` reads in every parse function.
fn reject<T>(message: String) -> Result<T, UsageError> {
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
    if word == "new" {
        return parse_new(rest);
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

/// Read `review`'s arguments: 7a's one request path, or the interpret form,
/// which is recognised by a word exactly `--template` and read by
/// `parse_interpret_form`' rules with `--request` required, so the place the
/// approved request will be written is stated before anything happens (7f2,
/// plan §3 rule 4). `--write-request` is not an option of `review`. Before
/// that, 7a's rule holds: one request path, `--` ending the options, and any
/// other word starting with `-` an unknown option. `--export DIR` (7g1) is the
/// directory the export screen writes to, on both forms, and, as
/// `--write-edited` is in `q5 design`, must come before the `--` that ends the
/// options. The first error found is the one reported.
fn parse_review(words: &[String]) -> Result<Command, UsageError> {
    if words.iter().any(|word| word == "--template") {
        let options: InterpretOptions = parse_interpret_form(words, "review", "--request", true)?;
        return match options.request {
            Some(path) => Ok(Command::ReviewInterpret {
                template: options.template,
                text: options.text,
                design: options.design,
                endpoint: options.endpoint,
                model: options.model,
                allow_remote: options.allow_remote,
                request: path,
                export: options.export,
            }),
            None => reject("'review' needs --request".to_string()),
        };
    }

    let mut request: Option<String> = None;
    let mut export: Option<String> = None;
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

        if word == "--export" && !options_end {
            match export {
                Some(_) => return reject("--export is given more than once".to_string()),
                None => {
                    index += 1;
                    if index == last {
                        return reject("--export needs a directory".to_string());
                    }

                    let value: &str = &words[index];
                    if value.is_empty() {
                        return reject("an empty path is not a file".to_string());
                    }
                    export = Some(value.to_string());
                }
            }
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
            export: export.map(PathBuf::from),
        }),
        None => reject("'review' needs a request file".to_string()),
    }
}

/// Read `new`'s arguments: the form opens with no path typed, so `--design` is
/// the `--design` profile 7k3 fills `Overlap` and `Codon` from, and `--export`
/// is 7g1's directory the result screen writes to, taken exactly as `review`
/// takes it. `--backend openai-compatible --endpoint URL --model NAME
/// [--allow-remote]` (7k7, M7-40) is the model the `In words` box sends its
/// words to, read by the interpret form's rules for these four options: the
/// value is the next word, whatever it is, an option given more than once is a
/// usage error naming it, an option without its value is a usage error naming
/// it, an empty path is not a file, and an empty model name is refused with the
/// words. `--backend`'s only value is `openai-compatible`; the endpoint, its
/// value empty included, is gated by `checked_endpoint` in the shell, as `q5
/// interpret` gates it. The three of `--backend`, `--endpoint` and `--model` are
/// all given, in `--backend`'s order, or none of them, else a usage error names
/// the first one missing, and `--allow-remote`, the flag of `--backend`, without
/// it is a usage error naming `--allow-remote`. There is no request file and no
/// template on the command line, so any other word, a repeated option, an option
/// without its value and an empty path are all usage errors naming the word.
/// `--` ends the options, as in `q5 design`, so a word after it is not an
/// option, and the first error found is the one reported.
fn parse_new(words: &[String]) -> Result<Command, UsageError> {
    let mut design: Option<String> = None;
    let mut export: Option<String> = None;
    let mut backend: Option<String> = None;
    let mut endpoint: Option<String> = None;
    let mut model: Option<String> = None;
    let mut allow_remote: bool = false;
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

        if !options_end && word == "--allow-remote" {
            match allow_remote {
                true => return reject("--allow-remote is given more than once".to_string()),
                false => {
                    allow_remote = true;
                    index += 1;
                    continue;
                }
            }
        }

        if !options_end && matches!(word, "--backend" | "--endpoint" | "--model") {
            // The value is the next word, whatever it is, as the interpret form
            // takes it, so the option's own rules are the form's.
            let stored: &mut Option<String> = match word {
                "--backend" => &mut backend,
                "--endpoint" => &mut endpoint,
                _ => &mut model,
            };
            if stored.is_some() {
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
            *stored = Some(value);
            index += 1;
            continue;
        }

        if !options_end && word == "--design" {
            match design {
                Some(_) => return reject("--design is given more than once".to_string()),
                None => {
                    index += 1;
                    if index == last {
                        return reject("--design needs a profile file".to_string());
                    }
                    let value: &str = &words[index];
                    if value.is_empty() {
                        return reject("an empty path is not a file".to_string());
                    }
                    design = Some(value.to_string());
                }
            }
            index += 1;
            continue;
        }

        if !options_end && word == "--export" {
            match export {
                Some(_) => return reject("--export is given more than once".to_string()),
                None => {
                    index += 1;
                    if index == last {
                        return reject("--export needs a directory".to_string());
                    }
                    let value: &str = &words[index];
                    if value.is_empty() {
                        return reject("an empty path is not a file".to_string());
                    }
                    export = Some(value.to_string());
                }
            }
            index += 1;
            continue;
        }

        return reject(format!(
            "'new' takes only --design, --export, --backend, --endpoint, --model and --allow-remote, got '{}'",
            word
        ));
    }

    // `--backend`'s value is its own, so an unknown one is named before the
    // options that follow it, as the interpret form names it.
    if let Some(kind) = &backend {
        if kind != "openai-compatible" {
            return reject(format!(
                "unknown backend '{}'; this build has only 'openai-compatible' (M6-18)",
                kind
            ));
        }
    }

    // The three are all given, in the form's order, or none of them, so a
    // partial backend names the first option missing (contract §2.1).
    if backend.is_none() {
        if allow_remote {
            return reject("--allow-remote needs --backend".to_string());
        }
        if endpoint.is_some() || model.is_some() {
            return reject("'new' needs --backend".to_string());
        }
        return Ok(Command::New {
            design: design.map(PathBuf::from),
            export: export.map(PathBuf::from),
            backend: None,
        });
    }

    let endpoint: String = match endpoint {
        Some(url) => url,
        None => return reject("'new' needs --endpoint".to_string()),
    };
    let model: String = match model {
        Some(name) => name,
        None => return reject("'new' needs --model".to_string()),
    };

    Ok(Command::New {
        design: design.map(PathBuf::from),
        export: export.map(PathBuf::from),
        backend: Some(NewBackend {
            endpoint,
            model,
            allow_remote,
        }),
    })
}

/// Whether `option` names a file or directory, whose value cannot be empty.
fn path_option(option: &str) -> bool {
    matches!(
        option,
        "--template" | "--text-file" | "--design" | "--write-request" | "--request" | "--export"
    )
}

/// The value `given` holds for `option`, when it was given at all.
fn given_value(given: &[(String, String)], option: &str) -> Option<String> {
    given
        .iter()
        .find(|pair| pair.0 == option)
        .map(|pair| pair.1.clone())
}

/// Whether `word` is one of the interpret form's value options, whose value is
/// the next word: the seven it has in both spellings, plus `request_option`,
/// the option naming the request file, which is `--write-request` for
/// `q5 interpret` and `--request` for `review` (7f2).
fn value_option(word: &str, request_option: &str) -> bool {
    matches!(
        word,
        "--template"
            | "--text"
            | "--text-file"
            | "--design"
            | "--backend"
            | "--endpoint"
            | "--model"
            | "--export"
    ) || word == request_option
}

/// One interpret form read: every option it holds, with the request file as
/// `request`, which is `--write-request`' path for `q5 interpret` and
/// `--request`' for `review`.
#[derive(Clone, Debug, PartialEq)]
struct InterpretOptions {
    template: PathBuf,
    text: TextSource,
    design: Option<PathBuf>,
    endpoint: String,
    model: String,
    allow_remote: bool,
    request: Option<PathBuf>,
    export: Option<PathBuf>,
}

/// Read the interpret form's words left to right: each value option takes the
/// next word as its value, whatever it is, and `--allow-remote` takes none.
/// The first error found is the one reported. An empty path and an empty
/// model name are refused with the words; an empty `--text` is not, because
/// `interpret` reports it as `text_empty`. After the words, the checks run
/// in the order plan §5.7 and §3 name: what must be present, and which
/// backend and endpoint are known. `named` is the command the messages name,
/// because `review` says `'review'` where `q5 interpret` says `'q5
/// interpret'` (7f2); `request_option` names the request file, and
/// `required` says whether it must be given.
fn parse_interpret_form(
    words: &[String],
    named: &str,
    request_option: &str,
    required: bool,
) -> Result<InterpretOptions, UsageError> {
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

        if value_option(word, request_option) {
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
        } else if word.starts_with("-") {
            return reject(format!("unknown option '{}'", word));
        } else {
            return reject(format!("'{}' takes options only, got '{}'", named, word));
        }

        index += 1;
    }

    let template: PathBuf = match given_value(&given, "--template") {
        None => return reject(format!("'{}' needs --template", named)),
        Some(path) => PathBuf::from(path),
    };
    let text: TextSource = match (
        given_value(&given, "--text"),
        given_value(&given, "--text-file"),
    ) {
        (None, None) => return reject(format!("'{}' needs --text or --text-file", named)),
        (Some(_), Some(_)) => return reject("give --text or --text-file, not both".to_string()),
        (Some(value), None) => TextSource::Inline(value.to_string()),
        (None, Some(path)) => TextSource::File(PathBuf::from(path)),
    };
    let backend: String = match given_value(&given, "--backend") {
        None => return reject(format!("'{}' needs --backend", named)),
        Some(kind) => kind,
    };
    if backend != "openai-compatible" {
        return reject(format!(
            "unknown backend '{}'; this build has only 'openai-compatible' (M6-18)",
            backend
        ));
    }
    let endpoint: String = match given_value(&given, "--endpoint") {
        None => return reject(format!("'{}' needs --endpoint", named)),
        Some(url) => url,
    };
    let model: String = match given_value(&given, "--model") {
        None => return reject(format!("'{}' needs --model", named)),
        Some(name) => name,
    };
    let design: Option<PathBuf> = given_value(&given, "--design").map(PathBuf::from);
    let request: Option<PathBuf> = match given_value(&given, request_option) {
        Some(path) => Some(PathBuf::from(path)),
        None => {
            if required {
                return reject(format!("'{}' needs {}", named, request_option));
            }
            None
        }
    };
    let export: Option<PathBuf> = given_value(&given, "--export").map(PathBuf::from);

    Ok(InterpretOptions {
        template,
        text,
        design,
        endpoint,
        model,
        allow_remote,
        request,
        export,
    })
}

/// Read `q5 interpret`'s words as the interpret form, with `--write-request`
/// optional, and build the command. Its own messages, and the rules, are
/// `parse_interpret_form`'s.
fn parse_interpret(words: &[String]) -> Result<Command, UsageError> {
    let options: InterpretOptions =
        parse_interpret_form(words, "q5 interpret", "--write-request", false)?;
    Ok(Command::Q5Interpret {
        template: options.template,
        text: options.text,
        design: options.design,
        endpoint: options.endpoint,
        model: options.model,
        allow_remote: options.allow_remote,
        write_request: options.request,
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
                request: PathBuf::from("r.json"),
                export: None
            })
        );
        // The word after -- is the request path, whatever it is, and -- is not
        // a path.
        assert_eq!(
            parse_words(&["review", "--", "-r.json"]),
            Ok(Command::Review {
                request: PathBuf::from("-r.json"),
                export: None
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

    /// 7g1: `--export DIR` on the request form, and the rules §3.1 states for
    /// it: the word after it is the directory, whatever it is, as 5g2's
    /// `--write-edited`'s is, and `--` ends the options before it.
    #[test]
    fn export_is_an_optional_directory_of_the_request_form() {
        let with_dir = Ok(Command::Review {
            request: PathBuf::from("r.json"),
            export: Some(PathBuf::from("out")),
        });
        assert_eq!(
            parse_words(&["review", "r.json", "--export", "out"]),
            with_dir
        );
        assert_eq!(
            parse_words(&["review", "--export", "out", "r.json"]),
            with_dir
        );
        // The word after --export is the directory, whatever it is, as --write-
        // edited's is in q5 design.
        assert_eq!(
            parse_words(&["review", "r.json", "--export", "-x"]),
            Ok(Command::Review {
                request: PathBuf::from("r.json"),
                export: Some(PathBuf::from("-x")),
            })
        );
        // `--` ends the options, so `--export` after it is the request path, as
        // `--write-edited` after it is in q5 design.
        assert_eq!(
            parse_words(&["review", "--", "--export"]),
            Ok(Command::Review {
                request: PathBuf::from("--export"),
                export: None,
            })
        );
        assert_eq!(
            parse_words(&["review", "--export", "out", "--", "-r.json"]),
            Ok(Command::Review {
                request: PathBuf::from("-r.json"),
                export: Some(PathBuf::from("out")),
            })
        );
        let cases: [(&[&str], &str); 4] = [
            (&["review", "--export"], "--export needs a directory"),
            (
                &["review", "r.json", "--export", "o", "--export", "p"],
                "--export is given more than once",
            ),
            (
                &["review", "r.json", "--export", ""],
                "an empty path is not a file",
            ),
            (
                &["review", "r.json", "--export=o"],
                "unknown option '--export=o'",
            ),
        ];
        for (words, message) in cases {
            assert_eq!(usage(words), message, "{:?}", words);
        }
    }

    /// One interpret form spelled for `review`: `--request` in place of
    /// `--write-request`, so `parse_review` reads the words as 7f2's form.
    fn review_interpret_words<'a>(extra: &[&'a str]) -> Vec<&'a str> {
        let mut words: Vec<&'a str> = vec![
            "review",
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
            "--request",
            "out.json",
        ];
        words.extend_from_slice(extra);
        words
    }

    #[test]
    fn review_interpret_reads_the_interpret_form_with_every_field() {
        assert_eq!(
            parse_words(&review_interpret_words(&[
                "--design",
                "p.json",
                "--allow-remote"
            ])),
            Ok(Command::ReviewInterpret {
                template: PathBuf::from("r.gb"),
                text: TextSource::Inline("make A42G".to_string()),
                design: Some(PathBuf::from("p.json")),
                endpoint: "http://127.0.0.1:8080/v1".to_string(),
                model: "qwen".to_string(),
                allow_remote: true,
                request: PathBuf::from("out.json"),
                export: None,
            })
        );
        // The words in any order, and `--text-file` in place of `--text`.
        let file: Vec<&str> = vec![
            "review",
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
            "--request",
            "o.json",
        ];
        assert_eq!(
            parse_words(&file),
            Ok(Command::ReviewInterpret {
                template: PathBuf::from("r.gb"),
                text: TextSource::File(PathBuf::from("t.txt")),
                design: None,
                endpoint: "e".to_string(),
                model: "m".to_string(),
                allow_remote: false,
                request: PathBuf::from("o.json"),
                export: None,
            })
        );
    }

    /// 7g1: `--export DIR` on the interpret form too, read by the same
    /// `parse_interpret_form` rules as every other path option, so the words
    /// `parse_review`' request form gives are the words the interpret form
    /// gives for it.
    #[test]
    fn export_is_an_optional_directory_of_the_interpret_form() {
        assert_eq!(
            parse_words(&review_interpret_words(&["--export", "dir"])),
            Ok(Command::ReviewInterpret {
                template: PathBuf::from("r.gb"),
                text: TextSource::Inline("make A42G".to_string()),
                design: None,
                endpoint: "http://127.0.0.1:8080/v1".to_string(),
                model: "qwen".to_string(),
                allow_remote: false,
                request: PathBuf::from("out.json"),
                export: Some(PathBuf::from("dir")),
            })
        );
        let cases: Vec<UsageCase> = vec![
            (
                review_interpret_words(&["--export", "o", "--export", "p"]),
                "--export is given more than once",
            ),
            (
                review_interpret_words(&["--export", ""]),
                "an empty path is not a file",
            ),
            (
                review_interpret_words(&["--export"]),
                "--export needs a value",
            ),
        ];
        for case in cases {
            assert_eq!(usage(&case.0), case.1, "{:?}", case.0);
        }
    }

    #[test]
    fn review_interpret_needs_request_and_has_no_write_request() {
        let base: Vec<&str> = review_interpret_words(&[]);
        // `--request` with no word after it, on a base that has no `--request`,
        // so the missing value is the first error found.
        let mut value_missing: Vec<&str> = without(&base, "--request");
        value_missing.push("--request");
        // An empty `--request` path on the same base, so the empty path is the
        // first error found.
        let mut empty: Vec<&str> = without(&base, "--request");
        empty.push("--request");
        empty.push("");
        let cases: Vec<UsageCase> = vec![
            (without(&base, "--request"), "'review' needs --request"),
            (
                // The contract's own words: the shortest interpret form, with
                // no `--request` at all.
                vec![
                    "review",
                    "--template",
                    "t.gb",
                    "--text",
                    "x",
                    "--backend",
                    "openai-compatible",
                    "--endpoint",
                    "http://127.0.0.1:1/v1",
                    "--model",
                    "m",
                ],
                "'review' needs --request",
            ),
            (
                review_interpret_words(&["--write-request", "r.json"]),
                "unknown option '--write-request'",
            ),
            (
                without(&base, "--text"),
                "'review' needs --text or --text-file",
            ),
            (
                review_interpret_words(&["stray"]),
                "'review' takes options only, got 'stray'",
            ),
            (value_missing, "--request needs a value"),
            (empty, "an empty path is not a file"),
            (
                review_interpret_words(&["--request", "p.json", "--request", "q.json"]),
                "--request is given more than once",
            ),
        ];
        for case in cases {
            assert_eq!(usage(&case.0), case.1, "{:?}", case.0);
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
