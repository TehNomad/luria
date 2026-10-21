//! A model endpoint's base URL, read strictly (milestone 6 plan §3, §5.6).
//!
//! The command line (6h) decides what a call may do *before* it sends
//! anything: `is_loopback` says whether the endpoint is this machine, and
//! says it from the text of the URL alone. The one name that counts as this
//! machine because of its text is `localhost`; its address is settled later,
//! when the call is made, by the system's resolver (normally `/etc/hosts`),
//! so whoever holds that file decides what `localhost` reaches. Every other
//! name is remote, however a resolver would have read it. No resolver is
//! asked before the decision is made: asking one would mean asking something
//! off this machine what this machine is, and a name can be pointed at a
//! remote address by whoever registers it.
//! `https://` gets its own refusal (M6-18): this build links no TLS and
//! cannot speak it, and quietly calling the same address over `http://`
//! would send the prompt somewhere other than the place the user named.
//!
//! What is kept is what the user typed, ASCII lower cased: the host, an
//! IPv6 address still inside its brackets, and the path with every trailing
//! `/` removed so `join` cannot write a double slash. A query, a fragment,
//! a user name or a password has nowhere to go in a base URL, so each is
//! refused with the reason rather than read and ignored: the command line
//! shows the endpoint it used, and that text has to be the endpoint the
//! user gave.

use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

/// A model endpoint's base URL, read strictly (milestone 6 plan §3, §5.6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Endpoint {
    host: String,
    port: Option<u16>,
    path: String,
}

/// Why a URL is not an endpoint luria will call. One line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndpointError {
    pub message: String,
}

impl Endpoint {
    /// Read a base URL, or say why this build will not call it. The first
    /// thing wrong is the thing reported (§3.3, rules 1 to 8).
    pub fn parse(url: &str) -> Result<Endpoint, EndpointError> {
        let rest: &str = bare(url)?;
        // The authority is the text up to the first `/`; the path keeps that
        // `/`, so `join` starts where the user stopped.
        let authority: &str;
        let path: &str;
        match rest.find('/') {
            Some(at) => {
                authority = &rest[..at];
                path = &rest[at..];
            }
            None => {
                authority = rest;
                path = "";
            }
        }
        // A user name, a query and a fragment are refused before the host is
        // judged, so each reason is the user's own, not a host error that
        // would blame the wrong part of their text.
        if authority.contains("@") {
            return Err(user_refusal());
        }
        if authority.contains("?")
            || authority.contains("#")
            || path.contains("?")
            || path.contains("#")
        {
            return Err(query_refusal());
        }
        if path.chars().any(|ch| ch.is_whitespace() || ch.is_control()) {
            return Err(path_refusal());
        }
        // A path character that a URL cannot carry is refused here, before the
        // host is read: left alone, it surfaces at send time as a backend that
        // cannot be reached, which blames the wrong part of the text (review
        // A-5).
        if path_refuses_char(path) {
            return Err(path_char_refusal());
        }
        let host: String;
        let port_text: Option<String>;
        match host_and_port(authority) {
            Err(reason) => return Err(reason),
            Ok((name, given)) => {
                host = name;
                port_text = given;
            }
        }
        // A host that is not bracketed is a name or a dotted address.
        if !host.starts_with("[") && !ascii_name(&host) {
            return Err(no_host());
        }
        let port: Option<u16> = port_of(port_text)?;
        Ok(Endpoint {
            host: host.to_ascii_lowercase(),
            port,
            path: trimmed(path),
        })
    }

    /// The host as stored: ASCII lower case, an IPv6 address in brackets.
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Whether the host is this machine, decided from the text of the URL
    /// alone: `localhost` counts by its name, and its address is settled only
    /// later, when the call is made, by the system's resolver (normally
    /// `/etc/hosts`). Every other name is remote.
    pub fn is_loopback(&self) -> bool {
        local(&self.host)
    }

    /// `http://` + host + `:port` when one was given + path.
    pub fn url(&self) -> String {
        match self.port {
            None => format!("http://{}{}", self.host, self.path),
            Some(port) => format!("http://{}:{}{}", self.host, port, self.path),
        }
    }

    /// `url()` followed by `suffix`, which starts with `/`.
    pub fn join(&self, suffix: &str) -> String {
        format!("{}{}", self.url(), suffix)
    }
}

/// Whether `text` opens with `prefix`, ASCII case-insensitively.
fn opens_with(text: &str, prefix: &str) -> bool {
    match text.get(..prefix.len()) {
        Some(head) => head.eq_ignore_ascii_case(prefix),
        None => false,
    }
}

/// The URL with its `http://` prefix removed, or the reason for refusing it.
/// Only `http://` is left: this build cannot speak `https://` (M6-18), and it
/// says so instead of treating a secure address as an insecure one.
fn bare(url: &str) -> Result<&str, EndpointError> {
    if opens_with(url, "https://") {
        return Err(EndpointError {
            message: "https endpoints are not supported in this build; use http:// (M6-18)"
                .to_string(),
        });
    }
    if opens_with(url, "http://") {
        match url.get("http://".len()..) {
            Some(rest) => Ok(rest),
            None => Err(http_refusal()),
        }
    } else {
        Err(http_refusal())
    }
}

/// Why the URL is not a plain-HTTP one.
fn http_refusal() -> EndpointError {
    EndpointError {
        message: "an endpoint must start with http://".to_string(),
    }
}

/// Why the URL carries credentials that a base URL has no place for.
fn user_refusal() -> EndpointError {
    EndpointError {
        message: "an endpoint may not carry a user name or password".to_string(),
    }
}

/// Why the URL carries a query or a fragment.
fn query_refusal() -> EndpointError {
    EndpointError {
        message: "an endpoint may not carry a query or a fragment".to_string(),
    }
}

/// Why the path cannot be part of a URL.
fn path_refusal() -> EndpointError {
    EndpointError {
        message: "an endpoint path may not hold spaces or control characters".to_string(),
    }
}

/// Why the path holds a character a URL path cannot carry.
fn path_char_refusal() -> EndpointError {
    EndpointError {
        message: "an endpoint path may hold only the characters a URL path allows".to_string(),
    }
}

/// Whether the path holds a character a URL path cannot carry, or a `%` that
/// escapes nothing. The path is read one character at a time, never as bytes:
/// a character outside ASCII is seen as one character and refused as such, not
/// as the bytes of its UTF-8 encoding.
fn path_refuses_char(path: &str) -> bool {
    let chars: Vec<char> = path.chars().collect();
    let mut at: usize = 0;
    while at < chars.len() {
        if chars[at] == '%' {
            // A `%` starts an escape, which needs two ASCII hex digits after
            // it: `%`, `%4` and `%zz` name no character.
            if at + 2 >= chars.len() {
                return true;
            }
            if hex_digit(chars[at + 1]) && hex_digit(chars[at + 2]) {
                at += 3;
                continue;
            }
            return true;
        }
        if path_carriable_char(chars[at]) {
            at += 1;
            continue;
        }
        return true;
    }
    false
}

/// Whether `ch` is a character a URL path can carry: an ASCII letter, an ASCII
/// digit, or one of `- . _ ~ ! $ & ' ( ) * + , ; = : @ / %`. Everything else
/// is refused here: a space, a control character, `?`, `#`, `[`, `]`, `{`, `}`,
/// `|`, `^`, `\`, `<`, `>`, a backtick, and any character outside ASCII.
fn path_carriable_char(ch: char) -> bool {
    if ch.is_ascii_alphanumeric() {
        return true;
    }
    if "-._~!$&'()*+,;=:@/%".contains(ch) {
        return true;
    }
    false
}

/// Whether `ch` is an ASCII hex digit: `0-9`, `a-f` or `A-F`.
fn hex_digit(ch: char) -> bool {
    if "0123456789abcdefABCDEF".contains(ch) {
        return true;
    }
    false
}

/// Why the authority holds no host, or no usable one.
fn no_host() -> EndpointError {
    EndpointError {
        message: "the endpoint's host is not valid".to_string(),
    }
}

/// Why the port that was given cannot be a port.
fn no_port() -> EndpointError {
    EndpointError {
        message: "the endpoint's port is not valid".to_string(),
    }
}

/// The host's text, and the port's text when one was given, or the reason
/// there is no host. A bracketed authority carries an IPv6 address; any
/// other authority is a host, or a host and a port split at a single `:`.
fn host_and_port(authority: &str) -> Result<(String, Option<String>), EndpointError> {
    if authority.starts_with("[") {
        return bracketed(authority);
    }
    match count_colons(authority) {
        0 => Ok((authority.to_string(), None)),
        1 => match authority.split_once(':') {
            None => Err(no_host()),
            Some(found) => Ok((found.0.to_string(), Some(found.1.to_string()))),
        },
        _ => Err(no_host()),
    }
}

/// The host of a bracketed authority, brackets included, and its port's text
/// when one follows the `]`. The text inside the brackets has to be an IPv6
/// address, and nothing but a port may follow the `]`.
fn bracketed(authority: &str) -> Result<(String, Option<String>), EndpointError> {
    match authority.find("]") {
        None => Err(no_host()),
        Some(at) => {
            let inside: &str = &authority[1..at];
            match inside.parse::<Ipv6Addr>() {
                Err(_) => Err(no_host()),
                Ok(_) => {
                    let host: String = authority[..at + 1].to_string();
                    let after: &str = authority.get(at + 1..).unwrap_or_default();
                    if after.is_empty() {
                        Ok((host, None))
                    } else if after.starts_with(":") {
                        match after.get(1..) {
                            Some(port) => Ok((host, Some(port.to_string()))),
                            None => Ok((host, Some("".to_string()))),
                        }
                    } else {
                        Err(no_host())
                    }
                }
            }
        }
    }
}

/// The port as a number, or the reason for refusing what was given: 1 to 5
/// ASCII digits, from 1 to 65535. An empty port after `:` is refused, and so
/// is a number too big for a port; a value above 65535 cannot be a port.
fn port_of(text: Option<String>) -> Result<Option<u16>, EndpointError> {
    match text {
        None => Ok(None),
        Some(given) => {
            if given.is_empty() || given.len() > 5 || !ascii_digits(&given) {
                return Err(no_port());
            }
            match given.parse::<u16>() {
                Err(_) => Err(no_port()),
                Ok(number) => {
                    if number < 1 {
                        Err(no_port())
                    } else {
                        Ok(Some(number))
                    }
                }
            }
        }
    }
}

/// A non-empty run of ASCII digits.
fn ascii_digits(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|ch| ch.is_ascii_digit())
}

/// A host: non-empty, and ASCII letters, digits, `-` and `.` only.
fn ascii_name(host: &str) -> bool {
    !host.is_empty()
        && host
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '.')
}

/// The count of `:` in an authority. More than one is not a host and a port.
fn count_colons(authority: &str) -> usize {
    authority.chars().filter(|ch| *ch == ':').count()
}

/// The text inside an IPv6 host's brackets, or nothing when the host is not
/// bracketed.
fn brackets_inside(host: &str) -> Option<String> {
    if host.len() >= 2 && host.starts_with("[") && host.ends_with("]") {
        return Some(host[1..host.len() - 1].to_string());
    }
    None
}

/// Whether the host text names this machine: `localhost`, any address in
/// `127.0.0.0/8` and `::1`. Everything else is remote, `0.0.0.0`, a short
/// form such as `127.1` and any other name included. The text alone decides,
/// and the one name it takes as this machine is `localhost`: which address
/// that is comes later, from the system's resolver when the call is made
/// (normally `/etc/hosts`). Nothing is looked up while the decision is made.
fn local(host: &str) -> bool {
    if host == "localhost" {
        return true;
    }
    match host.parse::<Ipv4Addr>() {
        Ok(found) => found.is_loopback(),
        Err(_) => match brackets_inside(host) {
            None => false,
            Some(inside) => match inside.parse::<Ipv6Addr>() {
                Ok(found) => found.is_loopback(),
                Err(_) => false,
            },
        },
    }
}

/// The path with every trailing `/` removed, so `join` cannot make a double
/// slash and a path given as `/v1//` means `/v1`.
fn trimmed(path: &str) -> String {
    let chars: Vec<char> = path.chars().collect();
    let mut to: usize = chars.len();
    while to > 0 && chars[to - 1] == '/' {
        to -= 1;
    }
    chars[..to].iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(url: &str) -> Endpoint {
        Endpoint::parse(url).unwrap_or_else(|error| panic!("{:?}: {}", url, error.message))
    }

    fn refusal(url: &str) -> String {
        match Endpoint::parse(url) {
            Err(error) => error.message,
            Ok(found) => panic!("{:?} parsed as {:?}", url, found),
        }
    }

    #[test]
    fn an_endpoint_is_normalized() {
        let cases: [(&str, &str, &str); 7] = [
            (
                "http://127.0.0.1:8080/v1",
                "127.0.0.1",
                "http://127.0.0.1:8080/v1",
            ),
            (
                "http://127.0.0.1:8080/v1/",
                "127.0.0.1",
                "http://127.0.0.1:8080/v1",
            ),
            (
                "HTTP://LocalHost:11434/v1//",
                "localhost",
                "http://localhost:11434/v1",
            ),
            ("http://[::1]:8080/v1", "[::1]", "http://[::1]:8080/v1"),
            ("http://gpu-box.lan", "gpu-box.lan", "http://gpu-box.lan"),
            ("http://10.0.0.7:80", "10.0.0.7", "http://10.0.0.7:80"),
            ("http://localhost/", "localhost", "http://localhost"),
        ];
        for case in cases {
            let endpoint: Endpoint = parsed(case.0);
            assert_eq!(endpoint.host(), case.1, "{}", case.0);
            assert_eq!(endpoint.url(), case.2, "{}", case.0);
        }
        assert_eq!(
            parsed("http://127.0.0.1:8080/v1/").join("/chat/completions"),
            "http://127.0.0.1:8080/v1/chat/completions"
        );
    }

    #[test]
    fn loopback_is_decided_from_the_text_alone() {
        let local: [&str; 6] = [
            "http://localhost:8080",
            "http://LOCALHOST",
            "http://127.0.0.1:8080",
            "http://127.255.0.1",
            "http://[::1]",
            "http://[0:0:0:0:0:0:0:1]:8080",
        ];
        for url in local {
            assert!(parsed(url).is_loopback(), "{} is local", url);
        }
        let remote: [&str; 8] = [
            "http://192.168.1.5:8080",
            "http://10.0.0.1",
            "http://0.0.0.0:8080",
            "http://127.1",
            "http://example.com",
            "http://localhost.example.com",
            "http://[::2]",
            "http://my-localhost",
        ];
        for url in remote {
            assert!(!parsed(url).is_loopback(), "{} is remote", url);
        }
    }

    #[test]
    fn each_unusable_url_is_refused_with_its_reason() {
        let cases: [(&str, &str); 16] = [
            (
                "https://api.example.com/v1",
                "https endpoints are not supported in this build; use http:// (M6-18)",
            ),
            (
                "HTTPS://127.0.0.1",
                "https endpoints are not supported in this build; use http:// (M6-18)",
            ),
            ("127.0.0.1:8080/v1", "an endpoint must start with http://"),
            ("ftp://127.0.0.1", "an endpoint must start with http://"),
            ("", "an endpoint must start with http://"),
            (
                "http://user:pass@127.0.0.1/v1",
                "an endpoint may not carry a user name or password",
            ),
            (
                "http://127.0.0.1/v1?key=1",
                "an endpoint may not carry a query or a fragment",
            ),
            (
                "http://127.0.0.1/v1#x",
                "an endpoint may not carry a query or a fragment",
            ),
            (
                "http://127.0.0.1/v 1",
                "an endpoint path may not hold spaces or control characters",
            ),
            ("http://", "the endpoint's host is not valid"),
            ("http://:8080/v1", "the endpoint's host is not valid"),
            ("http://bad_host/v1", "the endpoint's host is not valid"),
            ("http://[::1/v1", "the endpoint's host is not valid"),
            ("http://a:1:2", "the endpoint's host is not valid"),
            ("http://127.0.0.1:0", "the endpoint's port is not valid"),
            ("http://127.0.0.1:65536", "the endpoint's port is not valid"),
        ];
        for case in cases {
            assert_eq!(refusal(case.0), case.1, "{:?}", case.0);
        }
        assert_eq!(
            refusal("http://127.0.0.1:"),
            "the endpoint's port is not valid"
        );
        assert_eq!(
            refusal("http://h\u{e9}te"),
            "the endpoint's host is not valid"
        );
    }

    // A path character that a URL cannot carry is refused as a usage error,
    // with that reason (review A-5): left to reach the send, it shows up as a
    // backend that cannot be reached, which is the wrong blame.
    #[test]
    fn a_path_char_a_url_cannot_carry_is_refused() {
        let reason: &str = "an endpoint path may hold only the characters a URL path allows";
        let bad: [&str; 10] = [
            "http://127.0.0.1/v1{}",
            "http://127.0.0.1/v1|x",
            "http://127.0.0.1/v1^",
            r"http://127.0.0.1/v1\x",
            "http://127.0.0.1/v\"1",
            "http://127.0.0.1/v<1>",
            "http://127.0.0.1/v\u{e9}",
            "http://127.0.0.1/v1%",
            "http://127.0.0.1/v1%4",
            "http://127.0.0.1/v1%zz",
        ];
        for url in bad {
            assert_eq!(refusal(url), reason, "{:?}", url);
        }
        // The characters a path may carry stay accepted: the new refusal
        // takes nothing that a working endpoint already had.
        let good: [&str; 4] = [
            "http://127.0.0.1/v1",
            "http://127.0.0.1/a-b.c_d~e",
            "http://127.0.0.1/v1/%41",
            "http://127.0.0.1/x!$&'()*+,;=:@",
        ];
        for url in good {
            let endpoint: Endpoint = parsed(url);
            assert_eq!(endpoint.url(), url, "{:?}", url);
        }
    }

    // Hosts that are dressed up to look like this machine stay remote, or are
    // refused outright: nothing disguised becomes loopback (review A-7).
    #[test]
    fn no_disguised_host_is_loopback() {
        let disguised: [&str; 11] = [
            "127.0.0.1.evil.com",
            "localhost.",
            "[::ffff:127.0.0.1]",
            "127.0.0.1\\@evil.com",
            "evil.com@127.0.0.1",
            "2130706433",
            "0177.0.0.1",
            "0x7f.0.0.1",
            "[fe80::1%25eth0]",
            "0.0.0.0",
            "127.1",
        ];
        for host in disguised {
            let url: String = format!("http://{:}/v1", host);
            let local: bool = match Endpoint::parse(&url) {
                Err(_) => false,
                Ok(endpoint) => endpoint.is_loopback(),
            };
            assert!(
                !local,
                "{:} is not this machine, and is read as loopback",
                host
            );
        }
    }
}
