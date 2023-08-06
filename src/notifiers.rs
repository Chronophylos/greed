use miette::{IntoDiagnostic, Result};

use crate::config::{Config, Notifiers, NtfyConfig, SiteConfig};

pub async fn notify(
    config: &Config,
    site_config: &SiteConfig,
    last_value: Option<&str>,
    new_value: &str,
) -> Result<()> {
    for notifier in &site_config.notifiers {
        println!("Sending notification to: {:?}", notifier);

        match notifier {
            Notifiers::Ntfy => {
                send_ntfy_notification(&config.ntfy, site_config, last_value, new_value).await?;
            }
        }
    }

    Ok(())
}

async fn send_ntfy_notification(
    config: &NtfyConfig,
    site_config: &SiteConfig,
    last_value: Option<&str>,
    new_value: &str,
) -> Result<()> {
    println!("Notifying via ntfy");

    let client = reqwest::Client::new();
    let url = format!("{}/{}", config.server, config.topic);

    client
        .post(url)
        .header(
            "X-Title",
            format!("Rule for {} triggered", site_config.name),
        )
        .header("X-Markdown", "true")
        .header("X-Click", &site_config.url)
        .header("X-Tag", "loudspeaker")
        .body(format!(
            "A rule for {} has been triggered! The value has changed from `{}` to `{new_value}`.",
            site_config.name,
            last_value.unwrap_or("<nothing>")
        ))
        .send()
        .await
        .into_diagnostic()?;

    Ok(())
}
