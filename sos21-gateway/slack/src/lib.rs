use slack_hook::{Slack,PayloadBuilder,SlackTextContent::Text};
use sos21_domain::model::{form::FormName, project::ProjectName};

pub fn send_form_answer_notification(
    hook: &str,
    project_name: &ProjectName,
    form_name: &FormName,
) -> Result<(), slack_hook::Error> {
    let slack = Slack::new(hook)?;
    let payload = PayloadBuilder::new()
    .text(vec![
      Text(format!(
        "企画「{}」が申請「{}」に回答しました。",
        project_name.as_str(),
        form_name.as_str()
    ).into())
    ].as_slice())
    .build()?;

    slack.send(&payload)
}
