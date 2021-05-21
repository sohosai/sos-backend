use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::Arc;

use crate::context::pending_project_repository::PendingProjectWithOwner;
use crate::context::project_repository::ProjectWithOwners;
use crate::context::{
    Authentication, ConfigContext, FileDistributionRepository, FileRepository,
    FileSharingRepository, FormAnswerRepository, FormRepository, Login, ObjectRepository,
    PendingProjectRepository, ProjectRepository, RegistrationFormAnswerRepository,
    RegistrationFormRepository, UserInvitationRepository, UserRepository,
};
use crate::model::{
    file::{File, FileId},
    file_distribution::{FileDistribution, FileDistributionId},
    file_sharing::{FileSharing, FileSharingId, FileSharingScope},
    form::{Form, FormId},
    form_answer::{FormAnswer, FormAnswerId},
    object::{Object, ObjectData, ObjectId},
    pending_project::{PendingProject, PendingProjectId},
    project::{Project, ProjectId, ProjectIndex},
    registration_form::{RegistrationForm, RegistrationFormId},
    registration_form_answer::{RegistrationFormAnswer, RegistrationFormAnswerId},
    user::{User, UserEmailAddress, UserFileUsage, UserId, UserRole},
    user_invitation::{UserInvitation, UserInvitationId},
};
use crate::test::model as test_model;

use anyhow::Result;
use bytes::{BufMut, Bytes, BytesMut};
use futures::lock::Mutex;
use futures::{
    future,
    stream::{self, StreamExt, TryStreamExt},
};
use thiserror::Error;

#[derive(Default, Debug, Clone)]
pub struct MockAppBuilder {
    users: Vec<User>,
    projects: Vec<Project>,
    forms: Vec<Form>,
    answers: Vec<FormAnswer>,
    files: HashMap<FileId, File>,
    objects: HashMap<ObjectId, Bytes>,
    sharings: HashMap<FileSharingId, FileSharing>,
    distributions: HashMap<FileDistributionId, FileDistribution>,
    pending_projects: HashMap<PendingProjectId, PendingProject>,
    registration_forms: HashMap<RegistrationFormId, RegistrationForm>,
    registration_form_answers: HashMap<RegistrationFormAnswerId, RegistrationFormAnswer>,
    user_invitations: HashMap<UserInvitationId, UserInvitation>,
}

impl MockAppBuilder {
    pub fn new() -> Self {
        MockAppBuilder::default()
    }

    pub fn users<I>(&mut self, users: I) -> &mut Self
    where
        I: IntoIterator<Item = User>,
    {
        self.users.extend(users);
        self
    }

    pub fn projects<I>(&mut self, projects: I) -> &mut Self
    where
        I: IntoIterator<Item = Project>,
    {
        self.projects.extend(projects);
        self
    }

    pub fn forms<I>(&mut self, forms: I) -> &mut Self
    where
        I: IntoIterator<Item = Form>,
    {
        self.forms.extend(forms);
        self
    }

    pub fn answers<I>(&mut self, answers: I) -> &mut Self
    where
        I: IntoIterator<Item = FormAnswer>,
    {
        self.answers.extend(answers);
        self
    }

    pub fn files<I>(&mut self, files: I) -> &mut Self
    where
        I: IntoIterator<Item = File>,
    {
        self.files
            .extend(files.into_iter().map(|file| (file.id, file)));
        self
    }

    /// # Panics
    ///
    /// This function panics when it failed to read data from the given object.
    pub async fn objects<I>(&mut self, objects: I) -> &mut Self
    where
        I: IntoIterator<Item = Object>,
    {
        self.objects.extend(
            future::join_all(objects.into_iter().map(|object| async move {
                let object_id = object.id;
                let mut buf = BytesMut::new();
                let mut data = object.data.into_stream();
                while let Some(chunk) = data.try_next().await.unwrap() {
                    buf.put(chunk);
                }
                (object_id, buf.freeze())
            }))
            .await,
        );
        self
    }

    pub fn sharings<I>(&mut self, sharings: I) -> &mut Self
    where
        I: IntoIterator<Item = FileSharing>,
    {
        self.sharings
            .extend(sharings.into_iter().map(|sharing| (sharing.id(), sharing)));
        self
    }

    pub fn distributions<I>(&mut self, distributions: I) -> &mut Self
    where
        I: IntoIterator<Item = FileDistribution>,
    {
        self.distributions.extend(
            distributions
                .into_iter()
                .map(|distribution| (distribution.id, distribution)),
        );
        self
    }

    pub fn pending_projects<I>(&mut self, pending_projects: I) -> &mut Self
    where
        I: IntoIterator<Item = PendingProject>,
    {
        self.pending_projects.extend(
            pending_projects
                .into_iter()
                .map(|pending_project| (pending_project.id(), pending_project)),
        );
        self
    }

    pub fn registration_forms<I>(&mut self, registration_forms: I) -> &mut Self
    where
        I: IntoIterator<Item = RegistrationForm>,
    {
        self.registration_forms.extend(
            registration_forms
                .into_iter()
                .map(|registration_form| (registration_form.id, registration_form)),
        );
        self
    }

    pub fn registration_form_answers<I>(&mut self, registration_form_answers: I) -> &mut Self
    where
        I: IntoIterator<Item = RegistrationFormAnswer>,
    {
        self.registration_form_answers
            .extend(
                registration_form_answers
                    .into_iter()
                    .map(|registration_form_answer| {
                        (registration_form_answer.id, registration_form_answer)
                    }),
            );
        self
    }

    pub fn user_invitations<I>(&mut self, user_invitations: I) -> &mut Self
    where
        I: IntoIterator<Item = UserInvitation>,
    {
        self.user_invitations.extend(
            user_invitations
                .into_iter()
                .map(|invitation| (invitation.id(), invitation)),
        );
        self
    }

    pub fn build(&self) -> MockApp {
        let users = self
            .users
            .clone()
            .into_iter()
            .map(|user| (user.id().clone(), user))
            .collect();
        let projects = self
            .projects
            .clone()
            .into_iter()
            .map(|project| (project.id(), project))
            .collect();
        let forms = self
            .forms
            .clone()
            .into_iter()
            .map(|form| (form.id, form))
            .collect();
        let answers = self
            .answers
            .clone()
            .into_iter()
            .map(|answer| (answer.id, answer))
            .collect();

        MockApp {
            users: Arc::new(Mutex::new(users)),
            projects: Arc::new(Mutex::new(projects)),
            forms: Arc::new(Mutex::new(forms)),
            answers: Arc::new(Mutex::new(answers)),
            files: Arc::new(Mutex::new(self.files.clone())),
            objects: Arc::new(Mutex::new(self.objects.clone())),
            sharings: Arc::new(Mutex::new(self.sharings.clone())),
            distributions: Arc::new(Mutex::new(self.distributions.clone())),
            pending_projects: Arc::new(Mutex::new(self.pending_projects.clone())),
            registration_forms: Arc::new(Mutex::new(self.registration_forms.clone())),
            registration_form_answers: Arc::new(Mutex::new(self.registration_form_answers.clone())),
            user_invitations: Arc::new(Mutex::new(self.user_invitations.clone())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MockApp {
    users: Arc<Mutex<HashMap<UserId, User>>>,
    projects: Arc<Mutex<HashMap<ProjectId, Project>>>,
    forms: Arc<Mutex<HashMap<FormId, Form>>>,
    answers: Arc<Mutex<HashMap<FormAnswerId, FormAnswer>>>,
    files: Arc<Mutex<HashMap<FileId, File>>>,
    objects: Arc<Mutex<HashMap<ObjectId, Bytes>>>,
    sharings: Arc<Mutex<HashMap<FileSharingId, FileSharing>>>,
    distributions: Arc<Mutex<HashMap<FileDistributionId, FileDistribution>>>,
    pending_projects: Arc<Mutex<HashMap<PendingProjectId, PendingProject>>>,
    registration_forms: Arc<Mutex<HashMap<RegistrationFormId, RegistrationForm>>>,
    registration_form_answers:
        Arc<Mutex<HashMap<RegistrationFormAnswerId, RegistrationFormAnswer>>>,
    user_invitations: Arc<Mutex<HashMap<UserInvitationId, UserInvitation>>>,
}

impl MockApp {
    /// # Panics
    ///
    /// This function panics when the given email is not valid.
    pub fn authenticate_as(self, user_id: String, email: String) -> Authentication<MockApp> {
        Authentication::new(self, user_id, email).unwrap()
    }

    /// # Panics
    ///
    /// This function panics when the login is not successful.
    pub async fn login_as(self, user: User) -> Login<MockApp> {
        Login::new(
            Authentication::new(
                self,
                user.id().clone().0,
                user.email().clone().into_string(),
            )
            .unwrap(),
        )
        .await
        .unwrap()
    }
}

#[async_trait::async_trait]
impl UserRepository for MockApp {
    async fn store_user(&self, user: User) -> Result<()> {
        self.users.lock().await.insert(user.id().clone(), user);
        Ok(())
    }

    async fn get_user(&self, id: UserId) -> Result<Option<User>> {
        if id == *test_model::KNOWN_MOCK_GENERAL_USER_ID {
            return Ok(Some(test_model::mock_user(
                test_model::KNOWN_MOCK_GENERAL_USER_ID.clone(),
                UserRole::General,
            )));
        }

        Ok(self.users.lock().await.get(&id).cloned())
    }

    async fn list_users(&self) -> Result<Vec<User>> {
        Ok(self.users.lock().await.values().cloned().collect())
    }

    async fn get_user_by_email(&self, email: &UserEmailAddress) -> Result<Option<User>> {
        Ok(self
            .users
            .lock()
            .await
            .values()
            .find(|user| user.email() == email)
            .cloned())
    }
}

#[async_trait::async_trait]
impl ProjectRepository for MockApp {
    async fn store_project(&self, project: Project) -> Result<()> {
        self.projects.lock().await.insert(project.id(), project);
        Ok(())
    }

    async fn get_project(&self, id: ProjectId) -> Result<Option<ProjectWithOwners>> {
        let project = self.projects.lock().await.get(&id).cloned();
        if let Some(project) = project {
            let owner = self.get_user(project.owner_id().clone()).await?.unwrap();
            let subowner = self.get_user(project.subowner_id().clone()).await?.unwrap();
            Ok(Some(ProjectWithOwners {
                project,
                owner,
                subowner,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_project_by_index(&self, index: ProjectIndex) -> Result<Option<ProjectWithOwners>> {
        let project = self
            .projects
            .lock()
            .await
            .values()
            .find(|project| project.index() == index)
            .cloned();
        if let Some(project) = project {
            let owner = self.get_user(project.owner_id().clone()).await?.unwrap();
            let subowner = self.get_user(project.subowner_id().clone()).await?.unwrap();
            Ok(Some(ProjectWithOwners {
                project,
                owner,
                subowner,
            }))
        } else {
            Ok(None)
        }
    }

    async fn count_projects(&self) -> Result<u64> {
        let len = self.projects.lock().await.len().try_into()?;
        Ok(len)
    }

    async fn list_projects(&self) -> Result<Vec<ProjectWithOwners>> {
        futures::stream::iter(self.projects.lock().await.values().cloned())
            .then(|project| async move {
                let owner = self.get_user(project.owner_id().clone()).await?.unwrap();
                let subowner = self.get_user(project.subowner_id().clone()).await?.unwrap();
                Ok(ProjectWithOwners {
                    project,
                    owner,
                    subowner,
                })
            })
            .try_collect()
            .await
    }
}

#[async_trait::async_trait]
impl FormRepository for MockApp {
    async fn store_form(&self, form: Form) -> Result<()> {
        self.forms.lock().await.insert(form.id, form);
        Ok(())
    }

    async fn get_form(&self, id: FormId) -> Result<Option<Form>> {
        Ok(self.forms.lock().await.get(&id).cloned())
    }

    async fn list_forms(&self) -> Result<Vec<Form>> {
        Ok(self.forms.lock().await.values().cloned().collect())
    }

    async fn list_forms_by_project(&self, id: ProjectId) -> Result<Vec<Form>> {
        let project = self.get_project(id).await?.unwrap().project;
        Ok(self
            .forms
            .lock()
            .await
            .values()
            .filter(|form| form.condition.check(&project))
            .cloned()
            .collect())
    }
}

#[async_trait::async_trait]
impl FormAnswerRepository for MockApp {
    async fn store_form_answer(&self, answer: FormAnswer) -> Result<()> {
        self.answers.lock().await.insert(answer.id, answer);
        Ok(())
    }

    async fn get_form_answer(&self, id: FormAnswerId) -> Result<Option<FormAnswer>> {
        Ok(self.answers.lock().await.get(&id).cloned())
    }

    async fn get_form_answer_by_form_and_project(
        &self,
        form_id: FormId,
        project_id: ProjectId,
    ) -> Result<Option<FormAnswer>> {
        Ok(self
            .answers
            .lock()
            .await
            .values()
            .find(|answer| answer.form_id == form_id && answer.project_id == project_id)
            .cloned())
    }

    async fn list_form_answers(&self, form_id: FormId) -> Result<Vec<FormAnswer>> {
        Ok(self
            .answers
            .lock()
            .await
            .values()
            .filter(|answer| answer.form_id == form_id)
            .cloned()
            .collect())
    }
}

#[async_trait::async_trait]
impl FileRepository for MockApp {
    async fn store_file(&self, file: File) -> Result<()> {
        self.files.lock().await.insert(file.id, file);
        Ok(())
    }

    async fn get_file(&self, id: FileId) -> Result<Option<File>> {
        Ok(self.files.lock().await.get(&id).cloned())
    }

    async fn sum_file_usage_by_user(&self, user_id: UserId) -> Result<UserFileUsage> {
        Ok(UserFileUsage::from_number_of_bytes(
            self.files
                .lock()
                .await
                .values()
                .filter_map(|file| {
                    if file.author_id == user_id {
                        Some(file.size.to_number_of_bytes())
                    } else {
                        None
                    }
                })
                .sum(),
        ))
    }

    async fn list_files_by_user(&self, user_id: UserId) -> Result<Vec<File>> {
        Ok(self
            .files
            .lock()
            .await
            .values()
            .filter(|file| file.author_id == user_id)
            .cloned()
            .collect())
    }
}

#[derive(Debug, Error, Clone)]
#[error("out of limit object size")]
pub struct OutOfLimitSizeError {
    _priv: (),
}

#[async_trait::async_trait]
impl ObjectRepository for MockApp {
    type OutOfLimitSizeError = OutOfLimitSizeError;

    async fn store_object(&self, object: Object) -> Result<()> {
        let object_id = object.id;
        let mut buf = BytesMut::new();
        let mut data = object.data.into_stream();
        while let Some(chunk) = data.try_next().await? {
            buf.put(chunk);
        }
        self.objects.lock().await.insert(object_id, buf.freeze());
        Ok(())
    }

    async fn store_object_with_limit(
        &self,
        object: Object,
        limit: u64,
    ) -> Result<std::result::Result<(), Self::OutOfLimitSizeError>> {
        let object_id = object.id;
        let mut buf = BytesMut::new();
        let mut data = object.data.into_stream();
        while let Some(chunk) = data.try_next().await? {
            buf.put(chunk);
            if buf.len() > limit as usize {
                return Ok(Err(OutOfLimitSizeError { _priv: () }));
            }
        }
        self.objects.lock().await.insert(object_id, buf.freeze());
        Ok(Ok(()))
    }

    async fn get_object(&self, id: ObjectId) -> Result<Option<Object>> {
        Ok(self
            .objects
            .lock()
            .await
            .get(&id)
            .cloned()
            .map(|bytes| Object {
                id,
                data: ObjectData::from_stream(stream::once(async move { Ok(bytes) })),
            }))
    }
}

#[async_trait::async_trait]
impl FileSharingRepository for MockApp {
    async fn store_file_sharing(&self, sharing: FileSharing) -> Result<()> {
        self.sharings.lock().await.insert(sharing.id(), sharing);
        Ok(())
    }

    async fn get_file_sharing(&self, id: FileSharingId) -> Result<Option<(FileSharing, File)>> {
        if let Some(sharing) = self.sharings.lock().await.get(&id) {
            let file = self.get_file(sharing.file_id()).await?.unwrap();
            Ok(Some((sharing.clone(), file)))
        } else {
            Ok(None)
        }
    }

    async fn list_file_sharings_by_user(
        &self,
        user_id: UserId,
    ) -> Result<Vec<(FileSharing, File)>> {
        let mut result = Vec::new();

        for sharing in self.sharings.lock().await.values() {
            let file = self.get_file(sharing.file_id()).await?.unwrap();
            if file.author_id == user_id {
                result.push((sharing.clone(), file));
            }
        }

        Ok(result)
    }

    async fn list_file_sharings_by_pending_project(
        &self,
        pending_project_id: PendingProjectId,
    ) -> Result<Vec<FileSharing>> {
        let pending_project = self
            .get_pending_project(pending_project_id)
            .await?
            .unwrap()
            .pending_project;
        Ok(self
            .sharings
            .lock()
            .await
            .values()
            .filter(|sharing| {
                matches!(
                    sharing.scope(),
                    FileSharingScope::RegistrationFormAnswer(respondent, _)
                    if respondent.is_pending_project(&pending_project)
                )
            })
            .cloned()
            .collect())
    }
}

#[async_trait::async_trait]
impl FileDistributionRepository for MockApp {
    async fn store_file_distribution(&self, distribution: FileDistribution) -> Result<()> {
        self.distributions
            .lock()
            .await
            .insert(distribution.id, distribution);
        Ok(())
    }

    async fn get_file_distribution(
        &self,
        id: FileDistributionId,
    ) -> Result<Option<FileDistribution>> {
        Ok(self.distributions.lock().await.get(&id).cloned())
    }

    async fn list_file_distributions(&self) -> Result<Vec<FileDistribution>> {
        Ok(self.distributions.lock().await.values().cloned().collect())
    }

    async fn list_file_distributions_by_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<FileDistribution>> {
        let project = self.get_project(project_id).await?.unwrap().project;
        Ok(self
            .distributions
            .lock()
            .await
            .values()
            .filter(|distribution| distribution.is_targeted_to(&project))
            .cloned()
            .collect())
    }
}

#[async_trait::async_trait]
impl PendingProjectRepository for MockApp {
    async fn store_pending_project(&self, pending_project: PendingProject) -> Result<()> {
        self.pending_projects
            .lock()
            .await
            .insert(pending_project.id(), pending_project);
        Ok(())
    }

    async fn delete_pending_project(&self, id: PendingProjectId) -> Result<()> {
        self.pending_projects.lock().await.remove(&id);
        Ok(())
    }

    async fn get_pending_project(
        &self,
        id: PendingProjectId,
    ) -> Result<Option<PendingProjectWithOwner>> {
        let result = self.pending_projects.lock().await.get(&id).cloned();
        match result {
            Some(pending_project) => {
                let owner = self
                    .get_user(pending_project.owner_id().clone())
                    .await?
                    .unwrap();
                Ok(Some(PendingProjectWithOwner {
                    pending_project,
                    owner,
                }))
            }
            None => Ok(None),
        }
    }
}

#[async_trait::async_trait]
impl RegistrationFormRepository for MockApp {
    async fn store_registration_form(&self, registration_form: RegistrationForm) -> Result<()> {
        self.registration_forms
            .lock()
            .await
            .insert(registration_form.id, registration_form);
        Ok(())
    }

    async fn get_registration_form(
        &self,
        id: RegistrationFormId,
    ) -> Result<Option<RegistrationForm>> {
        Ok(self.registration_forms.lock().await.get(&id).cloned())
    }

    async fn list_registration_forms(&self) -> Result<Vec<RegistrationForm>> {
        Ok(self
            .registration_forms
            .lock()
            .await
            .values()
            .cloned()
            .collect())
    }

    async fn list_registration_forms_by_pending_project(
        &self,
        pending_project_id: PendingProjectId,
    ) -> Result<Vec<RegistrationForm>> {
        let pending_project = self
            .get_pending_project(pending_project_id)
            .await?
            .unwrap()
            .pending_project;
        Ok(self
            .registration_forms
            .lock()
            .await
            .values()
            .filter(|registration_form| {
                registration_form
                    .query
                    .check_pending_project(&pending_project)
            })
            .cloned()
            .collect())
    }

    async fn count_registration_forms_by_pending_project(
        &self,
        pending_project_id: PendingProjectId,
    ) -> Result<u64> {
        let pending_project = self
            .get_pending_project(pending_project_id)
            .await?
            .unwrap()
            .pending_project;
        let len = self
            .registration_forms
            .lock()
            .await
            .values()
            .filter(|registration_form| {
                registration_form
                    .query
                    .check_pending_project(&pending_project)
            })
            .cloned()
            .count();
        let len = len.try_into()?;
        Ok(len)
    }

    async fn list_registration_forms_by_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<RegistrationForm>> {
        let project = self.get_project(project_id).await?.unwrap().project;
        Ok(self
            .registration_forms
            .lock()
            .await
            .values()
            .filter(|registration_form| registration_form.query.check_project(&project))
            .cloned()
            .collect())
    }
}

#[async_trait::async_trait]
impl RegistrationFormAnswerRepository for MockApp {
    async fn store_registration_form_answer(&self, answer: RegistrationFormAnswer) -> Result<()> {
        self.registration_form_answers
            .lock()
            .await
            .insert(answer.id, answer);
        Ok(())
    }

    async fn get_registration_form_answer(
        &self,
        id: RegistrationFormAnswerId,
    ) -> Result<Option<RegistrationFormAnswer>> {
        Ok(self
            .registration_form_answers
            .lock()
            .await
            .get(&id)
            .cloned())
    }

    async fn get_registration_form_answer_by_registration_form_and_project(
        &self,
        registration_form_id: RegistrationFormId,
        project_id: ProjectId,
    ) -> Result<Option<RegistrationFormAnswer>> {
        let project = self.get_project(project_id).await?.unwrap().project;
        Ok(self
            .registration_form_answers
            .lock()
            .await
            .values()
            .find(|registration_form_answer| {
                registration_form_answer.registration_form_id == registration_form_id
                    && registration_form_answer.respondent.is_project(&project)
            })
            .cloned())
    }

    async fn get_registration_form_answer_by_registration_form_and_pending_project(
        &self,
        registration_form_id: RegistrationFormId,
        pending_project_id: PendingProjectId,
    ) -> Result<Option<RegistrationFormAnswer>> {
        let pending_project = self
            .get_pending_project(pending_project_id)
            .await?
            .unwrap()
            .pending_project;
        Ok(self
            .registration_form_answers
            .lock()
            .await
            .values()
            .find(|registration_form_answer| {
                registration_form_answer.registration_form_id == registration_form_id
                    && registration_form_answer
                        .respondent
                        .is_pending_project(&pending_project)
            })
            .cloned())
    }

    async fn list_registration_form_answers(
        &self,
        registration_form_id: RegistrationFormId,
    ) -> Result<Vec<RegistrationFormAnswer>> {
        Ok(self
            .registration_form_answers
            .lock()
            .await
            .values()
            .filter(|registration_form_answer| {
                registration_form_answer.registration_form_id == registration_form_id
            })
            .cloned()
            .collect())
    }

    async fn list_registration_form_answers_by_pending_project(
        &self,
        pending_project_id: PendingProjectId,
    ) -> Result<Vec<RegistrationFormAnswer>> {
        let pending_project = self
            .get_pending_project(pending_project_id)
            .await?
            .unwrap()
            .pending_project;
        Ok(self
            .registration_form_answers
            .lock()
            .await
            .values()
            .filter(|registration_form_answer| {
                registration_form_answer
                    .respondent
                    .is_pending_project(&pending_project)
            })
            .cloned()
            .collect())
    }

    async fn count_registration_form_answers_by_pending_project(
        &self,
        pending_project_id: PendingProjectId,
    ) -> Result<u64> {
        let pending_project = self
            .get_pending_project(pending_project_id)
            .await?
            .unwrap()
            .pending_project;
        let len = self
            .registration_form_answers
            .lock()
            .await
            .values()
            .filter(|registration_form_answer| {
                registration_form_answer
                    .respondent
                    .is_pending_project(&pending_project)
            })
            .cloned()
            .count();
        let len = len.try_into()?;
        Ok(len)
    }
}

#[async_trait::async_trait]
impl UserInvitationRepository for MockApp {
    async fn store_user_invitation(&self, invitation: UserInvitation) -> Result<()> {
        self.user_invitations
            .lock()
            .await
            .insert(invitation.id(), invitation);
        Ok(())
    }

    async fn delete_user_invitation(&self, id: UserInvitationId) -> Result<()> {
        self.user_invitations.lock().await.remove(&id);
        Ok(())
    }

    async fn get_user_invitation(&self, id: UserInvitationId) -> Result<Option<UserInvitation>> {
        Ok(self.user_invitations.lock().await.get(&id).cloned())
    }

    async fn list_user_invitations(&self) -> Result<Vec<UserInvitation>> {
        Ok(self
            .user_invitations
            .lock()
            .await
            .values()
            .cloned()
            .collect())
    }

    async fn get_user_invitation_by_email(
        &self,
        email: &UserEmailAddress,
    ) -> Result<Option<UserInvitation>> {
        Ok(self
            .user_invitations
            .lock()
            .await
            .values()
            .find(|invitation| invitation.email() == email)
            .cloned())
    }
}

impl ConfigContext for MockApp {
    fn administrator_email(&self) -> &UserEmailAddress {
        &*test_model::ADMINISTRATOR_EMAIL
    }
}
