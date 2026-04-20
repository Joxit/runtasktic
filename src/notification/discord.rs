use crate::config::Discord;
use crate::notification::replace_templates;

pub fn post_discord(discord: &Discord, message: &str) -> Result<(), String> {
  let content = json::object! {
    "username" => discord.username().as_ref().unwrap_or(&"runtasktic".to_string()).as_str(),
    "content" => replace_templates(message)
  };

  let resp = attohttpc::post(discord.url())
    .header_append("Content-Type", "application/json")
    .text(content.dump())
    .send()
    .unwrap();

  if !resp.status().is_success() {
    Err(format!(
      "Notification failed: status code {} and body: {}",
      resp.status(),
      resp.text().unwrap_or("<Empty Body>".to_string())
    ))
  } else {
    Ok(())
  }
}
