use super::replace_templates;
use crate::config::Discord;

pub fn post_discord(discord: &Discord, message: &str) -> Result<(), String> {
  let content = json::object! {
    "username" => discord.username.clone().unwrap_or("runtasktic".to_string()),
    "content" => replace_templates(message)
  };

  let resp = attohttpc::post(&discord.url)
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
