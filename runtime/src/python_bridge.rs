use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// --- Core Data Structures for the Python Bridge ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PythonRequest {
    Execute {
        script: String,
        args: Vec<serde_json::Value>,
    },
    Eval {
        expression: String,
    },
    GetVariable {
        name: String,
    },
    SetVariable {
        name: String,
        value: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PythonResponse {
    Success {
        output: serde_json::Value,
        stdout: String,
        stderr: String,
    },
    Error {
        message: String,
        traceback: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonEnvironment {
    pub id: String,
    pub python_version: String,
    pub installed_packages: HashMap<String, String>,
}

// NOTE: Removed placeholder implementation structs:
// - PythonBridge
// - EnvironmentManager
// This file now only defines the data models for the Python bridge.
