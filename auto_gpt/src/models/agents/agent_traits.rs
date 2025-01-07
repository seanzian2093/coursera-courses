use crate::models::agent_basic::basic_agent::BasicAgent;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RouteObject {
    pub is_route_dynamic: bool,
    pub method: String,
    pub request_body: serde_json::Value,
    pub response: serde_json::Value,
    pub route: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ProjectScope {
    pub is_crud_required: bool,
    pub is_user_login_and_logout: bool,
    pub is_external_urls_required: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FactSheet {
    pub project_description: String,
    pub project_scope: Option<ProjectScope>,
    pub external_urls: Option<Vec<String>>,
    pub backend_code: Option<String>,
    pub api_endpoint_schema: Option<Vec<RouteObject>>,
}

#[async_trait]
pub trait SpecialFunctions: Debug {
    // Used to get the attributes from the agent
    fn get_attributes_from_agent(&self) -> &BasicAgent;

    // Allow agent to perform its functionality
    async fn execute(&mut self, facsheet: &mut FactSheet)
        -> Result<(), Box<dyn std::error::Error>>;
}
