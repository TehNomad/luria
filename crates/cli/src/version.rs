use serde::Serialize;
use serde_json::to_string_pretty;

use crate::REQUEST_SCHEMA;
use crate::RESULT_SCHEMA;
use crate::VERSION_SCHEMA;
use luria_core::mutation::{MUTATION_ALGORITHM_NAME, MUTATION_ALGORITHM_VERSION};
use luria_core::primer::{PRIMER_DESIGN_NAME, PRIMER_DESIGN_VERSION};
use luria_core::primer_design::{NATIVE_DESIGN_ALGORITHM_NAME, NATIVE_DESIGN_ALGORITHM_VERSION};
use luria_core::q5::{Q5_WORKFLOW_NAME, Q5_WORKFLOW_VERSION};
use luria_core::tm::{TM_FUNCTION_NAME, TM_FUNCTION_VERSION};

/// One algorithm a result can name (D021).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Algorithm {
    pub role: &'static str,
    pub name: &'static str,
    pub version: &'static str,
}

/// The schema pair a request and a result carry (plan §3.4, §3.5).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Schemas {
    pub request: &'static str,
    pub result: &'static str,
}

/// `luria version`'s document. The field order is the document's key order
/// (plan §3.5): `schema`, `package`, `version`, `algorithms`, `schemas`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct VersionDocument {
    pub schema: &'static str,
    pub package: &'static str,
    pub version: &'static str,
    pub algorithms: Vec<Algorithm>,
    pub schemas: Schemas,
}

/// Every algorithm, in this order, from the `luria-core` constants: `mutation`
/// (MUTATION_ALGORITHM_NAME/VERSION), `primer` (PRIMER_DESIGN_NAME/VERSION),
/// `tm` (TM_FUNCTION_NAME/VERSION), `native_design`
/// (NATIVE_DESIGN_ALGORITHM_NAME/VERSION) and `q5_workflow`
/// (Q5_WORKFLOW_NAME/VERSION).
pub fn algorithms() -> Vec<Algorithm> {
    vec![
        Algorithm {
            role: "mutation",
            name: MUTATION_ALGORITHM_NAME,
            version: MUTATION_ALGORITHM_VERSION,
        },
        Algorithm {
            role: "primer",
            name: PRIMER_DESIGN_NAME,
            version: PRIMER_DESIGN_VERSION,
        },
        Algorithm {
            role: "tm",
            name: TM_FUNCTION_NAME,
            version: TM_FUNCTION_VERSION,
        },
        Algorithm {
            role: "native_design",
            name: NATIVE_DESIGN_ALGORITHM_NAME,
            version: NATIVE_DESIGN_ALGORITHM_VERSION,
        },
        Algorithm {
            role: "q5_workflow",
            name: Q5_WORKFLOW_NAME,
            version: Q5_WORKFLOW_VERSION,
        },
    ]
}

/// The schema pair, in the order `request`, `result`.
pub fn schemas() -> Schemas {
    Schemas {
        request: REQUEST_SCHEMA,
        result: RESULT_SCHEMA,
    }
}

/// `luria version`'s document, in the serde_json pretty form plus `\n`.
pub fn render_version() -> String {
    let document: VersionDocument = VersionDocument {
        schema: VERSION_SCHEMA,
        package: "luria-cli",
        version: env!("CARGO_PKG_VERSION"),
        algorithms: algorithms(),
        schemas: schemas(),
    };

    match to_string_pretty(&document) {
        Ok(text) => text + "\n",
        Err(problem) => panic!("the version document always serializes: {}", problem),
    }
}
