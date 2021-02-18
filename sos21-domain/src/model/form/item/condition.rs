use super::{checkbox::CheckboxId, grid_radio::GridRadioColumnId, radio::RadioId, FormItemId};
use crate::model::bound::{Bounded, Unbounded};
use crate::model::collection::{self, LengthLimitedVec};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormItemConditions(
    LengthLimitedVec<
        Unbounded,
        Bounded<typenum::U16>,
        LengthLimitedVec<Unbounded, Bounded<typenum::U16>, FormItemCondition>,
    >,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeErrorKind {
    TooLongConjunction(usize),
    TooLongDisjunction,
}

#[derive(Debug, Error, Clone)]
#[error("invalid form item condition query")]
pub struct SizeError {
    kind: SizeErrorKind,
}

impl SizeError {
    pub fn kind(&self) -> SizeErrorKind {
        self.kind
    }

    fn from_conj_error(e: collection::LengthError, idx: usize) -> Self {
        let kind = match e.kind() {
            collection::LengthErrorKind::TooLong => SizeErrorKind::TooLongConjunction(idx),
            // TODO: statically assert unreachability
            collection::LengthErrorKind::TooShort => unreachable!(),
        };
        SizeError { kind }
    }

    fn from_disj_error(e: collection::LengthError) -> Self {
        let kind = match e.kind() {
            collection::LengthErrorKind::TooLong => SizeErrorKind::TooLongDisjunction,
            // TODO: statically assert unreachability
            collection::LengthErrorKind::TooShort => unreachable!(),
        };
        SizeError { kind }
    }
}

impl FormItemConditions {
    pub fn from_conjunctions<I>(dnf: I) -> Result<Self, SizeError>
    where
        I: IntoIterator<Item = Vec<FormItemCondition>>,
    {
        let dnf = dnf
            .into_iter()
            .enumerate()
            .map(|(idx, conj)| {
                LengthLimitedVec::new(conj).map_err(|e| SizeError::from_conj_error(e, idx))
            })
            .collect::<Result<_, _>>()?;
        let dnf = LengthLimitedVec::new(dnf).map_err(SizeError::from_disj_error)?;
        Ok(FormItemConditions(dnf))
    }

    pub fn conjunctions(&self) -> impl Iterator<Item = &'_ Vec<FormItemCondition>> + '_ {
        self.0.iter().map(|v| v.as_inner())
    }

    pub fn into_conjunctions(self) -> impl Iterator<Item = Vec<FormItemCondition>> {
        self.0.into_inner().into_iter().map(|v| v.into_inner())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormItemCondition {
    Checkbox {
        item_id: FormItemId,
        checkbox_id: CheckboxId,
        expected: bool,
    },
    Radio {
        item_id: FormItemId,
        radio_id: RadioId,
        expected: bool,
    },
    GridRadio {
        item_id: FormItemId,
        column_id: GridRadioColumnId,
        expected: bool,
    },
}
