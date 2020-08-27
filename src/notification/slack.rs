use crate::config::Slack;

pub fn post_slack(slack: &Slack, message: &str) -> Result<(), String> {
  let mut content = json::object! {
    "channel" => slack.channel().as_str(),
    "username" => slack.username().as_ref().unwrap_or(&"runtasktic".to_string()).as_str(),
    "text" => message
  };

  if let Some(emoji) = slack.emoji() {
    content
      .insert("icon_emoji", emoji.as_str())
      .map_err(|msg| format!("{}", msg))?;
  }

  let resp = attohttpc::post(slack.url())
    .text(content.dump())
    .send()
    .unwrap();

  if resp.status() != 200 {
    Err(format!(
      "Notification failed: status code {} and body: {}",
      resp.status(),
      resp.text().unwrap_or("<Empty Body>".to_string())
    ))
  } else {
    Ok(())
  }
}
