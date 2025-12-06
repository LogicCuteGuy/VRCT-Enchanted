// Software update functionality for VRCT
// Implements checking for updates via GitHub API and downloading/launching the updater

use serde::{Deserialize, Serialize};
use tracing::{info, error};
use crate::utils::error::{Result, VrctError};

/// GitHub release information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub body: Option<String>,
    pub html_url: String,
    pub published_at: String,
    pub assets: Vec<GitHubAsset>,
}

/// GitHub release asset information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
    pub content_type: String,
}

/// Update information returned to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub available: bool,
    pub current_version: String,
    pub latest_version: String,
    pub release_notes: Option<String>,
    pub download_url: Option<String>,
    pub release_url: String,
}

/// Version comparison result
#[derive(Debug, Clone, PartialEq)]
pub enum VersionComparison {
    Newer,
    Same,
    Older,
}

/// Software updater for VRCT
pub struct Updater {
    client: reqwest::Client,
    github_repo: String,
    current_version: String,
}

impl Updater {
    /// Create a new updater instance
    pub fn new(current_version: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            github_repo: "misyaguziya/VRCT".to_string(),
            current_version: current_version.into(),
        }
    }
    
    /// Create an updater with a custom GitHub repository
    pub fn with_repo(current_version: impl Into<String>, repo: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            github_repo: repo.into(),
            current_version: current_version.into(),
        }
    }
    
    /// Check for updates by querying the GitHub API
    /// Returns UpdateInfo with version comparison and download URL
    pub async fn check_for_updates(&self) -> Result<UpdateInfo> {
        info!("Checking for updates from GitHub repository: {}", self.github_repo);
        
        let url = format!(
            "https://api.github.com/repos/{}/releases/latest",
            self.github_repo
        );
        
        let response = self.client
            .get(&url)
            .header("User-Agent", format!("VRCT/{}", self.current_version))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch release info: {}", e);
                VrctError::Http(e)
            })?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("GitHub API returned error: {} - {}", status, error_text);
            return Err(VrctError::Unknown(format!(
                "GitHub API error: {} - {}",
                status, error_text
            )));
        }
        
        let release: GitHubRelease = response.json().await.map_err(|e| {
            error!("Failed to parse release info: {}", e);
            VrctError::Unknown(format!("Failed to parse release info: {}", e))
        })?;
        
        let latest_version = Self::normalize_version(&release.tag_name);
        let current_version = Self::normalize_version(&self.current_version);
        
        let comparison = Self::compare_versions(&current_version, &latest_version);
        let available = comparison == VersionComparison::Older;
        
        // Find the updater executable in the release assets
        let download_url = self.find_updater_asset(&release.assets);
        
        let update_info = UpdateInfo {
            available,
            current_version: self.current_version.clone(),
            latest_version: release.tag_name.clone(),
            release_notes: release.body,
            download_url,
            release_url: release.html_url,
        };
        
        if available {
            info!(
                "Update available: {} -> {}",
                self.current_version, release.tag_name
            );
        } else {
            info!("No update available. Current version: {}", self.current_version);
        }
        
        Ok(update_info)
    }
    
    /// Find the updater executable asset from the release assets
    fn find_updater_asset(&self, assets: &[GitHubAsset]) -> Option<String> {
        // Look for common updater executable names
        let updater_patterns = [
            "VRCT_Updater.exe",
            "updater.exe",
            "VRCT-updater.exe",
            "vrct_updater.exe",
        ];
        
        for pattern in &updater_patterns {
            if let Some(asset) = assets.iter().find(|a| {
                a.name.to_lowercase() == pattern.to_lowercase()
            }) {
                return Some(asset.browser_download_url.clone());
            }
        }
        
        // If no specific updater found, look for any .exe that contains "updater"
        assets.iter()
            .find(|a| {
                let name_lower = a.name.to_lowercase();
                name_lower.contains("updater") && name_lower.ends_with(".exe")
            })
            .map(|a| a.browser_download_url.clone())
    }
    
    /// Normalize version string by removing 'v' prefix and trimming whitespace
    fn normalize_version(version: &str) -> String {
        version
            .trim()
            .trim_start_matches('v')
            .trim_start_matches('V')
            .to_string()
    }
    
    /// Compare two version strings
    /// Returns VersionComparison indicating if current is newer, same, or older than latest
    pub fn compare_versions(current: &str, latest: &str) -> VersionComparison {
        let current_parts = Self::parse_version(current);
        let latest_parts = Self::parse_version(latest);
        
        for (c, l) in current_parts.iter().zip(latest_parts.iter()) {
            match c.cmp(l) {
                std::cmp::Ordering::Greater => return VersionComparison::Newer,
                std::cmp::Ordering::Less => return VersionComparison::Older,
                std::cmp::Ordering::Equal => continue,
            }
        }
        
        // If all compared parts are equal, check if one has more parts
        match current_parts.len().cmp(&latest_parts.len()) {
            std::cmp::Ordering::Greater => VersionComparison::Newer,
            std::cmp::Ordering::Less => VersionComparison::Older,
            std::cmp::Ordering::Equal => VersionComparison::Same,
        }
    }
    
    /// Parse version string into numeric parts
    fn parse_version(version: &str) -> Vec<u32> {
        version
            .split(|c: char| c == '.' || c == '-' || c == '_')
            .filter_map(|part| {
                // Extract numeric prefix from each part (e.g., "3beta" -> 3)
                let numeric: String = part.chars().take_while(|c| c.is_ascii_digit()).collect();
                numeric.parse().ok()
            })
            .collect()
    }
    
    /// Get the current version
    pub fn current_version(&self) -> &str {
        &self.current_version
    }
    
    /// Download the updater executable from the given URL
    /// Returns the path to the downloaded file
    /// 
    /// # Arguments
    /// * `download_url` - URL to download the updater from
    /// * `progress_callback` - Optional callback for download progress (0.0 to 1.0)
    pub async fn download_updater<F>(
        &self,
        download_url: &str,
        progress_callback: Option<F>,
    ) -> Result<std::path::PathBuf>
    where
        F: Fn(f32) + Send + 'static,
    {
        info!("Downloading updater from: {}", download_url);
        
        // Get the temp directory for storing the updater
        let temp_dir = std::env::temp_dir();
        let updater_path = temp_dir.join("VRCT_Updater.exe");
        
        // Start the download
        let response = self.client
            .get(download_url)
            .header("User-Agent", format!("VRCT/{}", self.current_version))
            .send()
            .await
            .map_err(|e| {
                error!("Failed to start download: {}", e);
                VrctError::Update(format!("Failed to start download: {}", e))
            })?;
        
        if !response.status().is_success() {
            let status = response.status();
            error!("Download failed with status: {}", status);
            return Err(VrctError::Update(format!(
                "Download failed with status: {}",
                status
            )));
        }
        
        // Get content length for progress reporting (used for logging)
        let _total_size = response.content_length().unwrap_or(0);
        
        // Download the file
        let bytes = response.bytes().await.map_err(|e| {
            error!("Failed to download updater: {}", e);
            VrctError::Update(format!("Failed to download updater: {}", e))
        })?;
        
        // Report progress (simplified - full download complete)
        if let Some(callback) = progress_callback {
            callback(1.0);
        }
        
        // Write to file
        std::fs::write(&updater_path, &bytes).map_err(|e| {
            error!("Failed to write updater to disk: {}", e);
            VrctError::Update(format!("Failed to write updater to disk: {}", e))
        })?;
        
        info!(
            "Updater downloaded successfully to: {} ({} bytes)",
            updater_path.display(),
            bytes.len()
        );
        
        Ok(updater_path)
    }
    
    /// Download the updater with streaming progress updates
    /// This version provides more granular progress updates during download
    pub async fn download_updater_with_progress<F>(
        &self,
        download_url: &str,
        mut progress_callback: F,
    ) -> Result<std::path::PathBuf>
    where
        F: FnMut(DownloadProgress) + Send + 'static,
    {
        use tokio::io::AsyncWriteExt;
        use futures_util::StreamExt;
        
        info!("Downloading updater from: {}", download_url);
        
        // Get the temp directory for storing the updater
        let temp_dir = std::env::temp_dir();
        let updater_path = temp_dir.join("VRCT_Updater.exe");
        
        // Start the download
        let response = self.client
            .get(download_url)
            .header("User-Agent", format!("VRCT/{}", self.current_version))
            .send()
            .await
            .map_err(|e| {
                error!("Failed to start download: {}", e);
                VrctError::Update(format!("Failed to start download: {}", e))
            })?;
        
        if !response.status().is_success() {
            let status = response.status();
            error!("Download failed with status: {}", status);
            return Err(VrctError::Update(format!(
                "Download failed with status: {}",
                status
            )));
        }
        
        // Get content length for progress reporting
        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;
        
        // Report initial progress
        progress_callback(DownloadProgress {
            downloaded_bytes: 0,
            total_bytes: total_size,
            percentage: 0.0,
            status: DownloadStatus::InProgress,
        });
        
        // Create the output file
        let mut file = tokio::fs::File::create(&updater_path).await.map_err(|e| {
            error!("Failed to create updater file: {}", e);
            VrctError::Update(format!("Failed to create updater file: {}", e))
        })?;
        
        // Stream the download
        let mut stream = response.bytes_stream();
        
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| {
                error!("Error downloading chunk: {}", e);
                VrctError::Update(format!("Error downloading chunk: {}", e))
            })?;
            
            file.write_all(&chunk).await.map_err(|e| {
                error!("Error writing chunk to file: {}", e);
                VrctError::Update(format!("Error writing chunk to file: {}", e))
            })?;
            
            downloaded += chunk.len() as u64;
            
            // Calculate and report progress
            let percentage = if total_size > 0 {
                (downloaded as f32 / total_size as f32) * 100.0
            } else {
                0.0
            };
            
            progress_callback(DownloadProgress {
                downloaded_bytes: downloaded,
                total_bytes: total_size,
                percentage,
                status: DownloadStatus::InProgress,
            });
        }
        
        // Ensure all data is written
        file.flush().await.map_err(|e| {
            error!("Error flushing file: {}", e);
            VrctError::Update(format!("Error flushing file: {}", e))
        })?;
        
        // Report completion
        progress_callback(DownloadProgress {
            downloaded_bytes: downloaded,
            total_bytes: total_size,
            percentage: 100.0,
            status: DownloadStatus::Completed,
        });
        
        info!(
            "Updater downloaded successfully to: {} ({} bytes)",
            updater_path.display(),
            downloaded
        );
        
        Ok(updater_path)
    }
    
    /// Launch the updater executable and prepare for application exit
    /// 
    /// # Arguments
    /// * `updater_path` - Path to the downloaded updater executable
    /// 
    /// # Returns
    /// * `Ok(())` if the updater was launched successfully
    /// * The caller should exit the application after this returns successfully
    pub fn launch_updater(updater_path: &std::path::Path) -> Result<()> {
        info!("Launching updater: {}", updater_path.display());
        
        if !updater_path.exists() {
            error!("Updater executable not found: {}", updater_path.display());
            return Err(VrctError::Update(format!(
                "Updater executable not found: {}",
                updater_path.display()
            )));
        }
        
        // Launch the updater as a detached process
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            
            // CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
            const DETACHED_PROCESS: u32 = 0x00000008;
            
            std::process::Command::new(updater_path)
                .creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS)
                .spawn()
                .map_err(|e| {
                    error!("Failed to launch updater: {}", e);
                    VrctError::Update(format!("Failed to launch updater: {}", e))
                })?;
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            std::process::Command::new(updater_path)
                .spawn()
                .map_err(|e| {
                    error!("Failed to launch updater: {}", e);
                    VrctError::Update(format!("Failed to launch updater: {}", e))
                })?;
        }
        
        info!("Updater launched successfully");
        Ok(())
    }
    
    /// Download and launch the updater in one operation
    /// 
    /// # Arguments
    /// * `download_url` - URL to download the updater from
    /// * `progress_callback` - Callback for download progress
    /// 
    /// # Returns
    /// * `Ok(())` if successful - the caller should exit the application
    pub async fn download_and_launch<F>(
        &self,
        download_url: &str,
        progress_callback: F,
    ) -> Result<()>
    where
        F: FnMut(DownloadProgress) + Send + 'static,
    {
        let updater_path = self.download_updater_with_progress(download_url, progress_callback).await?;
        Self::launch_updater(&updater_path)?;
        Ok(())
    }
}

/// Download progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percentage: f32,
    pub status: DownloadStatus,
}

/// Download status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    InProgress,
    Completed,
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_normalize_version() {
        assert_eq!(Updater::normalize_version("v1.2.3"), "1.2.3");
        assert_eq!(Updater::normalize_version("V1.2.3"), "1.2.3");
        assert_eq!(Updater::normalize_version("1.2.3"), "1.2.3");
        assert_eq!(Updater::normalize_version(" v1.2.3 "), "1.2.3");
    }
    
    #[test]
    fn test_parse_version() {
        assert_eq!(Updater::parse_version("1.2.3"), vec![1, 2, 3]);
        assert_eq!(Updater::parse_version("3.3.2"), vec![3, 3, 2]);
        assert_eq!(Updater::parse_version("1.0.0-beta"), vec![1, 0, 0]);
        assert_eq!(Updater::parse_version("2.1"), vec![2, 1]);
    }
    
    #[test]
    fn test_compare_versions() {
        // Same versions
        assert_eq!(
            Updater::compare_versions("1.2.3", "1.2.3"),
            VersionComparison::Same
        );
        
        // Current is older
        assert_eq!(
            Updater::compare_versions("1.2.3", "1.2.4"),
            VersionComparison::Older
        );
        assert_eq!(
            Updater::compare_versions("1.2.3", "1.3.0"),
            VersionComparison::Older
        );
        assert_eq!(
            Updater::compare_versions("1.2.3", "2.0.0"),
            VersionComparison::Older
        );
        
        // Current is newer
        assert_eq!(
            Updater::compare_versions("1.2.4", "1.2.3"),
            VersionComparison::Newer
        );
        assert_eq!(
            Updater::compare_versions("1.3.0", "1.2.3"),
            VersionComparison::Newer
        );
        assert_eq!(
            Updater::compare_versions("2.0.0", "1.2.3"),
            VersionComparison::Newer
        );
        
        // Different number of parts
        assert_eq!(
            Updater::compare_versions("1.2", "1.2.1"),
            VersionComparison::Older
        );
        assert_eq!(
            Updater::compare_versions("1.2.1", "1.2"),
            VersionComparison::Newer
        );
    }
    
    #[test]
    fn test_updater_creation() {
        let updater = Updater::new("3.3.2");
        assert_eq!(updater.current_version(), "3.3.2");
        
        let updater_custom = Updater::with_repo("1.0.0", "owner/repo");
        assert_eq!(updater_custom.current_version(), "1.0.0");
    }
}
