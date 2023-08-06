use miette::{bail, miette, Context, IntoDiagnostic, Result};
use regex::Regex;
use scraper::{Html, Selector};
use thirtyfour::prelude::*;
use tokio::time;

use crate::config::{read_config, Config, RuleConfig, SiteConfig, Transformer};
use crate::notifiers::notify;

mod config;
mod notifiers;

#[derive(Debug, Clone)]
struct SiteContext {
    last_value: Option<String>,
    selector: Selector,
}

impl SiteContext {
    pub fn new(config: &SiteConfig) -> Result<Self> {
        let selector = match Selector::parse(&config.selector) {
            Ok(selector) => selector,
            Err(inner_error) => {
                let error = miette!("Failed to parse selector: {inner_error}");
                return Err(error);
            }
        };

        let context = Self {
            last_value: None,
            selector,
        };

        Ok(context)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = read_config()?;

    let tasks = config.clone().sites.into_iter().map(|site| {
        tokio::spawn({
            let config = config.clone();

            async move {
                if let Err(err) = monitor_site(config, site).await {
                    println!("{err:?}");
                }
            }
        })
    });

    futures::future::join_all(tasks).await;

    Ok(())
}

async fn monitor_site(config: Config, site_config: SiteConfig) -> Result<()> {
    let mut interval = time::interval(site_config.interval);
    let mut context = SiteContext::new(&site_config)?;

    println!("Started monitoring of: {}", site_config.url);

    loop {
        interval.tick().await;

        check_site(&config, &site_config, &mut context)
            .await
            .wrap_err_with(|| format!("Error checking site `{}`", site_config.url))?;
    }
}

async fn check_site(
    config: &Config,
    site_config: &SiteConfig,
    context: &mut SiteContext,
) -> Result<()> {
    println!("Checking site: {}", site_config.url);

    let html = get_page_html(config, site_config).await?;

    let scraped_value = scrape(&html, &context.selector).wrap_err("Failed to scrape html")?;

    let value = apply_transformers(scraped_value, &site_config.transformers)?;
    println!("Value: {}", value);

    let should_notify = check_rules(context, &site_config.rules, &value);
    if should_notify {
        notify(config, &site_config, context.last_value.as_deref(), &value).await?;
    }

    context.last_value = Some(value);

    println!("Done checking site: {}", site_config.url);

    Ok(())
}

async fn get_page_html(config: &Config, site_config: &SiteConfig) -> Result<String> {
    if site_config.use_browser {
        println!("Using browser");

        let caps: Capabilities = match config.selenium.driver {
            config::SeleniumDriver::Chrome => DesiredCapabilities::chrome().into(),
            config::SeleniumDriver::Firefox => DesiredCapabilities::firefox().into(),
        };

        println!("Starting driver");
        let driver = WebDriver::new(&config.selenium.url, caps)
            .await
            .into_diagnostic()?;

        println!("Navigating to: {}", site_config.url);
        driver.goto(&site_config.url).await.into_diagnostic()?;

        println!("Waiting for element: {}", site_config.selector);
        let html = driver.source().await.into_diagnostic()?;

        driver.quit().await.into_diagnostic()?;

        Ok(html)
    } else {
        println!("Downloading html");

        let html = download(&config.user_agent, &site_config.url)
            .await
            .wrap_err("Failed to download html")?;

        Ok(html)
    }
}

async fn download(user_agent: &str, url: &str) -> Result<String> {
    println!("Downloading: {url}");

    let client = reqwest::Client::builder()
        .user_agent(user_agent)
        .build()
        .into_diagnostic()?;

    let body = client
        .get(url)
        .send()
        .await
        .into_diagnostic()?
        .text()
        .await
        .into_diagnostic()?;

    Ok(body)
}

fn scrape(html: &str, selector: &Selector) -> Result<String> {
    println!("Searching for first occurance of: {selector:?}");

    let fragment = Html::parse_fragment(&html);
    let Some(element) = fragment.select(&selector).next() else {
        bail!("Failed to find element matching selector: {selector:?}");
    };

    let text = element.text().collect::<Vec<_>>().join("");
    Ok(text)
}

fn apply_transformers(value: String, transformers: &[Transformer]) -> Result<String> {
    println!("Applying {} transformers", transformers.len());

    let mut value = value;
    for transformer in transformers {
        value = apply_transformer(value, transformer)?;
    }

    Ok(value)
}

fn apply_transformer(value: String, transformer: &Transformer) -> Result<String> {
    println!("Applying transformer: {transformer:?} to {value}");

    match transformer {
        Transformer::RegexExtract(regex) => {
            let re = Regex::new(&regex)
                .into_diagnostic()
                .wrap_err("Error compiling regex")?;

            let Some(captures) = re.captures(&value) else {
                    bail!("Regex did not match");
                };
            let new_value = captures
                .iter()
                .skip(1) // skip full match
                .flatten() // skip non-matches
                .map(|m| m.as_str())
                .collect::<Vec<_>>()
                .join("");

            Ok(new_value)
        }
        Transformer::Replace { from, to } => Ok(value.replace(from, to)),
    }
}

fn check_rules(context: &mut SiteContext, rules: &[RuleConfig], value: &str) -> bool {
    let last_value = context.last_value.as_deref();

    println!(
        "Checking {} rules. Change from: {last_value:?} to: {value}",
        rules.len(),
    );

    let matched_rule = rules
        .iter()
        .find(|rule| check_rule(rule, last_value, value));

    if let Some(rule) = matched_rule {
        println!("Matched rule: {rule:?}");
    }

    matched_rule.is_some()
}

fn check_rule(rule: &RuleConfig, last_value: Option<&str>, new_value: &str) -> bool {
    println!("Checking rule: {rule:?}");

    match rule {
        RuleConfig::OnChange => last_value.is_some_and(|last_value| last_value != new_value),
        RuleConfig::OnChangeFrom(old) => {
            last_value.is_some_and(|last_value| last_value != new_value && last_value == old)
        }
        RuleConfig::OnChangeTo(new) => {
            last_value.is_some_and(|last_value| last_value != new_value && new_value == new)
        }
        RuleConfig::OnChangeFromTo(old, new) => {
            last_value.is_some_and(|last_value| last_value == old && new_value == new)
        }
        RuleConfig::LessThan(threshold) => {
            as_64_and(last_value, new_value, |_, new| new < *threshold)
        }
        RuleConfig::LessThanOrEqualTo(threshold) => {
            as_64_and(last_value, new_value, |_, x| x <= *threshold)
        }
        RuleConfig::EqualTo(threshold) => {
            as_64_and(last_value, new_value, |_, new| new == *threshold)
        }
        RuleConfig::MoreThan(threshold) => {
            as_64_and(last_value, new_value, |_, new| new > *threshold)
        }
        RuleConfig::MoreThanOrEqualTo(threshold) => {
            as_64_and(last_value, new_value, |_, new| new >= *threshold)
        }
        RuleConfig::OnDecrease => as_64_and(last_value, new_value, |last_value, new_value| {
            last_value
                .map(|last_value| last_value > new_value)
                .unwrap_or(false)
        }),
        RuleConfig::OnIncrease => as_64_and(last_value, new_value, |last_value, new_value| {
            last_value
                .map(|last_value| last_value < new_value)
                .unwrap_or(false)
        }),
    }
}

fn as_64_and<F>(old_value: Option<&str>, new_value: &str, f: F) -> bool
where
    F: Fn(Option<f64>, f64) -> bool,
{
    let Ok(new_value) = new_value.parse::<f64>() else {
        println!("Failed to parse new value as float: {new_value}");

        return false;
    };

    let last_value = old_value.and_then(|last_value| last_value.parse::<f64>().ok());

    f(last_value, new_value)
}
