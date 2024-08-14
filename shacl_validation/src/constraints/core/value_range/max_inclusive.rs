use indoc::formatdoc;
use srdf::literal::Literal;
use srdf::QuerySRDF;
use srdf::RDFNode;
use srdf::SRDFBasic;
use srdf::SRDF;

use crate::constraints::constraint_error::ConstraintError;
use crate::constraints::ConstraintResult;
use crate::constraints::DefaultConstraintComponent;
use crate::constraints::SparqlConstraintComponent;
use crate::context::Context;
use crate::executor::DefaultExecutor;
use crate::executor::QueryExecutor;
use crate::executor::SHACLExecutor;
use crate::shape::ValueNode;
use crate::validation_report::result::ValidationResult;

/// https://www.w3.org/TR/shacl/#MaxInclusiveConstraintComponent
pub(crate) struct MaxInclusive<S: SRDFBasic> {
    max_inclusive: S::Term,
}

impl<S: SRDFBasic> MaxInclusive<S> {
    pub fn new(literal: Literal) -> Self {
        MaxInclusive {
            max_inclusive: S::object_as_term(&RDFNode::literal(literal)),
        }
    }
}

impl<S: SRDF + 'static> DefaultConstraintComponent<S> for MaxInclusive<S> {
    fn evaluate_default(
        &self,
        _executor: &DefaultExecutor<S>,
        _context: &Context,
        _value_nodes: &ValueNode<S>,
    ) -> ConstraintResult<S> {
        Err(ConstraintError::NotImplemented)
    }
}

impl<S: QuerySRDF + 'static> SparqlConstraintComponent<S> for MaxInclusive<S> {
    fn evaluate_sparql(
        &self,
        executor: &QueryExecutor<S>,
        context: &Context,
        value_nodes: &ValueNode<S>,
    ) -> ConstraintResult<S> {
        let mut results = Vec::new();

        for (focus_node, value_nodes) in value_nodes {
            for value_node in value_nodes {
                let query = formatdoc! {
                    " ASK {{ FILTER ({} > {}) }} ",
                    value_node, self.max_inclusive
                };
                let ask = match executor.store().query_ask(&query) {
                    Ok(ask) => ask,
                    Err(_) => return Err(ConstraintError::Query),
                };
                if !ask {
                    results.push(ValidationResult::new(focus_node, context, Some(value_node)));
                }
            }
        }

        Ok(results)
    }
}
