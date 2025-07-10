use crate::{Environment, Commands};

#[derive(Debug)]
pub enum EstimatorError {
    InvalidHexFormat,
    InvalidCommand(String),
}

impl std::fmt::Display for EstimatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EstimatorError::InvalidHexFormat => write!(f, "Command must start with 0x"),
            EstimatorError::InvalidCommand(cmd) => write!(f, "Invalid command: {}", cmd),
        }
    }
}

impl std::error::Error for EstimatorError {}

pub struct EstimatorArgs {
    env: Environment,
    command: Commands,
}

impl EstimatorArgs {
    pub fn new(env: Environment, command: Commands) -> Result<Self, EstimatorError> {
        match &command {
            Commands::V2SendMessage { xcm, claimer, .. } => {
                if !xcm.starts_with("0x") {
                    return Err(EstimatorError::InvalidHexFormat);
                }
                if !claimer.starts_with("0x") {
                    return Err(EstimatorError::InvalidHexFormat);
                }
            }
        }
        Ok(EstimatorArgs { env, command })
    }
}

// TODO: Implement XCM building functionality
// pub fn build_asset_hub_xcm(xcm: Xcm) -> Xcm {
//     // XCM building implementation will be added later
// }
