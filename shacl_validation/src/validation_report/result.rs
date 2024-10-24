use super::validation_report_error::ResultError;
use crate::helpers::srdf::get_object_iri_for;
use iri_s::IriS;
use shacl_ast::*;
use srdf::SRDF;
use std::fmt::Debug;

pub struct ValidationResult {
    focus_node: IriS,           // required
    path: Option<IriS>,         // optional
    value: Option<IriS>,        // optional
    source: Option<IriS>,       // optional
    constraint_component: IriS, // required
    details: Option<Vec<IriS>>, // optional
    message: Option<IriS>,      // optional
    severity: IriS,             // required
}

#[allow(clippy::too_many_arguments)]
impl ValidationResult {
    pub fn new(
        focus_node: IriS,
        path: Option<IriS>,
        value: Option<IriS>,
        source: Option<IriS>,
        constraint_component: IriS,
        details: Option<Vec<IriS>>,
        message: Option<IriS>,
        severity: IriS,
    ) -> Self {
        Self {
            focus_node,
            path,
            value,
            source,
            constraint_component,
            details,
            message,
            severity,
        }
    }
}

impl Debug for ValidationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValidationResult")
            .field("focus_node", &self.focus_node)
            .field("path", &self.path)
            .field("value", &self.value)
            .field("source", &self.source)
            .field("constraint_component", &self.constraint_component)
            .field("details", &self.details)
            .field("message", &self.message)
            .field("severity", &self.severity)
            .finish()
    }
}

impl ValidationResult {
    pub(crate) fn parse<S: SRDF>(
        store: &S,
        validation_result: &S::Term,
    ) -> Result<Self, ResultError> {
        // 1. First, we must start processing the required fields. In case some
        //    don't appear, an error message must be raised
        let focus_node =
            match get_object_iri_for(store, validation_result, &S::iri_s2iri(&SH_FOCUS_NODE))? {
                Some(focus_node) => focus_node,
                None => return Err(ResultError::MissingRequiredField("FocusNode".to_owned())),
            };
        let severity =
            match get_object_iri_for(store, validation_result, &S::iri_s2iri(&SH_RESULT_SEVERITY))?
            {
                Some(severity) => severity,
                None => return Err(ResultError::MissingRequiredField("Severity".to_owned())),
            };
        let constraint_component = match get_object_iri_for(
            store,
            validation_result,
            &S::iri_s2iri(&SH_SOURCE_CONSTRAINT_COMPONENT),
        )? {
            Some(constraint_component) => constraint_component,
            None => {
                return Err(ResultError::MissingRequiredField(
                    "SourceConstraintComponent".to_owned(),
                ))
            }
        };

        // 2. Second, we must process the optional fields
        let path = get_object_iri_for(store, validation_result, &S::iri_s2iri(&SH_RESULT_PATH))?;
        let source = get_object_iri_for(store, validation_result, &S::iri_s2iri(&SH_SOURCE_SHAPE))?;
        let value = get_object_iri_for(store, validation_result, &S::iri_s2iri(&SH_VALUE))?;

        // 3. Lastly we build the ValidationResult
        Ok(ValidationResult {
            focus_node,
            path,
            value,
            source,
            constraint_component,
            details: None,
            message: None,
            severity,
        })
    }
}
