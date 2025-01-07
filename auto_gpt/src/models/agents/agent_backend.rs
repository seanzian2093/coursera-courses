use crate::ai_functions::aifunc_backend::{
    print_backend_webserver_code, print_fixed_code, print_improved_webserver_code,
    print_rest_api_endpoints,
};
use crate::helpers::general::{
    check_status_code, read_code_template_contents, read_exec_main_contents, save_api_endpoints,
    save_backend_code, WEB_SERVER_PROJECT_PATH,
};

use crate::helpers::command_line::{confirm_safe_code, PrintCommand};
use crate::helpers::general::ai_task_request;
use crate::models::agent_basic::basic_agent::{AgentState, BasicAgent};
use crate::models::agents::agent_traits::{FactSheet, RouteObject, SpecialFunctions};

use async_trait::async_trait;
use reqwest::Client;
use std::env::current_dir;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time;

#[derive(Debug)]
pub struct AgentBackendDeveloper {
    attributes: BasicAgent,
    bug_errors: Option<String>,
    bug_count: u8,
}

impl AgentBackendDeveloper {
    pub fn new() -> Self {
        let attributes = BasicAgent {
            objective: "Develop backend code for webserver and json database".to_string(),
            position: "Backend Developer".to_string(),
            state: AgentState::Discovery,
            memory: vec![],
        };

        Self {
            attributes,
            bug_errors: None,
            bug_count: 0,
        }
    }

    async fn call_initial_backend_code(&mut self, factsheet: &mut FactSheet) {
        let code_template_str = read_code_template_contents();

        // Concatenate instruction
        let mut msg_context = format!(
            "CODE TEMPLATE: {} \n PROJECT_DESCRIPTION: {} \n",
            code_template_str, factsheet.project_description
        );

        let ai_response = ai_task_request(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_backend_webserver_code),
            print_backend_webserver_code,
        )
        .await;

        save_backend_code(&ai_response);
        factsheet.backend_code = Some(ai_response);
    }

    async fn call_improved_backend_code(&mut self, factsheet: &mut FactSheet) {
        // Display generated code
        let mut msg_context = format!(
            "CODE TEMPLATE: {:?} \n PROJECT_DESCRIPTION: {:?} \n",
            factsheet.backend_code, factsheet
        );

        // Generate improved code
        let ai_response = ai_task_request(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_improved_webserver_code),
            print_improved_webserver_code,
        )
        .await;

        save_backend_code(&ai_response);
        factsheet.backend_code = Some(ai_response);
    }

    async fn call_fix_code_bugs(&mut self, factsheet: &mut FactSheet) {
        // Display error and bugs
        let mut msg_context = format!(
            "BROKE_CODE: {:?} \n ERROR_BUGS: {:?} \n
            THIS FUNCTIONS ONLY OUTPUTS CODE. JUST OUTPUT THE CODE.",
            factsheet.backend_code, self.bug_errors
        );

        // Generate fixed code
        let ai_response = ai_task_request(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_fixed_code),
            print_fixed_code,
        )
        .await;

        save_backend_code(&ai_response);
        factsheet.backend_code = Some(ai_response);
    }

    async fn call_extract_rest_api_endpoints(&self) -> String {
        let backend_code = read_exec_main_contents();

        let msg_context = format!("CODE_INPUT: {}", backend_code);

        // Generate fixed code
        let ai_response = ai_task_request(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_rest_api_endpoints),
            print_rest_api_endpoints,
        )
        .await;

        ai_response
    }
}

#[async_trait]
impl SpecialFunctions for AgentBackendDeveloper {
    fn get_attributes_from_agent(&self) -> &BasicAgent {
        &self.attributes
    }
    async fn execute(
        &mut self,
        factsheet: &mut FactSheet,
    ) -> Result<(), Box<dyn std::error::Error>> {
        while self.attributes.state != AgentState::Finished {
            match &self.attributes.state {
                AgentState::Discovery => {
                    self.call_initial_backend_code(factsheet).await;
                    self.attributes.state = AgentState::Working;
                    continue;
                }
                AgentState::Working => {
                    if self.bug_count == 0 {
                        self.call_improved_backend_code(factsheet).await;
                    } else {
                        self.call_fix_code_bugs(factsheet).await;
                    }
                    self.attributes.state = AgentState::UnitTesting;
                    continue;
                }
                AgentState::UnitTesting => {
                    // Guard: ensure AI safety
                    PrintCommand::UnitTest.print_agent_message(
                        self.attributes.position.as_str(),
                        "Backend code unit testing: requring user input",
                    );

                    // Get user input
                    let is_safe_code = confirm_safe_code();
                    if !is_safe_code {
                        panic!("Code is not safe to run");
                    }

                    // BUild and test code
                    PrintCommand::UnitTest.print_agent_message(
                        self.attributes.position.as_str(),
                        "Backend code unit testing: building",
                    );

                    let build_backend_server = Command::new("cargo")
                        .arg("build")
                        .current_dir(WEB_SERVER_PROJECT_PATH)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .output()
                        .expect("Failed to build backend server");
                    // Check if build was successful
                    if build_backend_server.status.success() {
                        self.bug_count = 0;
                        PrintCommand::UnitTest.print_agent_message(
                            self.attributes.position.as_str(),
                            "Backend code unit testing: build successful",
                        );
                    } else {
                        let err_arr = build_backend_server.stderr;
                        let err_str = String::from_utf8(err_arr).unwrap();
                        // update error status
                        self.bug_count += 1;
                        self.bug_errors = Some(err_str);

                        // Exit if too many errors
                        if self.bug_count > 10 {
                            PrintCommand::UnitTest.print_agent_message(
                                self.attributes.position.as_str(),
                                "Backend code unit testing: too many errors",
                            );
                            panic!("Error: Too many errors in code");
                        }

                        // back to working state
                        self.attributes.state = AgentState::Working;
                        continue;
                    };

                    // Extract and test API endpoints
                    let api_endpoints_str = self.call_extract_rest_api_endpoints().await;

                    // Convert api enpoints to values
                    let api_enpoints: Vec<RouteObject> =
                        serde_json::from_str(api_endpoints_str.as_str())
                            .expect("Error parsing api endpoints");

                    // Check endpoints
                    let check_endpoints: Vec<RouteObject> = api_enpoints
                        .iter()
                        .filter(|&ro| ro.method == "GET" && ro.is_route_dynamic == false)
                        .cloned()
                        .collect();

                    // Save api endpoints
                    factsheet.api_endpoint_schema = Some(check_endpoints.clone());

                    // Run backend server
                    PrintCommand::UnitTest.print_agent_message(
                        self.attributes.position.as_str(),
                        "Backend code unit testing: running server",
                    );
                    let mut run_backend_server = Command::new("cargo")
                        .arg("run")
                        .current_dir(WEB_SERVER_PROJECT_PATH)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                        .expect("Failed to run backend server");

                    PrintCommand::UnitTest.print_agent_message(
                        self.attributes.position.as_str(),
                        "Backend code unit testing: launching tests on server in 5 secs",
                    );
                    let sec_sleep = time::Duration::from_secs(5);
                    time::sleep(sec_sleep).await;
                    // check status code
                    for endpoint in check_endpoints {
                        let testing_msg = format!("Testing endpoint: {}", endpoint.route);
                        PrintCommand::UnitTest.print_agent_message(
                            self.attributes.position.as_str(),
                            testing_msg.as_str(),
                        );

                        // create client
                        let client = Client::builder()
                            .timeout(Duration::from_secs(5))
                            .build()
                            .unwrap();

                        let url = format!("http://localhost:8000{}", endpoint.route);
                        match check_status_code(&client, &url).await {
                            Ok(status_code) => {
                                if status_code != 200 {
                                    let err_msg = format!(
                                        "Error: Status code is not 200 for endpoint: {}",
                                        endpoint.route
                                    );
                                    PrintCommand::Issue.print_agent_message(
                                        self.attributes.position.as_str(),
                                        testing_msg.as_str(),
                                    );
                                }
                            }

                            Err(e) => {
                                // kill server - kill $(lsof -t -i:8000)
                                run_backend_server
                                    .kill()
                                    .expect("Failed to kill backend server");
                                // error msg
                                let err_msg = format!("Error checking backend {}", e);
                                PrintCommand::Issue.print_agent_message(
                                    self.attributes.position.as_str(),
                                    err_msg.as_str(),
                                );
                            }
                        }
                    }
                    save_api_endpoints(&api_endpoints_str);

                    PrintCommand::Issue.print_agent_message(
                        self.attributes.position.as_str(),
                        "Backend code unit testing: all tests passed",
                    );
                    run_backend_server
                        .kill()
                        .expect("Failed to kill backend server on completion");

                    self.attributes.state = AgentState::Finished;
                }
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_writing_backend_code() {
        let mut agent = AgentBackendDeveloper::new();

        let factsheet_str: &str = r#"
        {
          "project_description": "build a website that fetches and return current time.",
          "project_scope": {
            "is_crud_required": false,
            "is_user_login_and_logout": false,
            "is_external_urls_required": false 
          },
          "external_urls": [],
          "backend_code": null,
          "api_endpoint_schema": null
        }"#;

        let mut factsheet: FactSheet = serde_json::from_str(factsheet_str).unwrap();

        // Control the starting of testing process
        // agent.attributes.state = AgentState::UnitTesting;
        agent.attributes.state = AgentState::Discovery;

        agent
            .execute(&mut factsheet)
            .await
            .expect("Error executing backend developer agent");
    }
}
