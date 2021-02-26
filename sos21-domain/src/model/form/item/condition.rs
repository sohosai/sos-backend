use std::collections::HashMap;

use super::{checkbox::CheckboxId, grid_radio::GridRadioColumnId, radio::RadioId, FormItemId};
use crate::model::bound::{Bounded, Unbounded};
use crate::model::collection::{self, LengthLimitedVec};
use crate::model::form_answer::item::{FormAnswerItem, FormAnswerItemBody};

use anyhow::bail;
use serde::{
    de::{self, Deserializer},
    Deserialize, Serialize,
};
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
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

    pub fn is_matched_in(
        &self,
        known_answers: &HashMap<FormItemId, FormAnswerItem>,
    ) -> Result<bool, anyhow::Error> {
        let is_matched_in_conj = |conj: &Vec<FormItemCondition>| -> Result<bool, anyhow::Error> {
            for condition in conj.iter() {
                if !condition.is_matched_in(known_answers)? {
                    return Ok(false);
                }
            }
            Ok(true)
        };

        for conj in self.conjunctions() {
            if is_matched_in_conj(conj)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn conjunctions(&self) -> impl Iterator<Item = &'_ Vec<FormItemCondition>> + '_ {
        self.0.iter().map(|v| v.as_inner())
    }

    pub fn into_conjunctions(self) -> impl Iterator<Item = Vec<FormItemCondition>> {
        self.0.into_inner().into_iter().map(|v| v.into_inner())
    }
}

impl<'de> Deserialize<'de> for FormItemConditions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        FormItemConditions::from_conjunctions(Vec::<Vec<FormItemCondition>>::deserialize(
            deserializer,
        )?)
        .map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormItemCondition {
    Checkbox {
        item_id: FormItemId,
        checkbox_id: CheckboxId,
        expected: bool,
    },
    RadioSelected {
        item_id: FormItemId,
        radio_id: RadioId,
    },
    GridRadioSelected {
        item_id: FormItemId,
        column_id: GridRadioColumnId,
    },
}

impl FormItemCondition {
    /// Checks that the condition matches in the environment.
    ///
    /// Note that the answer environment is expected to have all preceding items of the condition
    /// in the checked form item and the answer checked with the item.
    ///
    /// This, this returns `anyhow::Error` for unchecked items, which should be interpreted as
    /// an unexpected error.
    pub fn is_matched_in(
        &self,
        known_answers: &HashMap<FormItemId, FormAnswerItem>,
    ) -> Result<bool, anyhow::Error> {
        match self {
            FormItemCondition::Checkbox {
                item_id,
                checkbox_id,
                expected,
            } => {
                let answer_item = match known_answers.get(item_id) {
                    Some(item) => item,
                    None => bail!("item_id must be known on the valid form"),
                };
                let checks = match answer_item.body.as_ref() {
                    Some(FormAnswerItemBody::Checkbox(checks)) => checks,
                    None => return Ok(false),
                    _ => bail!("answer_item.body must be Checkbox on the valid form"),
                };
                Ok(checks.is_checked(*checkbox_id) == *expected)
            }
            FormItemCondition::RadioSelected { item_id, radio_id } => {
                let answer_item = match known_answers.get(item_id) {
                    Some(item) => item,
                    None => bail!("item_id must be known on the valid form"),
                };
                let button_id = match answer_item.body.as_ref() {
                    Some(FormAnswerItemBody::Radio(Some(button_id))) => button_id,
                    Some(FormAnswerItemBody::Radio(None)) | None => return Ok(false),
                    _ => bail!("answer_item.body must be Radio on the valid form"),
                };
                Ok(button_id == radio_id)
            }
            FormItemCondition::GridRadioSelected { item_id, column_id } => {
                let answer_item = match known_answers.get(item_id) {
                    Some(item) => item,
                    None => bail!("item_id must be known on the valid form"),
                };
                let rows = match answer_item.body.as_ref() {
                    Some(FormAnswerItemBody::GridRadio(rows)) => rows,
                    None => return Ok(false),
                    _ => bail!("answer_item.body must be GridRadio on the valid form"),
                };
                let is_match = rows
                    .row_answers()
                    .any(|row_answer| row_answer.value == Some(*column_id));
                Ok(is_match)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::FormItemCondition;
    use crate::model::form_answer::item::{
        FormAnswerItem, FormAnswerItemBody, FormAnswerItemChecks, FormAnswerItemGridRows,
        GridRadioRowAnswer,
    };
    use crate::test::model as test_model;

    #[test]
    fn test_checkbox() {
        let checkbox_id = test_model::new_form_checkbox_id();
        let checks1 = FormAnswerItemChecks::from_checked_ids(vec![checkbox_id]).unwrap();
        let item1 = FormAnswerItem {
            item_id: test_model::new_form_item_id(),
            body: Some(FormAnswerItemBody::Checkbox(checks1)),
        };
        let checks2 = FormAnswerItemChecks::from_checked_ids(vec![]).unwrap();
        let item2 = FormAnswerItem {
            item_id: test_model::new_form_item_id(),
            body: Some(FormAnswerItemBody::Checkbox(checks2)),
        };
        let item3 = FormAnswerItem {
            item_id: test_model::new_form_item_id(),
            body: None,
        };

        let mut known_answers = HashMap::new();
        known_answers.insert(item1.item_id, item1.clone());
        known_answers.insert(item2.item_id, item2.clone());
        known_answers.insert(item3.item_id, item3.clone());

        {
            let condition = FormItemCondition::Checkbox {
                item_id: item1.item_id,
                checkbox_id,
                expected: true,
            };
            assert!(condition.is_matched_in(&known_answers).unwrap());
        }

        {
            let condition = FormItemCondition::Checkbox {
                item_id: item2.item_id,
                checkbox_id,
                expected: true,
            };
            assert!(!condition.is_matched_in(&known_answers).unwrap());
        }

        {
            let condition = FormItemCondition::Checkbox {
                item_id: item2.item_id,
                checkbox_id,
                expected: false,
            };
            assert!(condition.is_matched_in(&known_answers).unwrap());
        }

        {
            let condition = FormItemCondition::Checkbox {
                item_id: item3.item_id,
                checkbox_id,
                expected: false,
            };
            assert!(!condition.is_matched_in(&known_answers).unwrap());
        }
    }

    #[test]
    fn test_radio() {
        let radio_id1 = test_model::new_form_radio_button_id();
        let item1 = FormAnswerItem {
            item_id: test_model::new_form_item_id(),
            body: Some(FormAnswerItemBody::Radio(Some(radio_id1))),
        };
        let radio_id2 = test_model::new_form_radio_button_id();
        let item2 = FormAnswerItem {
            item_id: test_model::new_form_item_id(),
            body: Some(FormAnswerItemBody::Radio(Some(radio_id2))),
        };
        let item3 = FormAnswerItem {
            item_id: test_model::new_form_item_id(),
            body: None,
        };

        let mut known_answers = HashMap::new();
        known_answers.insert(item1.item_id, item1.clone());
        known_answers.insert(item2.item_id, item2.clone());
        known_answers.insert(item3.item_id, item3.clone());

        {
            let condition = FormItemCondition::RadioSelected {
                item_id: item1.item_id,
                radio_id: radio_id1,
            };
            assert!(condition.is_matched_in(&known_answers).unwrap());
        }

        {
            let condition = FormItemCondition::RadioSelected {
                item_id: item2.item_id,
                radio_id: radio_id1,
            };
            assert!(!condition.is_matched_in(&known_answers).unwrap());
        }

        {
            let condition = FormItemCondition::RadioSelected {
                item_id: item3.item_id,
                radio_id: radio_id1,
            };
            assert!(!condition.is_matched_in(&known_answers).unwrap());
        }
    }

    #[test]
    fn test_grid_radio() {
        let column_id = test_model::new_form_grid_radio_column_id();
        let rows1 = FormAnswerItemGridRows::from_row_answers(vec![
            GridRadioRowAnswer {
                row_id: test_model::new_form_grid_radio_row_id(),
                value: None,
            },
            GridRadioRowAnswer {
                row_id: test_model::new_form_grid_radio_row_id(),
                value: Some(column_id),
            },
        ])
        .unwrap();
        let item1 = FormAnswerItem {
            item_id: test_model::new_form_item_id(),
            body: Some(FormAnswerItemBody::GridRadio(rows1)),
        };
        let rows2 = FormAnswerItemGridRows::from_row_answers(vec![GridRadioRowAnswer {
            row_id: test_model::new_form_grid_radio_row_id(),
            value: None,
        }])
        .unwrap();
        let item2 = FormAnswerItem {
            item_id: test_model::new_form_item_id(),
            body: Some(FormAnswerItemBody::GridRadio(rows2)),
        };

        let mut known_answers = HashMap::new();
        known_answers.insert(item1.item_id, item1.clone());
        known_answers.insert(item2.item_id, item2.clone());

        {
            let condition = FormItemCondition::GridRadioSelected {
                item_id: item1.item_id,
                column_id,
            };
            assert!(condition.is_matched_in(&known_answers).unwrap());
        }
        {
            let condition = FormItemCondition::GridRadioSelected {
                item_id: item2.item_id,
                column_id,
            };
            assert!(!condition.is_matched_in(&known_answers).unwrap());
        }
    }
}
