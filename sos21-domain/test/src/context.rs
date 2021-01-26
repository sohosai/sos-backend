use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use futures::lock::Mutex;
use futures::{
    future::TryFutureExt,
    stream::{StreamExt, TryStreamExt},
};
use sos21_domain_context::{Authentication, Login, ProjectRepository, UserRepository};
use sos21_domain_model::{
    project::{Project, ProjectId},
    user::{User, UserId},
};

#[derive(Default, Debug, Clone)]
pub struct MockAppBuilder {
    users: Vec<User>,
    projects: Vec<Project>,
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

        MockApp {
            users: Arc::new(Mutex::new(users)),
            projects: Arc::new(Mutex::new(projects)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MockApp {
    users: Arc<Mutex<HashMap<UserId, User>>>,
    projects: Arc<Mutex<HashMap<ProjectId, Project>>>,
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
    async fn create_user(&self, user: User) -> Result<()> {
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
    async fn create_project(&self, project: Project) -> Result<()> {
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
