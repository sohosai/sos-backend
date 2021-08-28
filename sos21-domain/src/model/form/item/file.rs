use crate::model::form_answer::item::FormAnswerItemFileSharings;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod types;
pub use types::FileFormItemTypes;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileFormItem {
    pub types: Option<FileFormItemTypes>,
    pub accept_multiple_files: bool,
    pub is_required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckAnswerErrorKind {
    NotAnswered,
    NotAllowedMultipleFiles,
    NotAllowedFileType,
}

#[derive(Debug, Error, Clone)]
#[error("invalid form answer file item")]
pub struct CheckAnswerError {
    kind: CheckAnswerErrorKind,
}

impl CheckAnswerError {
    pub fn kind(&self) -> CheckAnswerErrorKind {
        self.kind
    }
}

impl FileFormItem {
    pub fn check_answer(
        &self,
        answer: &FormAnswerItemFileSharings,
    ) -> Result<(), CheckAnswerError> {
        if answer.is_multiple() && !self.accept_multiple_files {
            return Err(CheckAnswerError {
                kind: CheckAnswerErrorKind::NotAllowedMultipleFiles,
            });
        }

        if answer.is_empty() && self.is_required {
            return Err(CheckAnswerError {
                kind: CheckAnswerErrorKind::NotAnswered,
            });
        }

        if let Some(types) = &self.types {
            for sharing_answer in answer.sharing_answers() {
                if !types.contains(&sharing_answer.type_) {
                    return Err(CheckAnswerError {
                        kind: CheckAnswerErrorKind::NotAllowedFileType,
                    });
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{CheckAnswerErrorKind, FileFormItem, FileFormItemTypes};
    use crate::test::model as test_model;

    #[test]
    fn test_answer_single_optional_pass() {
        use crate::model::form_answer::item::{FileSharingAnswer, FormAnswerItemFileSharings};

        let item = FileFormItem {
            types: None,
            accept_multiple_files: false,
            is_required: false,
        };

        item.check_answer(&FormAnswerItemFileSharings::from_sharing_answers(vec![]).unwrap())
            .unwrap();
        item.check_answer(
            &FormAnswerItemFileSharings::from_sharing_answers(vec![FileSharingAnswer {
                sharing_id: test_model::new_file_sharing_id(),
                type_: test_model::mock_file_type(),
            }])
            .unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn test_answer_single_optional_fail_multiple() {
        use crate::model::form_answer::item::{FileSharingAnswer, FormAnswerItemFileSharings};

        let item = FileFormItem {
            types: None,
            accept_multiple_files: false,
            is_required: false,
        };

        assert_eq!(
            item.check_answer(
                &FormAnswerItemFileSharings::from_sharing_answers(vec![
                    FileSharingAnswer {
                        sharing_id: test_model::new_file_sharing_id(),
                        type_: test_model::mock_file_type(),
                    },
                    FileSharingAnswer {
                        sharing_id: test_model::new_file_sharing_id(),
                        type_: test_model::mock_file_type(),
                    },
                ])
                .unwrap(),
            )
            .unwrap_err()
            .kind(),
            CheckAnswerErrorKind::NotAllowedMultipleFiles
        );
    }

    #[test]
    fn test_answer_multiple_optional_pass() {
        use crate::model::form_answer::item::{FileSharingAnswer, FormAnswerItemFileSharings};

        let item = FileFormItem {
            types: None,
            accept_multiple_files: true,
            is_required: false,
        };

        item.check_answer(&FormAnswerItemFileSharings::from_sharing_answers(vec![]).unwrap())
            .unwrap();
        item.check_answer(
            &FormAnswerItemFileSharings::from_sharing_answers(vec![FileSharingAnswer {
                sharing_id: test_model::new_file_sharing_id(),
                type_: test_model::mock_file_type(),
            }])
            .unwrap(),
        )
        .unwrap();
        item.check_answer(
            &FormAnswerItemFileSharings::from_sharing_answers(vec![
                FileSharingAnswer {
                    sharing_id: test_model::new_file_sharing_id(),
                    type_: test_model::mock_file_type(),
                },
                FileSharingAnswer {
                    sharing_id: test_model::new_file_sharing_id(),
                    type_: test_model::mock_file_type(),
                },
            ])
            .unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn test_answer_single_required_pass() {
        use crate::model::form_answer::item::{FileSharingAnswer, FormAnswerItemFileSharings};

        let item = FileFormItem {
            types: None,
            accept_multiple_files: false,
            is_required: true,
        };

        item.check_answer(
            &FormAnswerItemFileSharings::from_sharing_answers(vec![FileSharingAnswer {
                sharing_id: test_model::new_file_sharing_id(),
                type_: test_model::mock_file_type(),
            }])
            .unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn test_answer_single_required_fail_empty() {
        use crate::model::form_answer::item::FormAnswerItemFileSharings;

        let item = FileFormItem {
            types: None,
            accept_multiple_files: false,
            is_required: true,
        };

        assert_eq!(
            item.check_answer(&FormAnswerItemFileSharings::from_sharing_answers(vec![]).unwrap(),)
                .unwrap_err()
                .kind(),
            CheckAnswerErrorKind::NotAnswered
        );
    }

    #[test]
    fn test_answer_single_required_fail_multiple() {
        use crate::model::form_answer::item::{FileSharingAnswer, FormAnswerItemFileSharings};

        let item = FileFormItem {
            types: None,
            accept_multiple_files: false,
            is_required: true,
        };

        assert_eq!(
            item.check_answer(
                &FormAnswerItemFileSharings::from_sharing_answers(vec![
                    FileSharingAnswer {
                        sharing_id: test_model::new_file_sharing_id(),
                        type_: test_model::mock_file_type(),
                    },
                    FileSharingAnswer {
                        sharing_id: test_model::new_file_sharing_id(),
                        type_: test_model::mock_file_type(),
                    },
                ])
                .unwrap(),
            )
            .unwrap_err()
            .kind(),
            CheckAnswerErrorKind::NotAllowedMultipleFiles
        );
    }

    #[test]
    fn test_answer_multiple_required_pass() {
        use crate::model::form_answer::item::{FileSharingAnswer, FormAnswerItemFileSharings};

        let item = FileFormItem {
            types: None,
            accept_multiple_files: true,
            is_required: true,
        };

        item.check_answer(
            &FormAnswerItemFileSharings::from_sharing_answers(vec![FileSharingAnswer {
                sharing_id: test_model::new_file_sharing_id(),
                type_: test_model::mock_file_type(),
            }])
            .unwrap(),
        )
        .unwrap();
        item.check_answer(
            &FormAnswerItemFileSharings::from_sharing_answers(vec![
                FileSharingAnswer {
                    sharing_id: test_model::new_file_sharing_id(),
                    type_: test_model::mock_file_type(),
                },
                FileSharingAnswer {
                    sharing_id: test_model::new_file_sharing_id(),
                    type_: test_model::mock_file_type(),
                },
            ])
            .unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn test_answer_multiple_required_fail_empty() {
        use crate::model::form_answer::item::FormAnswerItemFileSharings;

        let item = FileFormItem {
            types: None,
            accept_multiple_files: true,
            is_required: true,
        };

        assert_eq!(
            item.check_answer(&FormAnswerItemFileSharings::from_sharing_answers(vec![]).unwrap(),)
                .unwrap_err()
                .kind(),
            CheckAnswerErrorKind::NotAnswered
        );
    }

    #[test]
    fn test_answer_types_pass() {
        use crate::model::file::FileType;
        use crate::model::form_answer::item::{FileSharingAnswer, FormAnswerItemFileSharings};

        let type1 = FileType::from_mime(mime::IMAGE_PNG);
        let type2 = FileType::from_mime(mime::IMAGE_JPEG);

        FileFormItem {
            types: Some(FileFormItemTypes::from_types(vec![type1.clone()]).unwrap()),
            accept_multiple_files: true,
            is_required: false,
        }
        .check_answer(&FormAnswerItemFileSharings::from_sharing_answers(vec![]).unwrap())
        .unwrap();

        FileFormItem {
            types: Some(FileFormItemTypes::from_types(vec![type1.clone()]).unwrap()),
            accept_multiple_files: true,
            is_required: false,
        }
        .check_answer(
            &FormAnswerItemFileSharings::from_sharing_answers(vec![
                FileSharingAnswer {
                    sharing_id: test_model::new_file_sharing_id(),
                    type_: type1.clone(),
                },
                FileSharingAnswer {
                    sharing_id: test_model::new_file_sharing_id(),
                    type_: type1.clone(),
                },
            ])
            .unwrap(),
        )
        .unwrap();

        FileFormItem {
            types: Some(FileFormItemTypes::from_types(vec![type1.clone(), type2.clone()]).unwrap()),
            accept_multiple_files: true,
            is_required: false,
        }
        .check_answer(
            &FormAnswerItemFileSharings::from_sharing_answers(vec![
                FileSharingAnswer {
                    sharing_id: test_model::new_file_sharing_id(),
                    type_: type1,
                },
                FileSharingAnswer {
                    sharing_id: test_model::new_file_sharing_id(),
                    type_: type2,
                },
            ])
            .unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn test_answer_types_fail() {
        use crate::model::file::FileType;
        use crate::model::form_answer::item::{FileSharingAnswer, FormAnswerItemFileSharings};

        let type1 = FileType::from_mime(mime::IMAGE_PNG);
        let type2 = FileType::from_mime(mime::IMAGE_JPEG);

        assert_eq!(
            FileFormItem {
                types: Some(FileFormItemTypes::from_types(vec![type2.clone()]).unwrap()),
                accept_multiple_files: true,
                is_required: false,
            }
            .check_answer(
                &FormAnswerItemFileSharings::from_sharing_answers(vec![FileSharingAnswer {
                    sharing_id: test_model::new_file_sharing_id(),
                    type_: type1.clone(),
                },])
                .unwrap(),
            )
            .unwrap_err()
            .kind(),
            CheckAnswerErrorKind::NotAllowedFileType
        );

        assert_eq!(
            FileFormItem {
                types: Some(FileFormItemTypes::from_types(vec![type2.clone()]).unwrap()),
                accept_multiple_files: true,
                is_required: false,
            }
            .check_answer(
                &FormAnswerItemFileSharings::from_sharing_answers(vec![
                    FileSharingAnswer {
                        sharing_id: test_model::new_file_sharing_id(),
                        type_: type1,
                    },
                    FileSharingAnswer {
                        sharing_id: test_model::new_file_sharing_id(),
                        type_: type2,
                    },
                ])
                .unwrap(),
            )
            .unwrap_err()
            .kind(),
            CheckAnswerErrorKind::NotAllowedFileType
        );
    }
}
