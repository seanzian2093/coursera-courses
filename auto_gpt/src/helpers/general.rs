use std::fs;

use reqwest::Client;
use serde::de::DeserializeOwned;

use crate::{apis::call_request::call_gpt, models::general::llm::Message};

use super::command_line::PrintCommand;

const CODE_TEMPLATE_PATH: &str =
    "/users/s0046425/git_projects/2025/coursera-courses/web_template/src/code_template.rs";
pub const WEB_SERVER_PROJECT_PATH: &str =
    "/users/s0046425/git_projects/2025/coursera-courses/web_template/";
const EXEC_MAIN_PATH: &str =
    "/users/s0046425/git_projects/2025/coursera-courses/web_template/src/main.rs";
const API_SCHEMA_PATH: &str =
    "/users/s0046425/git_projects/2025/coursera-courses/auto_gpt/schemas/api_schema.json";

// Takes a function from `ai_functions` and a dummy input, generate a prompt for LLM
pub fn extend_ai_function(ai_func: fn(&str) -> &'static str, func_input: &str) -> Message {
    let ai_function_str = ai_func(func_input);
    // Extend the string to encourage only printing the output
    let msg = format!(
        "FUNCTION {}
    INSTRUCTION: You are a function printer. You ONLY print the results of functions.
    Nothing else. No commentary. Here is the input to the function {}.
    Print out what the function will return.",
        ai_function_str, func_input
    );

    // Return message
    Message {
        role: "system".to_string(),
        content: msg,
    }
}

// Perform call to LLM
pub async fn ai_task_request(
    msg_context: String,
    agent_position: &str,
    agent_operation: &str,
    function_pass: for<'a> fn(&'a str) -> &'static str,
) -> String {
    let extended_msg = extend_ai_function(function_pass, &msg_context);

    PrintCommand::AICall.print_agent_message(agent_position, agent_operation);

    let llm_response_res = call_gpt(vec![extended_msg.clone()]).await;

    // Handle error- call it again and if error again, panic
    match llm_response_res {
        Ok(llm_response) => llm_response,
        Err(_) => call_gpt(vec![extended_msg.clone()])
            .await
            .expect("Failed twice to call LLM"),
    }
}

// Perform call to LLM - Decoded
pub async fn ai_task_request_decoded<T: DeserializeOwned>(
    msg_context: String,
    agent_position: &str,
    agent_operation: &str,
    function_pass: for<'a> fn(&'a str) -> &'static str,
) -> T {
    let llm_response =
        ai_task_request(msg_context, agent_position, agent_operation, function_pass).await;

    let decoded_response: T = serde_json::from_str(llm_response.as_str())
        .expect("Failed to decode a LLM response from serde_json");
    decoded_response
}

// Check if url is valid
pub async fn check_status_code(client: &Client, url: &str) -> Result<u16, reqwest::Error> {
    let response = client.get(url).send().await?;
    Ok(response.status().as_u16())
}

// Get code template
pub fn read_code_template_contents() -> String {
    let path = String::from(CODE_TEMPLATE_PATH);
    fs::read_to_string(path).expect("Failed to read code template")
}

// Get exec main
pub fn read_exec_main_contents() -> String {
    let path = String::from(EXEC_MAIN_PATH);
    fs::read_to_string(path).expect("Failed to read main.rs file")
}

// Save new backend code
pub fn save_backend_code(contents: &String) {
    let path = String::from(EXEC_MAIN_PATH);
    fs::write(path, contents).expect("Failed to write main.rs file")
}

// Save JSON API endpoint schema
pub fn save_api_endpoints(api_endpoints: &String) {
    let path = String::from(API_SCHEMA_PATH);
    fs::write(path, api_endpoints).expect("Failed to write api endpoints JSON file")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai_functions::aifunc_managing::convert_user_input_to_goal;

    #[test]
    fn test_extend_ai_function() {
        let extended_msg = extend_ai_function(convert_user_input_to_goal, "dummy input");
        assert_eq!(extended_msg.role, "system".to_string());
    }

    #[tokio::test]
    async fn test_ai_task_request() {
        let ai_func_param = "Build a website for making stock price requests".to_string();
        let llm_response = ai_task_request(
            ai_func_param,
            "Managing Agent",
            "Defining user requirements",
            convert_user_input_to_goal,
        )
        .await;
        assert!(llm_response.len() > 20);
    }
}
