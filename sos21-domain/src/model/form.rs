use crate::model::date_time::DateTime;
use crate::model::permissions::Permissions;
use crate::model::project::Project;
use crate::model::user::{User, UserId};

use uuid::Uuid;

pub mod condition;
pub mod description;
pub mod item;
pub mod name;
pub mod period;

pub use condition::{FormCondition, FormConditionProjectSet};
pub use description::FormDescription;
pub use item::{FormItem, FormItems};
pub use name::FormName;
pub use period::FormPeriod;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FormId(Uuid);

impl FormId {
    pub fn from_uuid(uuid: Uuid) -> Self {
        FormId(uuid)
    }

    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Form {
    pub id: FormId,
    pub created_at: DateTime,
    pub author_id: UserId,
    pub name: FormName,
    pub description: FormDescription,
    pub period: FormPeriod,
    pub items: FormItems,
    pub condition: FormCondition,
}

impl Form {
    pub fn is_visible_to(&self, user: &User) -> bool {
        user.permissions().contains(Permissions::READ_ALL_FORMS)
    }

    pub fn is_visible_to_with_project(&self, user: &User, project: &Project) -> bool {
        if self.is_visible_to(user) {
            return true;
        }

        self.condition.check(project) && project.is_visible_to(user)
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{
        project::ProjectAttributes,
        project_query::{ProjectQuery, ProjectQueryConjunction},
    };
    use crate::test::model as test_model;

    #[test]
    fn test_visibility_general() {
        let user = test_model::new_general_user();
        let operator = test_model::new_operator_user();
        let form = test_model::new_form(operator.id);
        assert!(!form.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_committee() {
        let user = test_model::new_committee_user();
        let operator = test_model::new_operator_user();
        let form = test_model::new_form(operator.id);
        assert!(form.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_operator() {
        let user = test_model::new_operator_user();
        let operator = test_model::new_operator_user();
        let form = test_model::new_form(operator.id);
        assert!(form.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_general_via_project() {
        let user = test_model::new_general_user();
        let user_project = test_model::new_general_project(user.id.clone());
        let operator = test_model::new_operator_user();
        let tautology_query = ProjectQuery::from_conjunctions(vec![ProjectQueryConjunction {
            category: None,
            attributes: ProjectAttributes::from_attributes(vec![]).unwrap(),
        }])
        .unwrap();
        let form = test_model::new_form_with_query(operator.id, tautology_query);
        assert!(form.is_visible_to_with_project(&user, &user_project));
    }
}
