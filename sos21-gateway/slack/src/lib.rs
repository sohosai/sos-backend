use slack_hook2::{Payload, SlackError, SlackText};
use sos21_domain::model::{form::FormName, project::ProjectName};
use tokio_compat_02::FutureExt;

pub async fn send_form_answer_notification(
    hook: impl reqwest::IntoUrl,
    project_name: &ProjectName,
    form_name: &FormName,
) -> Result<(), SlackError> {
    let slack = slack_hook2::Slack::new(hook)?;

    let slack_text = SlackText::new(format!(
        "企画「{}」が申請「{}」に回答しました。",
        project_name.as_str(),
        form_name.as_str()
    ));

    slack
        .send(&Payload {
            text: Some(slack_text),
            ..Default::default()
        })
        .compat()
        .await?;
    Ok(())
}
