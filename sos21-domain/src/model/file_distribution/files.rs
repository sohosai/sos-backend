use std::collections::HashMap;

use crate::model::collection::{self, LengthBoundedMap};
use crate::model::file_sharing::FileSharingId;
use crate::model::project::{Project, ProjectId};

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct FileDistributionFiles(
    LengthBoundedMap<typenum::U1, typenum::U1024, ProjectId, FileSharingId>,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FromSharingsErrorKind {
    Duplicated(ProjectId),
    TooLong,
    Empty,
}

#[derive(Debug, Error, Clone)]
#[error("invalid file distribution file sharings")]
pub struct FromSharingsError {
    kind: FromSharingsErrorKind,
}

impl FromSharingsError {
    pub fn kind(&self) -> FromSharingsErrorKind {
        self.kind
    }

    fn from_length_error(err: collection::BoundedLengthError<typenum::U1, typenum::U1024>) -> Self {
        let kind = match err.kind() {
            collection::LengthErrorKind::TooShort => FromSharingsErrorKind::Empty,
            collection::LengthErrorKind::TooLong => FromSharingsErrorKind::TooLong,
        };

        FromSharingsError { kind }
    }
}

impl FileDistributionFiles {
    pub fn from_sharings<I>(sharings: I) -> Result<Self, FromSharingsError>
    where
        I: IntoIterator<Item = (ProjectId, FileSharingId)>,
    {
        let mut result = HashMap::new();

        for (project_id, sharing_id) in sharings {
            if result.insert(project_id, sharing_id).is_some() {
                return Err(FromSharingsError {
                    kind: FromSharingsErrorKind::Duplicated(project_id),
                });
            }
        }

        let sharings =
            LengthBoundedMap::new(result).map_err(FromSharingsError::from_length_error)?;
        Ok(FileDistributionFiles(sharings))
    }

    pub fn contains_project(&self, project: &Project) -> bool {
        self.0.contains_key(&project.id())
    }

    pub fn get_sharing_for(&self, project: &Project) -> Option<FileSharingId> {
        self.0.get(&project.id()).copied()
    }

    pub fn sharings(&self) -> impl Iterator<Item = (ProjectId, FileSharingId)> + '_ {
        self.0
            .iter()
            .map(|(project_id, sharing_id)| (*project_id, *sharing_id))
    }
}
