use crate::config::Mail;
use anyhow::{anyhow, Result};
use mail_send::{mail_builder::MessageBuilder, Credentials, SmtpClientBuilder};

pub async fn notification_email(mail: &Mail, body: &str) -> Result<()> {
  let message = MessageBuilder::new()
    .from(mail.from().clone())
    .to(mail.to().clone())
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
