use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;

use super::types::{
    DeleteWebhookResponse, GetMeResponse, GetUpdatesResponse, GetWebhookInfoResponse,
    SendMessageResponse, SetWebhookResponse,
};

/// HTTP client for interacting with the Telegram Bot API.
///
/// This client provides methods to call various Telegram Bot API endpoints,
/// including fetching updates, sending messages, and validating bot credentials.
///
/// # Examples
///
/// ```no_run
/// use telegram_bot_debugger::telegram::TelegramClient;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let client = TelegramClient::new("YOUR_BOT_TOKEN".to_string());
///     
///     // Validate the bot token
///     let me = client.get_me().await?;
///     println!("Bot info: {:?}", me);
///     
///     // Fetch updates
///     let updates = client.get_updates(None, Some(30)).await?;
///     println!("Received {} updates", updates.result.len());
///     
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct TelegramClient {
    token: String,
    client: Client,
    base_url: String,
}

impl TelegramClient {
    /// Creates a new Telegram client with the given bot token.
    ///
    /// # Arguments
    ///
    /// * `token` - The bot token obtained from @BotFather
    ///
    /// # Examples
    ///
    /// ```
    /// use telegram_bot_debugger::telegram::TelegramClient;
    ///
    /// let client = TelegramClient::new("123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11".to_string());
    /// ```
    pub fn new(token: String) -> Self {
        let base_url = format!("https://api.telegram.org/bot{token}");
        Self {
            token,
            client: Client::new(),
            base_url,
        }
    }

    /// Validates the bot token by calling the getMe API method.
    ///
    /// Returns basic information about the bot.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The network request fails
    /// - The bot token is invalid
    /// - The response cannot be parsed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use telegram_bot_debugger::telegram::TelegramClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let client = TelegramClient::new("YOUR_TOKEN".to_string());
    /// let response = client.get_me().await?;
    /// if response.ok {
    ///     println!("Bot: {:?}", response.result);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_me(&self) -> Result<GetMeResponse> {
        let url = format!("{}/getMe", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send getMe request")?;

        let result = response
            .json::<GetMeResponse>()
            .await
            .context("Failed to parse getMe response")?;

        Ok(result)
    }

    /// Fetches incoming updates using long polling.
    ///
    /// Use this method to receive incoming updates. An array of Update objects is returned.
    ///
    /// # Arguments
    ///
    /// * `offset` - Identifier of the first update to be returned. Pass `update_id + 1` to
    ///   confirm receipt of previous updates.
    /// * `timeout` - Timeout in seconds for long polling (0-50). Defaults to 0 (short polling).
    ///
    /// # Errors
    ///
    /// Returns an error if the network request fails or the response cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use telegram_bot_debugger::telegram::TelegramClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let client = TelegramClient::new("YOUR_TOKEN".to_string());
    ///
    /// // Fetch updates with 30 second timeout
    /// let updates = client.get_updates(None, Some(30)).await?;
    /// for update in updates.result {
    ///     println!("Update ID: {}", update.update_id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_updates(
        &self,
        offset: Option<i64>,
        timeout: Option<i64>,
    ) -> Result<GetUpdatesResponse> {
        let url = format!("{}/getUpdates", self.base_url);

        let mut params = vec![];
        if let Some(offset) = offset {
            params.push(("offset", offset.to_string()));
        }
        if let Some(timeout) = timeout {
            params.push(("timeout", timeout.to_string()));
        }

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .context("Failed to send getUpdates request")?;

        let result = response
            .json::<GetUpdatesResponse>()
            .await
            .context("Failed to parse getUpdates response")?;

        Ok(result)
    }

    /// Sends a text message to a specified chat.
    ///
    /// # Arguments
    ///
    /// * `chat_id` - Unique identifier for the target chat
    /// * `text` - Text of the message to be sent (1-4096 characters)
    /// * `message_thread_id` - Optional unique identifier for the target message thread (topic) in forum groups
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The network request fails
    /// - The chat_id is invalid
    /// - The bot doesn't have permission to send messages in this chat
    /// - The text is empty or too long
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use telegram_bot_debugger::telegram::TelegramClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let client = TelegramClient::new("YOUR_TOKEN".to_string());
    ///
    /// // Send a simple message
    /// let response = client.send_message(123456789, "Hello, World!", None).await?;
    /// if response.ok {
    ///     println!("Message sent successfully!");
    /// }
    ///
    /// // Send to a specific topic in a forum group
    /// client.send_message(123456789, "Topic message", Some(42)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_message(
        &self,
        chat_id: i64,
        text: &str,
        message_thread_id: Option<i64>,
    ) -> Result<SendMessageResponse> {
        let url = format!("{}/sendMessage", self.base_url);

        let mut body = json!({
            "chat_id": chat_id,
            "text": text,
        });

        if let Some(thread_id) = message_thread_id {
            body["message_thread_id"] = json!(thread_id);
        }

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("Failed to send message")?;

        let result = response
            .json::<SendMessageResponse>()
            .await
            .context("Failed to parse sendMessage response")?;

        Ok(result)
    }

    /// Returns a reference to the bot token.
    ///
    /// This is primarily for internal use or debugging purposes.
    pub fn get_token(&self) -> &str {
        &self.token
    }

    /// Retrieves the current webhook configuration.
    ///
    /// Use this method to get current webhook status. This will return information about
    /// the webhook URL, pending updates, and any errors that occurred.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The network request fails
    /// - The response cannot be parsed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use telegram_bot_debugger::telegram::TelegramClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let client = TelegramClient::new("YOUR_TOKEN".to_string());
    ///
    /// let webhook_info = client.get_webhook_info().await?;
    /// if webhook_info.ok {
    ///     if let Some(info) = webhook_info.result {
    ///         if info.url.is_empty() {
    ///             println!("No webhook is set");
    ///         } else {
    ///             println!("Webhook URL: {}", info.url);
    ///             println!("Pending updates: {}", info.pending_update_count);
    ///         }
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_webhook_info(&self) -> Result<GetWebhookInfoResponse> {
        let url = format!("{}/getWebhookInfo", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send getWebhookInfo request")?;

        let result = response
            .json::<GetWebhookInfoResponse>()
            .await
            .context("Failed to parse getWebhookInfo response")?;

        Ok(result)
    }

    /// Sets a new webhook URL for receiving updates.
    ///
    /// Use this method to specify a URL and receive incoming updates via an outgoing webhook.
    /// Whenever there is an update for the bot, Telegram will send an HTTPS POST request to the
    /// specified URL.
    ///
    /// # Arguments
    ///
    /// * `webhook_url` - HTTPS URL to send updates to. Use an empty string to remove webhook integration
    /// * `max_connections` - Optional maximum allowed number of simultaneous HTTPS connections to the webhook (1-100, default 40)
    /// * `allowed_updates` - Optional list of update types you want your bot to receive (e.g., ["message", "edited_channel_post"])
    /// * `drop_pending_updates` - Optional flag to drop all pending updates before setting the new webhook
    /// * `secret_token` - Optional secret token to be sent in a header "X-Telegram-Bot-Api-Secret-Token" (1-256 characters)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The network request fails
    /// - The webhook URL is invalid (must be HTTPS)
    /// - The response cannot be parsed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use telegram_bot_debugger::telegram::TelegramClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let client = TelegramClient::new("YOUR_TOKEN".to_string());
    ///
    /// // Set a webhook
    /// let response = client.set_webhook(
    ///     "https://example.com/webhook",
    ///     None,
    ///     None,
    ///     None,
    ///     None,
    /// ).await?;
    ///
    /// if response.ok {
    ///     println!("Webhook set successfully");
    /// }
    ///
    /// // Set webhook with options
    /// let response = client.set_webhook(
    ///     "https://example.com/webhook",
    ///     Some(100),
    ///     Some(vec!["message".to_string(), "callback_query".to_string()]),
    ///     Some(true),
    ///     Some("my_secret_token".to_string()),
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_webhook(
        &self,
        webhook_url: &str,
        max_connections: Option<i32>,
        allowed_updates: Option<Vec<String>>,
        drop_pending_updates: Option<bool>,
        secret_token: Option<String>,
    ) -> Result<SetWebhookResponse> {
        let url = format!("{}/setWebhook", self.base_url);

        let mut body = json!({
            "url": webhook_url,
        });

        if let Some(max_conn) = max_connections {
            body["max_connections"] = json!(max_conn);
        }
        if let Some(updates) = allowed_updates {
            body["allowed_updates"] = json!(updates);
        }
        if let Some(drop_pending) = drop_pending_updates {
            body["drop_pending_updates"] = json!(drop_pending);
        }
        if let Some(token) = secret_token {
            body["secret_token"] = json!(token);
        }

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("Failed to send setWebhook request")?;

        let result = response
            .json::<SetWebhookResponse>()
            .await
            .context("Failed to parse setWebhook response")?;

        Ok(result)
    }

    /// Removes the webhook integration.
    ///
    /// Use this method to remove webhook integration if you decide to switch back to getUpdates.
    /// Returns True on success.
    ///
    /// # Arguments
    ///
    /// * `drop_pending_updates` - Optional flag to drop all pending updates
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The network request fails
    /// - The response cannot be parsed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use telegram_bot_debugger::telegram::TelegramClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let client = TelegramClient::new("YOUR_TOKEN".to_string());
    ///
    /// // Delete webhook without dropping pending updates
    /// let response = client.delete_webhook(None).await?;
    /// if response.ok {
    ///     println!("Webhook deleted successfully");
    /// }
    ///
    /// // Delete webhook and drop all pending updates
    /// let response = client.delete_webhook(Some(true)).await?;
    /// if response.ok {
    ///     println!("Webhook deleted and pending updates dropped");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_webhook(
        &self,
        drop_pending_updates: Option<bool>,
    ) -> Result<DeleteWebhookResponse> {
        let url = format!("{}/deleteWebhook", self.base_url);

        let mut body = json!({});
        if let Some(drop_pending) = drop_pending_updates {
            body["drop_pending_updates"] = json!(drop_pending);
        }

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("Failed to send deleteWebhook request")?;

        let result = response
            .json::<DeleteWebhookResponse>()
            .await
            .context("Failed to parse deleteWebhook response")?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{Mock, Server};

    async fn create_mock_client(server: &Server, token: &str) -> TelegramClient {
        let base_url = format!("{}/bot{}", server.url(), token);
        TelegramClient {
            token: token.to_string(),
            client: Client::new(),
            base_url,
        }
    }

    fn create_success_get_me_mock(server: &mut Server) -> Mock {
        server
            .mock("GET", "/bottest_token/getMe")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "ok": true,
                "result": {
                    "id": 123456789,
                    "is_bot": true,
                    "first_name": "Test Bot",
                    "username": "test_bot"
                }
            }"#,
            )
            .create()
    }

    fn create_error_get_me_mock(server: &mut Server) -> Mock {
        server
            .mock("GET", "/botinvalid_token/getMe")
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "ok": false,
                "error_code": 401,
                "description": "Unauthorized"
            }"#,
            )
            .create()
    }

    fn create_success_get_updates_mock(server: &mut Server) -> Mock {
        server
            .mock("GET", "/bottest_token/getUpdates")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "ok": true,
                "result": [
                    {
                        "update_id": 123,
                        "message": {
                            "message_id": 1,
                            "from": {
                                "id": 100,
                                "is_bot": false,
                                "first_name": "User"
                            },
                            "chat": {
                                "id": 100,
                                "type": "private",
                                "first_name": "User"
                            },
                            "date": 1000,
                            "text": "Hello"
                        }
                    }
                ]
            }"#,
            )
            .create()
    }

    fn create_empty_updates_mock(server: &mut Server) -> Mock {
        server
            .mock("GET", "/bottest_token/getUpdates")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "ok": true,
                "result": []
            }"#,
            )
            .create()
    }

    fn create_success_send_message_mock(server: &mut Server) -> Mock {
        server
            .mock("POST", "/bottest_token/sendMessage")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "ok": true,
                "result": {
                    "message_id": 42,
                    "from": {
                        "id": 123456789,
                        "is_bot": true,
                        "first_name": "Test Bot"
                    },
                    "chat": {
                        "id": 100,
                        "type": "private",
                        "first_name": "User"
                    },
                    "date": 1000,
                    "text": "Test message"
                }
            }"#,
            )
            .create()
    }

    fn create_error_send_message_mock(server: &mut Server) -> Mock {
        server
            .mock("POST", "/bottest_token/sendMessage")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "ok": false,
                "error_code": 400,
                "description": "Bad Request: chat not found"
            }"#,
            )
            .create()
    }

    #[tokio::test]
    async fn test_get_me_success() {
        let mut server = Server::new_async().await;
        let _mock = create_success_get_me_mock(&mut server);
        let client = create_mock_client(&server, "test_token").await;

        let result = client.get_me().await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.ok);
        assert!(response.result.is_some());

        let user = response.result.unwrap();
        assert_eq!(user.id, 123456789);
        assert!(user.is_bot);
        assert_eq!(user.username, Some("test_bot".to_string()));
    }

    #[tokio::test]
    async fn test_get_me_unauthorized() {
        let mut server = Server::new_async().await;
        let _mock = create_error_get_me_mock(&mut server);
        let client = create_mock_client(&server, "invalid_token").await;

        let result = client.get_me().await;
        // The request succeeds but returns an error response
        // The actual parsing will succeed, and we'll get an ok=false response
        // Note: Depending on how the API is designed, this might be an Err or Ok with ok=false
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_updates_success() {
        let mut server = Server::new_async().await;
        let _mock = create_success_get_updates_mock(&mut server);
        let client = create_mock_client(&server, "test_token").await;

        let result = client.get_updates(None, None).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.ok);
        assert_eq!(response.result.len(), 1);
        assert_eq!(response.result[0].update_id, 123);
    }

    #[tokio::test]
    async fn test_get_updates_empty_result() {
        let mut server = Server::new_async().await;
        let _mock = create_empty_updates_mock(&mut server);
        let client = create_mock_client(&server, "test_token").await;

        let result = client.get_updates(None, None).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.ok);
        assert_eq!(response.result.len(), 0);
    }

    #[tokio::test]
    async fn test_get_updates_with_offset() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/bottest_token/getUpdates")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "offset".into(),
                "100".into(),
            )]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"ok": true, "result": []}"#)
            .create();

        let client = create_mock_client(&server, "test_token").await;
        let result = client.get_updates(Some(100), None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_updates_with_timeout() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/bottest_token/getUpdates")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "timeout".into(),
                "30".into(),
            )]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"ok": true, "result": []}"#)
            .create();

        let client = create_mock_client(&server, "test_token").await;
        let result = client.get_updates(None, Some(30)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_message_success() {
        let mut server = Server::new_async().await;
        let _mock = create_success_send_message_mock(&mut server);
        let client = create_mock_client(&server, "test_token").await;

        let result = client.send_message(100, "Test message", None).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.ok);
        assert!(response.result.is_some());

        let message = response.result.unwrap();
        assert_eq!(message.message_id, 42);
        assert_eq!(message.text, Some("Test message".to_string()));
    }

    #[tokio::test]
    async fn test_send_message_with_thread_id() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/bottest_token/sendMessage")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "ok": true,
                "result": {
                    "message_id": 50,
                    "from": {"id": 1, "is_bot": true, "first_name": "Bot"},
                    "chat": {"id": 100, "type": "supergroup", "title": "Forum"},
                    "date": 1000,
                    "text": "Forum message",
                    "message_thread_id": 42
                }
            }"#,
            )
            .create();

        let client = create_mock_client(&server, "test_token").await;
        let result = client.send_message(100, "Forum message", Some(42)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.ok);
        let message = response.result.unwrap();
        assert_eq!(message.message_thread_id, Some(42));
    }

    #[tokio::test]
    async fn test_send_message_error() {
        let mut server = Server::new_async().await;
        let _mock = create_error_send_message_mock(&mut server);
        let client = create_mock_client(&server, "test_token").await;

        let result = client.send_message(999, "Test", None).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.ok);
        assert!(response.result.is_none());
        assert!(response.description.is_some());
    }

    #[tokio::test]
    async fn test_invalid_json_response() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/bottest_token/getMe")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("not valid json")
            .create();

        let client = create_mock_client(&server, "test_token").await;
        let result = client.get_me().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_network_error() {
        // Create a client with an invalid URL that will fail to connect
        let client = TelegramClient {
            token: "test".to_string(),
            client: Client::new(),
            base_url: "http://invalid-domain-that-does-not-exist-12345.com/bottest".to_string(),
        };

        let result = client.get_me().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_token() {
        let server = Server::new_async().await;
        let client = create_mock_client(&server, "my_secret_token").await;
        assert_eq!(client.get_token(), "my_secret_token");
    }

    #[tokio::test]
    async fn test_url_construction() {
        let client = TelegramClient::new("my_token".to_string());
        assert_eq!(client.base_url, "https://api.telegram.org/botmy_token");
    }

    #[tokio::test]
    async fn test_send_message_empty_text() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/bottest_token/sendMessage")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "ok": true,
                "result": {
                    "message_id": 1,
                    "from": {"id": 1, "is_bot": true, "first_name": "Bot"},
                    "chat": {"id": 100, "type": "private", "first_name": "User"},
                    "date": 1000,
                    "text": ""
                }
            }"#,
            )
            .create();

        let client = create_mock_client(&server, "test_token").await;
        let result = client.send_message(100, "", None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_message_long_text() {
        let mut server = Server::new_async().await;
        let long_text = "A".repeat(4096);
        let _mock = server
            .mock("POST", "/bottest_token/sendMessage")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(format!(
                r#"{{
                "ok": true,
                "result": {{
                    "message_id": 1,
                    "from": {{"id": 1, "is_bot": true, "first_name": "Bot"}},
                    "chat": {{"id": 100, "type": "private", "first_name": "User"}},
                    "date": 1000,
                    "text": "{long_text}"
                }}
            }}"#
            ))
            .create();

        let client = create_mock_client(&server, "test_token").await;
        let result = client.send_message(100, &long_text, None).await;
        assert!(result.is_ok());
    }

    // Webhook management tests
    #[tokio::test]
    async fn test_get_webhook_info_with_webhook_set() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/bottest_token/getWebhookInfo")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "ok": true,
                "result": {
                    "url": "https://example.com/webhook",
                    "has_custom_certificate": false,
                    "pending_update_count": 5,
                    "max_connections": 40
                }
            }"#,
            )
            .create();

        let client = create_mock_client(&server, "test_token").await;
        let result = client.get_webhook_info().await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.ok);
        assert!(response.result.is_some());

        let webhook_info = response.result.unwrap();
        assert_eq!(webhook_info.url, "https://example.com/webhook");
        assert!(!webhook_info.has_custom_certificate);
        assert_eq!(webhook_info.pending_update_count, 5);
        assert_eq!(webhook_info.max_connections, Some(40));
    }

    #[tokio::test]
    async fn test_get_webhook_info_no_webhook() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/bottest_token/getWebhookInfo")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "ok": true,
                "result": {
                    "url": "",
                    "has_custom_certificate": false,
                    "pending_update_count": 0
                }
            }"#,
            )
            .create();

        let client = create_mock_client(&server, "test_token").await;
        let result = client.get_webhook_info().await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.ok);
        assert!(response.result.is_some());

        let webhook_info = response.result.unwrap();
        assert_eq!(webhook_info.url, "");
        assert_eq!(webhook_info.pending_update_count, 0);
    }

    #[tokio::test]
    async fn test_get_webhook_info_with_error() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/bottest_token/getWebhookInfo")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "ok": true,
                "result": {
                    "url": "https://example.com/webhook",
                    "has_custom_certificate": false,
                    "pending_update_count": 10,
                    "last_error_date": 1234567890,
                    "last_error_message": "Connection timeout",
                    "max_connections": 40
                }
            }"#,
            )
            .create();

        let client = create_mock_client(&server, "test_token").await;
        let result = client.get_webhook_info().await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.ok);
        assert!(response.result.is_some());

        let webhook_info = response.result.unwrap();
        assert_eq!(webhook_info.url, "https://example.com/webhook");
        assert_eq!(webhook_info.pending_update_count, 10);
        assert_eq!(webhook_info.last_error_date, Some(1234567890));
        assert_eq!(
            webhook_info.last_error_message,
            Some("Connection timeout".to_string())
        );
    }

    #[tokio::test]
    async fn test_get_webhook_info_network_error() {
        let client = TelegramClient {
            token: "test".to_string(),
            client: Client::new(),
            base_url: "http://invalid-domain-that-does-not-exist-12345.com/bottest".to_string(),
        };

        let result = client.get_webhook_info().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_set_webhook_success() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/bottest_token/setWebhook")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "ok": true,
                "description": "Webhook was set"
            }"#,
            )
            .create();

        let client = create_mock_client(&server, "test_token").await;
        let result = client
            .set_webhook("https://example.com/webhook", None, None, None, None)
            .await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.ok);
        assert_eq!(response.description, Some("Webhook was set".to_string()));
    }

    #[tokio::test]
    async fn test_set_webhook_with_options() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/bottest_token/setWebhook")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "ok": true,
                "description": "Webhook was set"
            }"#,
            )
            .create();

        let client = create_mock_client(&server, "test_token").await;
        let result = client
            .set_webhook(
                "https://example.com/webhook",
                Some(100),
                Some(vec!["message".to_string(), "callback_query".to_string()]),
                Some(true),
                Some("my_secret_token".to_string()),
            )
            .await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.ok);
    }

    #[tokio::test]
    async fn test_set_webhook_invalid_url() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/bottest_token/setWebhook")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "ok": false,
                "description": "Bad Request: invalid webhook URL"
            }"#,
            )
            .create();

        let client = create_mock_client(&server, "test_token").await;
        let result = client
            .set_webhook("http://example.com/webhook", None, None, None, None)
            .await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.ok);
        assert!(response.description.is_some());
        assert!(
            response
                .description
                .unwrap()
                .contains("invalid webhook URL")
        );
    }

    #[tokio::test]
    async fn test_set_webhook_error_response() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/bottest_token/setWebhook")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "ok": false,
                "description": "Bad Request: webhook URL is invalid"
            }"#,
            )
            .create();

        let client = create_mock_client(&server, "test_token").await;
        let result = client
            .set_webhook("invalid-url", None, None, None, None)
            .await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.ok);
        assert!(response.description.is_some());
    }

    #[tokio::test]
    async fn test_delete_webhook_success() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/bottest_token/deleteWebhook")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "ok": true,
                "description": "Webhook was deleted"
            }"#,
            )
            .create();

        let client = create_mock_client(&server, "test_token").await;
        let result = client.delete_webhook(None).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.ok);
        assert_eq!(
            response.description,
            Some("Webhook was deleted".to_string())
        );
    }

    #[tokio::test]
    async fn test_delete_webhook_with_drop_pending() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/bottest_token/deleteWebhook")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "ok": true,
                "description": "Webhook was deleted"
            }"#,
            )
            .create();

        let client = create_mock_client(&server, "test_token").await;
        let result = client.delete_webhook(Some(true)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.ok);
    }

    #[tokio::test]
    async fn test_delete_webhook_no_webhook_set() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/bottest_token/deleteWebhook")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "ok": true,
                "description": "Webhook is already deleted"
            }"#,
            )
            .create();

        let client = create_mock_client(&server, "test_token").await;
        let result = client.delete_webhook(None).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.ok);
        assert!(response.description.is_some());
    }

    #[tokio::test]
    async fn test_delete_webhook_error() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/bottest_token/deleteWebhook")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "ok": false,
                "description": "Bad Request: some error"
            }"#,
            )
            .create();

        let client = create_mock_client(&server, "test_token").await;
        let result = client.delete_webhook(None).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.ok);
        assert!(response.description.is_some());
    }
}
