//! Struct that converts DCTAP to ShEx schemas
//!
//!

use dctap::{DCTap, DatatypeId, PropertyId, ShapeId, TapShape, TapStatement};
use iri_s::IriS;
use prefixmap::IriRef;
use shex_ast::{NodeConstraint, Schema, Shape, ShapeDecl, ShapeExpr, ShapeExprLabel, TripleExpr};

use crate::{Tap2ShExConfig, Tap2ShExError};
pub struct Tap2ShEx {
    config: Tap2ShExConfig,
}

impl Tap2ShEx {
    pub fn new(config: Tap2ShExConfig) -> Tap2ShEx {
        Tap2ShEx { config }
    }

    pub fn convert(&self, tap: &DCTap) -> Result<Schema, Tap2ShExError> {
        let mut schema = Schema::new().with_prefixmap(Some(self.config.prefixmap.clone()));
        for tap_shape in tap.shapes() {
            let shape_decl = tapshape_to_shape(tap_shape, &self.config)?;
            schema.add_shape_decl(&shape_decl)
        }
        Ok(schema)
    }
}

fn tapshape_to_shape(
    tap_shape: &TapShape,
    config: &Tap2ShExConfig,
) -> Result<ShapeDecl, Tap2ShExError> {
    if let Some(shape_id) = tap_shape.shape_id() {
        let id = shape_id2iri(&shape_id, config)?;
        let label = ShapeExprLabel::iri(id);
        let shape_expr = tapshape_to_shape_expr(tap_shape, config)?;
        let shape = ShapeDecl::new(label, shape_expr, false);
        Ok(shape)
    } else {
        Err(Tap2ShExError::NoShapeId {
            tap_shape: tap_shape.clone(),
        })
    }
}

fn shape_id2iri<'a>(
    shape_id: &'a ShapeId,
    config: &'a Tap2ShExConfig,
) -> Result<IriS, Tap2ShExError> {
    let iri = match &config.base_iri {
        None => {
            todo!()
        }
        Some(base_iri) => base_iri.extend(shape_id.as_local_name().as_str())?,
    };
    Ok(iri.clone())
}

fn tapshape_to_shape_expr(
    tap_shape: &TapShape,
    config: &Tap2ShExConfig,
) -> Result<ShapeExpr, Tap2ShExError> {
    let mut tes = Vec::new();
    for statement in tap_shape.statements() {
        let te = statement_to_triple_expr(statement, config)?;
        tes.push(te)
    }
    let shape = if tes.is_empty() {
        Shape::new(None, None, None)
    } else {
        let te = TripleExpr::each_of(tes);
        Shape::new(None, None, Some(te))
    };
    let se = ShapeExpr::shape(shape);
    Ok(se)
}

fn statement_to_triple_expr(
    statement: &TapStatement,
    config: &Tap2ShExConfig,
) -> Result<TripleExpr, Tap2ShExError> {
    let pred = property_id2iri(&statement.property_id(), config)?;
    let min = get_min(statement.mandatory());
    let max = get_max(statement.repeatable());
    let value_expr = match (statement.value_datatype(), statement.value_shape()) {
        (Some(datatype), None) => {
            let iri = datatype_id2iri(&datatype, config)?;
            Ok(Some(ShapeExpr::node_constraint(
                NodeConstraint::new().with_datatype(IriRef::iri(iri)),
            )))
        }
        (None, Some(shape_id)) => {
            let iri = shape_id2iri(&shape_id, config)?;
            Ok(Some(ShapeExpr::iri_ref(IriRef::iri(iri))))
        }
        (None, None) => Ok(None),
        (Some(datatype), Some(valueshape)) => Err(Tap2ShExError::MultipleValueExprInStatement {
            value_datatype: datatype.clone(),
            value_shape: valueshape.clone(),
        }),
    }?;
    Ok(TripleExpr::triple_constraint(
        None,
        None,
        IriRef::Iri(pred),
        value_expr,
        min,
        max,
    ))
}

fn get_min(mandatory: Option<bool>) -> Option<i32> {
    match mandatory {
        Some(true) => Some(1),
        Some(false) => Some(0),
        None => Some(1),
    }
}

fn get_max(repeatable: Option<bool>) -> Option<i32> {
    match repeatable {
        Some(false) => Some(1),
        Some(true) => Some(-1),
        None => None,
    }
}

fn datatype_id2iri<'a>(
    datatype_id: &'a DatatypeId,
    config: &'a Tap2ShExConfig,
) -> Result<IriS, Tap2ShExError> {
    let iri = match &config.datatype_base_iri {
        None => {
            if let Some((prefix, localname)) = datatype_id.as_prefix_local_name() {
                let iri = config
                    .prefixmap
                    .resolve_prefix_local(prefix.as_str(), localname.as_str())?;
                Ok(iri)
            } else {
                Err(Tap2ShExError::DatatypeId2IriNoPrefix {
                    datatype_id: datatype_id.clone(),
                })
            }
        }
        Some(base_iri) => {
            let iri = base_iri.extend(datatype_id.as_local_name().as_str())?;
            Ok(iri)
        }
    }?;
    Ok(iri.clone())
}

fn property_id2iri<'a>(
    property_id: &'a PropertyId,
    config: &'a Tap2ShExConfig,
) -> Result<IriS, Tap2ShExError> {
    let iri = match &config.base_iri {
        None => {
            todo!()
        }
        Some(base_iri) => base_iri.extend(property_id.as_local_name().as_str())?,
    };
    Ok(iri.clone())
}