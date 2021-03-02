use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::Arc;

use crate::context::{
    Authentication, FormAnswerRepository, FormRepository, Login, ProjectRepository, UserRepository,
};
use crate::model::{
    form::{Form, FormId},
    form_answer::{FormAnswer, FormAnswerId},
    project::{Project, ProjectDisplayId, ProjectId},
    user::{User, UserId},
};

use anyhow::Result;
use futures::lock::Mutex;
use futures::{
    future::TryFutureExt,
    stream::{StreamExt, TryStreamExt},
};

#[derive(Default, Debug, Clone)]
pub struct MockAppBuilder {
    users: Vec<User>,
    projects: Vec<Project>,
    forms: Vec<Form>,
    answers: Vec<FormAnswer>,
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
        }
    }
}

#[derive(Debug, Clone)]
pub struct MockApp {
    users: Arc<Mutex<HashMap<UserId, User>>>,
    projects: Arc<Mutex<HashMap<ProjectId, Project>>>,
    forms: Arc<Mutex<HashMap<FormId, Form>>>,
    answers: Arc<Mutex<HashMap<FormAnswerId, FormAnswer>>>,
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

    async fn find_project_by_display_id(
        &self,
        display_id: ProjectDisplayId,
    ) -> Result<Option<(Project, User)>> {
        let project = self
            .projects
            .lock()
            .await
            .values()
            .find(|project| project.display_id == display_id)
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
