use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::Arc;

use crate::context::{
    Authentication, FileRepository, FormAnswerRepository, FormRepository, Login, ObjectRepository,
    ProjectRepository, UserRepository,
};
use crate::model::{
    file::{File, FileId},
    form::{Form, FormId},
    form_answer::{FormAnswer, FormAnswerId},
    object::{Object, ObjectData, ObjectId},
    project::{Project, ProjectId, ProjectIndex},
    user::{User, UserId},
};

use anyhow::Result;
use bytes::{BufMut, Bytes, BytesMut};
use futures::lock::Mutex;
use futures::{
    future::{self, TryFutureExt},
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

    pub fn build(&self) -> MockApp {
        let users = self
            .users
            .clone()
            .into_iter()
            .map(|user| (user.id.clone(), user))
            .collect();
        let projects = self
            .projects
            .clone()
            .into_iter()
            .map(|project| (project.id, project))
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
            Authentication::new(self, user.id.clone().0, user.email.clone().into_string()).unwrap(),
        )
        .await
        .unwrap()
    }
}

#[async_trait::async_trait]
impl UserRepository for MockApp {
    async fn store_user(&self, user: User) -> Result<()> {
        self.users.lock().await.insert(user.id.clone(), user);
        Ok(())
    }

    async fn get_user(&self, id: UserId) -> Result<Option<User>> {
        Ok(self.users.lock().await.get(&id).cloned())
    }

    async fn list_users(&self) -> Result<Vec<User>> {
        Ok(self.users.lock().await.values().cloned().collect())
    }
}

#[async_trait::async_trait]
impl ProjectRepository for MockApp {
    async fn store_project(&self, project: Project) -> Result<()> {
        self.projects.lock().await.insert(project.id, project);
        Ok(())
    }

    async fn get_project(&self, id: ProjectId) -> Result<Option<(Project, User)>> {
        let project = self.projects.lock().await.get(&id).cloned();
        if let Some(project) = project {
            let owner = self.get_user(project.owner_id.clone()).await?.unwrap();
            Ok(Some((project, owner)))
        } else {
            Ok(None)
        }
    }

    async fn get_project_by_index(&self, index: ProjectIndex) -> Result<Option<(Project, User)>> {
        let project = self
            .projects
            .lock()
            .await
            .values()
            .find(|project| project.index == index)
            .cloned();
        if let Some(project) = project {
            let owner = self.get_user(project.owner_id.clone()).await?.unwrap();
            Ok(Some((project, owner)))
        } else {
            Ok(None)
        }
    }

    async fn count_projects(&self) -> Result<u64> {
        let len = self.projects.lock().await.len().try_into()?;
        Ok(len)
    }

    async fn list_projects(&self) -> Result<Vec<(Project, User)>> {
        futures::stream::iter(self.projects.lock().await.values().cloned())
            .then(|project| {
                self.get_user(project.owner_id.clone())
                    .map_ok(|owner| (project, owner.unwrap()))
            })
            .try_collect()
            .await
    }

    async fn list_projects_by_owner(&self, id: UserId) -> Result<Vec<Project>> {
        Ok(self
            .projects
            .lock()
            .await
            .values()
            .filter(|project| project.owner_id == id)
            .cloned()
            .collect())
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
        let (project, _) = self.get_project(id).await?.unwrap();
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

    async fn sum_usage_by_user(&self, user_id: UserId) -> Result<u64> {
        Ok(self
            .files
            .lock()
            .await
            .values()
            .filter_map(|file| {
                if file.author_id == user_id {
                    Some(file.size)
                } else {
                    None
                }
            })
            .sum())
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

    async fn store_object(&self, object: Object) -> Result<u64> {
        let object_id = object.id;
        let mut buf = BytesMut::new();
        let mut data = object.data.into_stream();
        while let Some(chunk) = data.try_next().await? {
            buf.put(chunk);
        }
        let len = buf.len().try_into()?;
        self.objects.lock().await.insert(object_id, buf.freeze());
        Ok(len)
    }

    async fn store_object_with_limit(
        &self,
        object: Object,
        limit: u64,
    ) -> Result<std::result::Result<u64, Self::OutOfLimitSizeError>> {
        let object_id = object.id;
        let mut buf = BytesMut::new();
        let mut data = object.data.into_stream();
        while let Some(chunk) = data.try_next().await? {
            buf.put(chunk);
            if buf.len() > limit as usize {
                return Ok(Err(OutOfLimitSizeError { _priv: () }));
            }
        }
        let len = buf.len().try_into()?;
        self.objects.lock().await.insert(object_id, buf.freeze());
        Ok(Ok(len))
    }

    async fn get_object(&self, id: ObjectId) -> Result<Option<Object>> {
        Ok(self.objects.lock().await.get(&id).cloned().map(|bytes| {
            let size = bytes.len() as u64;
            Object {
                id,
                data: ObjectData::from_stream_with_size(
                    stream::once(async move { Ok(bytes) }),
                    size,
                ),
            }
        }))
    }
}
