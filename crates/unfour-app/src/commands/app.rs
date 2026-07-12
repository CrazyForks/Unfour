use crate::AppState;
use serde::Serialize;
use tauri::State;
use unfour_core::AppResult;

use super::trace_command;

/// The build edition surfaced to the frontend. Serialized as the lowercase
/// string so it stays stable and locale-independent (`"community"` / `"pro"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AppEditionDto {
    Community,
    Pro,
}

impl From<crate::AppEdition> for AppEditionDto {
    fn from(edition: crate::AppEdition) -> Self {
        match edition {
            crate::AppEdition::Community => Self::Community,
            crate::AppEdition::Pro => Self::Pro,
        }
    }
}

/// The distribution channel surfaced to the frontend. Serialized as the
/// lowercase string so it stays stable and locale-independent
/// (`"github"` / `"website"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AppDistributionDto {
    GitHub,
    Website,
}

impl From<crate::PackageKind> for AppDistributionDto {
    fn from(kind: crate::PackageKind) -> Self {
        match kind {
            crate::PackageKind::GitHub => Self::GitHub,
            crate::PackageKind::Website => Self::Website,
        }
    }
}

/// The release channel surfaced to the frontend. Serialized as the lowercase
/// string so it stays stable and locale-independent (`"test"` / `"stable"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AppChannelDto {
    Test,
    Stable,
}

impl From<crate::ReleaseChannel> for AppChannelDto {
    fn from(channel: crate::ReleaseChannel) -> Self {
        match channel {
            crate::ReleaseChannel::Test => Self::Test,
            crate::ReleaseChannel::Stable => Self::Stable,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub edition: AppEditionDto,
    pub distribution: AppDistributionDto,
    pub channel: AppChannelDto,
    pub commit: Option<String>,
}

/// Expose the compile-time build identity the frontend needs for the About
/// page. Every field comes from the Rust [`AppState`] config, never guessed
/// from the repo name, package name, env vars, or feature flags.
#[tauri::command]
pub async fn get_app_info(state: State<'_, AppState>) -> AppResult<AppInfo> {
    trace_command("get_app_info", async {
        Ok(AppInfo {
            name: state.config.app_name.clone(),
            version: state.config.app_version.clone(),
            edition: state.config.edition.into(),
            distribution: state.config.package_kind.into(),
            channel: state.config.channel.into(),
            commit: state.config.commit.clone(),
        })
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dto_values_are_stable_and_lowercase() {
        assert_eq!(
            serde_json::to_string(&AppEditionDto::Community).unwrap(),
            "\"community\""
        );
        assert_eq!(
            serde_json::to_string(&AppEditionDto::Pro).unwrap(),
            "\"pro\""
        );
        assert_eq!(
            serde_json::to_string(&AppDistributionDto::GitHub).unwrap(),
            "\"github\""
        );
        assert_eq!(
            serde_json::to_string(&AppDistributionDto::Website).unwrap(),
            "\"website\""
        );
        assert_eq!(
            serde_json::to_string(&AppChannelDto::Test).unwrap(),
            "\"test\""
        );
        assert_eq!(
            serde_json::to_string(&AppChannelDto::Stable).unwrap(),
            "\"stable\""
        );
    }

    #[test]
    fn app_info_serializes_with_camel_case_and_null_commit() {
        let info = AppInfo {
            name: "Unfour".to_string(),
            version: "0.1.0".to_string(),
            edition: AppEditionDto::Community,
            distribution: AppDistributionDto::GitHub,
            channel: AppChannelDto::Test,
            commit: None,
        };
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["name"], "Unfour");
        assert_eq!(json["version"], "0.1.0");
        assert_eq!(json["edition"], "community");
        assert_eq!(json["distribution"], "github");
        assert_eq!(json["channel"], "test");
        assert!(json["commit"].is_null());
    }
}
