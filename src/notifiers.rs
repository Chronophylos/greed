use crate::config::{Config, Notifiers, NtfyConfig, SiteConfig, TelegramConfig};

pub fn notify(
    config: &Config,
    site_config: &SiteConfig,
    last_value: Option<&str>,
    new_value: &str,
) {
    let message = format!(
        "Rule for {} triggered! Value changed from {} to {}",
        site_config.name,
        last_value.unwrap_or("<nothing>"),
        new_value
    );

    for notifier in &site_config.notifiers {
        println!("Sending notification to: {:?}", notifier);

        match notifier {
            Notifiers::Telegram => {
                send_telegram_notification(&config.telegram, &message);
            }
            Notifiers::Ntfy => {
                send_ntfy_notification(&config.ntfy, message.clone());
            }
            Notifiers::Twitch | Notifiers::Email => todo!(),
        }
    }
}

fn send_telegram_notification(_config: &TelegramConfig, message: &str) {
    println!("Sending telegram notification: {}", message);

    todo!()
}

fn send_ntfy_notification(config: &NtfyConfig, message: String) {
    println!("Sending ntfy notification: {}", message);

    let client = reqwest::Client::new();
    let url = format!("{}/{}", config.server, config.topic);
    tokio::spawn(async move {
        client.post(url).body(message).send().await.unwrap();
    });
}
