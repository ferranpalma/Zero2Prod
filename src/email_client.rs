use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

use crate::domain::SubscriberEmail;

pub struct EmailClient {
    http_client: Client,
    base_url: reqwest::Url,
    sender: SubscriberEmail,
    authorization_token: Secret<String>,
}

impl EmailClient {
    pub fn new(
        base_url: reqwest::Url,
        sender: SubscriberEmail,
        authorization_token: Secret<String>,
        timeout: std::time::Duration,
    ) -> Self {
        Self {
            http_client: Client::builder()
                .timeout(timeout)
                .build()
                .expect("Unable to build the HTTP client."),
            base_url,
            sender,
            authorization_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url =
            reqwest::Url::join(&self.base_url, "/email").expect("Unable to create /email url.");
        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body: html_content,
            text_body: text_content,
        };
        let builder = self
            .http_client
            .post(url)
            .header(
                "X-Postmark-Server-Token",
                self.authorization_token.expose_secret(),
            )
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
        Fake, Faker,
    };
    use reqwest::Url;
    use secrecy::Secret;
    use wiremock::{
        matchers::{any, header, header_exists, method, path},
        Mock, MockServer, Request, ResponseTemplate,
    };

    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                false
            }
        }
    }

    fn set_subject() -> String {
        Sentence(1..2).fake()
    }

    fn set_content() -> String {
        Paragraph(1..10).fake()
    }

    fn set_email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).expect("Unable to generate email from SafeEmail")
    }

    fn set_email_client(base_url: Url) -> EmailClient {
        let sender = set_email();
        EmailClient::new(
            base_url,
            sender,
            Secret::new(Faker.fake()),
            std::time::Duration::from_millis(200),
        )
    }

    #[tokio::test]
    async fn test_send_email_sends_expected_request() {
        // Arrange HTTP background server on random local port
        let mock_server = MockServer::start().await;
        let url = reqwest::Url::parse(&mock_server.uri())
            .unwrap_or_else(|_| panic!("Can't parse {} as url", mock_server.uri()));
        let email_client = set_email_client(url);

        // Returns HTTP 200 to any request. Only asserts that a request has been made
        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Send the actual email to the mock server
        let server_response = email_client
            .send_email(set_email(), &set_subject(), &set_content(), &set_content())
            .await;

        // Assert
        assert_ok!(server_response);
    }

    #[tokio::test]
    async fn test_send_email_fails_if_server_retuns_500() {
        // Arrange HTTP background server on random local port
        let mock_server = MockServer::start().await;
        let url = reqwest::Url::parse(&mock_server.uri())
            .unwrap_or_else(|_| panic!("Can't parse {} as url", mock_server.uri()));
        let email_client = set_email_client(url);

        // Force 500 return value
        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let server_response = email_client
            .send_email(set_email(), &set_subject(), &set_content(), &set_content())
            .await;

        assert_err!(server_response);
    }

    #[tokio::test]
    async fn test_send_email_timeouts_if_server_takes_too_long() {
        // Arrange HTTP background server on random local port
        let mock_server = MockServer::start().await;
        let url = reqwest::Url::parse(&mock_server.uri())
            .unwrap_or_else(|_| panic!("Can't parse {} as url", mock_server.uri()));
        let email_client = set_email_client(url);

        // Force response to take too long
        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180));
        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        let server_response = email_client
            .send_email(set_email(), &set_subject(), &set_content(), &set_content())
            .await;

        assert_err!(server_response);
    }
}
