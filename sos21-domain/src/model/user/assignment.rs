use crate::model::pending_project::PendingProjectId;
use crate::model::project::ProjectId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UserAssignment {
    ProjectOwner(ProjectId),
    ProjectSubowner(ProjectId),
    PendingProjectOwner(PendingProjectId),
}
