use crate::ai_functions::aifunc_architect::{print_project_scope, print_site_urls};
use crate::helpers::command_line::PrintCommand;
use crate::helpers::general::{ai_task_request_decoded, check_status_code};
use crate::models::agent_basic::basic_agent::{AgentState, BasicAgent};
use crate::models::agent_basic::basic_traits::BasicTraits;
use crate::models::agents::agent_traits::{FactSheet, ProjectScope, SpecialFunctions};

use async_trait::async_trait;
use reqwest::Client;
use std::os::macos::raw::stat;
use std::time::Duration;

// Solutions Architect
#[derive(Debug)]
pub struct AgentSolutionArchitect {
    attributes: BasicAgent,
}

impl AgentSolutionArchitect {
    pub fn new() -> Self {
        let attributes = BasicAgent {
            objective: "Gathers information".to_string(),
            position: "Solutions Architect".to_string(),
            state: AgentState::Discovery,
            memory: vec![],
        };

        Self { attributes }
    }
    // Retrive project scope
    async fn call_project_scope(&mut self, factsheet: &mut FactSheet) -> ProjectScope {
        let msg_context = format!("{}", factsheet.project_description);
        let ai_response = ai_task_request_decoded::<ProjectScope>(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_project_scope),
            print_project_scope,
        )
        .await;

        factsheet.project_scope = Some(ai_response.clone());
        self.attributes.update_state(AgentState::Finished);

        ai_response
    }

    // Retrive external urls
    async fn call_determine_external_urls(
        &mut self,
        factsheet: &mut FactSheet,
        msg_context: String,
    ) {
        let ai_response = ai_task_request_decoded::<Vec<String>>(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_site_urls),
            print_site_urls,
        )
        .await;

        factsheet.external_urls = Some(ai_response);
        self.attributes.update_state(AgentState::UnitTesting);
    }
}

#[async_trait]
impl SpecialFunctions for AgentSolutionArchitect {
    fn get_attributes_from_agent(&self) -> &BasicAgent {
        &self.attributes
    }

    async fn execute(
        &mut self,
        factsheet: &mut FactSheet,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Infinite loop - be careful
        while self.attributes.state != AgentState::Finished {
            match self.attributes.state {
                AgentState::Discovery => {
                    let project_scope = self.call_project_scope(factsheet).await;
                    // confirm external urls
                    if project_scope.is_external_urls_required {
                        self.call_determine_external_urls(
                            factsheet,
                            factsheet.project_description.clone(),
                        )
                        .await;
                        self.attributes.state = AgentState::UnitTesting;
                    }
                }

                AgentState::UnitTesting => {
                    let mut exclude_urls: Vec<String> = vec![];
                    let client = Client::builder()
                        .timeout(Duration::from_secs(10))
                        .build()
                        .unwrap();

                    // Find faulty urls
                    let urls = factsheet
                        .external_urls
                        .as_ref()
                        .expect("No URL object on factsheet");

                    for url in urls {
                        let endpoint_str = format!("Testing URL: {}", url);
                        PrintCommand::UnitTest.print_agent_message(
                            self.attributes.position.as_str(),
                            endpoint_str.as_str(),
                        );

                        // Test URL
                        match check_status_code(&client, url).await {
                            Ok(status_code) => {
                                if status_code != 200 {
                                    exclude_urls.push(url.clone());
                                }
                            }
                            Err(e) => println!("Error: {} - {}", url, e),
                        }
                    }

                    // Remove faulty urls
                    if exclude_urls.len() > 0 {
                        let new_url: Vec<String> = factsheet
                            .external_urls
                            .as_ref()
                            .unwrap()
                            .iter()
                            .filter(|url| !exclude_urls.contains(url))
                            .cloned()
                            .collect();

                        factsheet.external_urls = Some(new_url);
                    }

                    // Confirm done
                    self.attributes.state = AgentState::Finished;
                }

                // Default state to finished
                _ => {
                    self.attributes.state = AgentState::Finished;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_solution_architect() {
        let mut agent = AgentSolutionArchitect::new();

        let mut factsheet = FactSheet {
            project_description: "Build a full stack website with user login and logout that shows latest Forex prices.".to_string(),
            project_scope: None,
            external_urls: None,
            backend_code: None,
            api_endpoint_schema: None,
        };

        agent
            .execute(&mut factsheet)
            .await
            .expect("Unable to execute Solution Architect agent");

        assert!(factsheet.project_scope.is_some());
        assert!(factsheet.external_urls.is_some());

        dbg!(factsheet);
    }
}
