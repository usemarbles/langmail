use mail_parser::{Header, HeaderName, HeaderValue};

const NOTIFICATION_DOMAINS: &[&str] = &[
    "asana.com",
    "atlassian.net",
    "github.com",
    "gitlab.com",
    "jira.com",
    "linear.app",
    "notion.so",
    "pagerduty.com",
    "sentry.io",
    "slack.com",
    "trello.com",
];

const ESP_DOMAINS: &[&str] = &[
    "amazonses.com",
    "beehiiv.com",
    "brevo.com",
    "buttondown.email",
    "convertkit.com",
    "customer.io",
    "ghost.ghost.org",
    "klaviyo.com",
    "loops.so",
    "mailchimp.com",
    "mailgun.org",
    "mandrillapp.com",
    "postmarkapp.com",
    "sendgrid.net",
    "sparkpostmail.com",
    "substack.com",
];

const PLATFORM_PREFIXES: &[&str] = &[
    "x-asana-",
    "x-atlassian-",
    "x-github-",
    "x-gitlab-",
    "x-jira-",
    "x-linear-",
    "x-notion-",
    "x-pagerduty-",
    "x-sentry-",
    "x-slack-",
];

pub(crate) fn is_newsletter(headers: &[Header<'_>]) -> bool {
    if is_excluded(headers) {
        return false;
    }
    is_included(headers)
}

fn is_excluded(headers: &[Header<'_>]) -> bool {
    for header in headers {
        match &header.name {
            HeaderName::InReplyTo => return true,
            HeaderName::ListPost => return true,
            HeaderName::ReturnPath => {
                if let Some(domain) = return_path_domain(&header.value) {
                    if NOTIFICATION_DOMAINS.binary_search(&domain.as_str()).is_ok() {
                        return true;
                    }
                }
            }
            HeaderName::Other(name) => {
                if name.eq_ignore_ascii_case("auto-submitted") {
                    if let HeaderValue::Text(val) = &header.value {
                        if val.eq_ignore_ascii_case("auto-notified") {
                            return true;
                        }
                    }
                } else {
                    let lower = name.to_ascii_lowercase();
                    if PLATFORM_PREFIXES.iter().any(|p| lower.starts_with(p)) {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }
    false
}

fn is_included(headers: &[Header<'_>]) -> bool {
    for header in headers {
        match &header.name {
            HeaderName::ListUnsubscribe | HeaderName::ListId => return true,
            HeaderName::DkimSignature => {
                if let HeaderValue::Text(val) = &header.value {
                    if let Some(domain) = dkim_domain(val) {
                        if ESP_DOMAINS.binary_search(&domain.as_str()).is_ok() {
                            return true;
                        }
                    }
                }
            }
            HeaderName::ReturnPath => {
                if let Some(domain) = return_path_domain(&header.value) {
                    if ESP_DOMAINS.binary_search(&domain.as_str()).is_ok() {
                        return true;
                    }
                }
            }
            HeaderName::Other(name) => {
                if name.eq_ignore_ascii_case("list-unsubscribe-post")
                    || name.eq_ignore_ascii_case("x-feedback-id")
                {
                    return true;
                }
                if name.eq_ignore_ascii_case("precedence") {
                    if let HeaderValue::Text(val) = &header.value {
                        if val.eq_ignore_ascii_case("bulk") {
                            return true;
                        }
                    }
                } else {
                    let lower = name.to_ascii_lowercase();
                    if lower.starts_with("x-mailgun-")
                        || lower.starts_with("x-ses-")
                        || lower.starts_with("x-campaign-")
                        || lower.starts_with("x-batch-")
                    {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }
    false
}

fn return_path_domain(value: &HeaderValue<'_>) -> Option<String> {
    // parse_id() strips angle brackets → returns Text("email@domain") for Return-Path
    if let HeaderValue::Text(email) = value {
        return email.rsplit_once('@').map(|(_, d)| d.to_lowercase());
    }
    None
}

fn dkim_domain(raw_sig: &str) -> Option<String> {
    // Raw DKIM-Signature: "v=1; a=rsa-sha256; d=mailgun.org; s=mg; ..."
    for segment in raw_sig.split(';') {
        if let Some(domain) = segment.trim().strip_prefix("d=") {
            return Some(domain.trim().to_lowercase());
        }
    }
    None
}
