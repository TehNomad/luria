//! The backend interface and the scripted backend (contract 6f).

use luria_nl::backend::Backend;
use luria_nl::backend::BackendError;
use luria_nl::backend::BackendErrorKind;
use luria_nl::backend::BackendInfo;
use luria_nl::backend::Reply;
use luria_nl::backend::ScriptedBackend;
use luria_nl::prompt::Prompt;

fn asked(text: &str) -> Prompt {
    Prompt {
        system: "s".to_string(),
        user: text.to_string(),
    }
}

#[test]
fn the_scripted_backend_replays_in_order_and_records_each_prompt() {
    let failure: BackendError = BackendError {
        kind: BackendErrorKind::Unreachable,
        message: "no route".to_string(),
    };
    let mut backend: ScriptedBackend =
        ScriptedBackend::new("m", vec![Ok("one".to_string()), Err(failure.clone())]);
    assert_eq!(
        backend.complete(&asked("a")),
        Ok(Reply {
            text: "one".to_string()
        })
    );
    assert_eq!(backend.complete(&asked("b")), Err(failure));
    assert_eq!(
        backend.complete(&asked("c")),
        Err(BackendError {
            kind: BackendErrorKind::Response,
            message: "the scripted backend has no reply left".to_string(),
        })
    );
    assert_eq!(backend.prompts(), &[asked("a"), asked("b"), asked("c")][..]);
}

#[test]
fn a_backend_describes_itself_without_a_key() {
    let backend: ScriptedBackend = ScriptedBackend::new("qwen", vec![]);
    assert_eq!(
        backend.describe(),
        BackendInfo {
            kind: "scripted".to_string(),
            endpoint: None,
            model: "qwen".to_string(),
        }
    );
}

#[test]
fn every_backend_error_code_is_pinned() {
    assert_eq!(BackendErrorKind::Unreachable.code(), "backend_unreachable");
    assert_eq!(BackendErrorKind::Status.code(), "backend_status");
    assert_eq!(BackendErrorKind::Response.code(), "backend_response");
}

// A caller holds a backend as a trait object, whichever kind it is.
#[test]
fn a_backend_is_usable_as_a_trait_object() {
    let mut scripted: ScriptedBackend = ScriptedBackend::new("m", vec![Ok("r".to_string())]);
    let backend: &mut dyn Backend = &mut scripted;
    assert_eq!(
        backend.complete(&asked("x")).map(|reply| reply.text),
        Ok("r".to_string())
    );
    assert_eq!(backend.describe().kind, "scripted");
}
