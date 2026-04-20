use crate::config::Mail;
use anyhow::{Result, anyhow};
use mail_send::{Credentials, SmtpClientBuilder, mail_builder::MessageBuilder};

pub async fn notification_email(mail: &Mail, body: &str) -> Result<()> {
  let from: (String, String) = mail.from().clone().into();
  let to: Vec<(String, String)> = mail.to().clone().into();
  let message = MessageBuilder::new()
    .from(from)
    .to(to)
    .subject(mail.subject())
    .html_body(format!("<p>{}</p>", body))
    .text_body(body);

  SmtpClientBuilder::new(mail.smtp_hostname(), mail.smtp_port())
    .implicit_tls(mail.smtp_tls())
    .credentials(Credentials::Plain {
      username: mail.smtp_username(),
      secret: mail.smtp_secret(),
    })
    .connect()
    .await
    .map_err(|e| anyhow!("Connection to SMTP failed: {}", e))?
    .send(message)
    .await
    .map_err(|e| anyhow!("Failed to sending email: {}", e))?;
  Ok(())
}
