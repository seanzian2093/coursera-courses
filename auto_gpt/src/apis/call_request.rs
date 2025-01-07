use crate::models::general::llm::{APIResponse, ChatCompletion, Message};
use dotenv::dotenv;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use std::env;

// Call LLM
pub async fn call_gpt(messages: Vec<Message>) -> Result<String, Box<dyn std::error::Error + Send>> {
    dotenv().ok();

    // Extract LLM API information from environment variables - Azure OpenAI GPT4 specific
    let api_key = env::var("AZURE_OPENAI_GPT4_KEY").expect("AZURE_OPENAI_GPT4_KEY must be set");
    let api_version = env::var("AZURE_OPENAI_GPT4_API_VERSION")
        .expect("AZURE_OPENAI_GPT4_API_VERSION must be set");
    let endpoint =
        env::var("AZURE_OPENAI_GPT4_ENDPOINT").expect("AZURE_OPENAI_GPT4_ENDPOINT must be set");
    let deployment =
        env::var("AZURE_OPENAI_GPT4_DEPLOYMENT").expect("AZURE_OPENAI_GPT4_DEPLOYMENT must be set");

    // Confirm LLM url
    let llm_url = format!(
        "{}/openai/deployments/{}/chat/completions?api-version={}",
        endpoint, deployment, api_version
    );

    // Create headers
    let mut headers = HeaderMap::new();

    // Headers - api key
    headers.insert(
        "api-key",
        // use `unwrap` for now
        HeaderValue::from_str(api_key.as_str())
            .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?,
    );

    // Create reqwest client
    let client = Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?;

    // Create chatcompletion
    let chat_completion = ChatCompletion {
        model: "gpt-4".to_string(),
        messages,
        temperature: 0.1,
    };

    // Trouble shoot errors from LLM API for now
    // let res_raw = client
    //     .post(llm_url)
    //     .json(&chat_completion)
    //     .send()
    //     .await
    //     .unwrap();

    // dbg!(res_raw.text().await.unwrap());

    // Extract API response
    let res: APIResponse = client
        .post(llm_url)
        .json(&chat_completion)
        .send()
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?
        .json()
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?;

    // Send response
    Ok(res.choices[0].message.content.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_call_gpt() {
        let message = Message {
            role: "user".to_string(),
            content: "Hello, who are you?".to_string(),
        };

        let messages = vec![message];
        let res = call_gpt(messages).await;

        match res {
            Ok(res_str) => {
                dbg!(res_str);
                assert!(true)
            }
            Err(_) => assert!(false),
        }
    }
}
