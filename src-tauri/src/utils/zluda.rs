// ZLUDA detection and initialization module
// Provides functionality to detect, initialize, and manage ZLUDA for AMD GPU acceleration

use crate::utils::error::{Result, VrctError};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{debug, error, info, warn};

#[cfg(target_os = "windows")]
use std::process::Command;

/// Global flag indicating if ZLUDA fallback to CPU has occurred
static ZLUDA_FALLBACK_OCCURRED: AtomicBool = AtomicBool::new(false);

/// Result of a ZLUDA operation with fallback information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZludaOperationResult<T> {
    /// The result value
    pub value: T,
    /// Whether fallback to CPU occurred
    pub fallback_occurred: bool,
    /// Reason for fallback (if any)
    pub fallback_reason: Option<String>,
    /// Original device that was attempted
    pub original_device: Option<String>,
}

impl<T> ZludaOperationResult<T> {
    /// Create a successful result without fallback
    pub fn success(value: T) -> Self {
        Self {
            value,
            fallback_occurred: false,
            fallback_reason: None,
            original_device: None,
        }
    }

    /// Create a result with fallback
    pub fn with_fallback(value: T, reason: String, original_device: String) -> Self {
        Self {
            value,
            fallback_occurred: true,
            fallback_reason: Some(reason),
            original_device: Some(original_device),
        }
    }
}

/// Notification callback type for ZLUDA fallback events
pub type FallbackNotificationCallback = Box<dyn Fn(&str, &str) + Send + Sync>;

/// Information about a detected ZLUDA device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZludaDevice {
    /// Device index
    pub index: usize,
    /// Device name (e.g., "AMD Radeon RX 6800")
    pub name: String,
    /// ZLUDA installation path
    pub zluda_path: String,
    /// Whether the device is available for use
    pub available: bool,
}

/// ZLUDA detection and management
pub struct ZludaManager {
    /// Path to ZLUDA installation
    zluda_path: Option<PathBuf>,
    /// Whether ZLUDA has been initialized
    initialized: bool,
    /// Detected ZLUDA devices
    devices: Vec<ZludaDevice>,
}

impl ZludaManager {
    /// Create a new ZLUDA manager
    pub fn new() -> Self {
        Self {
            zluda_path: None,
            initialized: false,
            devices: Vec::new(),
        }
    }

    /// Detect ZLUDA installation and enumerate devices
    /// 
    /// This function checks multiple locations for ZLUDA installation:
    /// 1. ZLUDA_PATH environment variable
    /// 2. Standard installation paths relative to the application
    /// 3. System PATH for ZLUDA binaries
    pub fn detect(&mut self) -> Result<bool> {
        info!("Detecting ZLUDA installation...");

        // Try to find ZLUDA installation
        self.zluda_path = self.find_zluda_path();

        if let Some(ref path) = self.zluda_path {
            info!("ZLUDA detected at: {:?}", path);
            
            // Enumerate AMD GPUs
            self.enumerate_devices()?;
            
            Ok(true)
        } else {
            info!("ZLUDA not detected");
            Ok(false)
        }
    }

    /// Find ZLUDA installation path
    fn find_zluda_path(&self) -> Option<PathBuf> {
        // Check 1: ZLUDA_PATH environment variable
        if let Ok(env_path) = env::var("ZLUDA_PATH") {
            let path = PathBuf::from(&env_path);
            if self.is_valid_zluda_path(&path) {
                debug!("ZLUDA found via ZLUDA_PATH environment variable: {:?}", path);
                return Some(path);
            }
        }

        // Check 2: Standard installation paths relative to executable
        if let Ok(exe_path) = env::current_exe() {
            if let Some(app_root) = exe_path.parent().and_then(|p| p.parent()) {
                let standard_paths = [
                    app_root.join(".venv_zluda"),
                    app_root.join(".venv_cuda").join("zluda"),
                    app_root.join("zluda"),
                ];

                for path in &standard_paths {
                    if self.is_valid_zluda_path(path) {
                        debug!("ZLUDA found at standard path: {:?}", path);
                        return Some(path.clone());
                    }
                }
            }
        }

        // Check 3: System PATH for ZLUDA binaries
        if let Some(path) = self.find_in_system_path() {
            debug!("ZLUDA found in system PATH: {:?}", path);
            return Some(path);
        }

        None
    }

    /// Check if a path contains a valid ZLUDA installation
    fn is_valid_zluda_path(&self, path: &Path) -> bool {
        if !path.is_dir() {
            return false;
        }

        // Check for key ZLUDA files based on platform
        #[cfg(target_os = "windows")]
        let required_files = ["nvcuda.dll", "cublas.dll", "cudart.dll", "nvml.dll"];
        
        #[cfg(not(target_os = "windows"))]
        let required_files = ["libcuda.so", "libcublas.so", "libcudart.so"];

        // Check root directory and common subdirectories
        let subdirs = ["", "bin", "lib", "lib64"];
        
        for subdir in &subdirs {
            let check_path = if subdir.is_empty() {
                path.to_path_buf()
            } else {
                path.join(subdir)
            };

            if !check_path.exists() {
                continue;
            }

            for file in &required_files {
                if check_path.join(file).exists() {
                    debug!("Found ZLUDA file: {:?}", check_path.join(file));
                    return true;
                }
            }
        }

        // Fallback: check for any DLL/SO files
        #[cfg(target_os = "windows")]
        let extension = "dll";
        #[cfg(not(target_os = "windows"))]
        let extension = "so";

        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == extension {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Search for ZLUDA in system PATH
    fn find_in_system_path(&self) -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        let zluda_binary = "nvcuda.dll";
        #[cfg(not(target_os = "windows"))]
        let zluda_binary = "libcuda.so";

        let path_env = env::var("PATH").ok()?;
        
        #[cfg(target_os = "windows")]
        let separator = ';';
        #[cfg(not(target_os = "windows"))]
        let separator = ':';

        for path in path_env.split(separator) {
            if path.is_empty() {
                continue;
            }

            let file_path = PathBuf::from(path).join(zluda_binary);
            if file_path.exists() {
                return Some(PathBuf::from(path));
            }
        }

        None
    }

    /// Enumerate AMD GPU devices
    fn enumerate_devices(&mut self) -> Result<()> {
        self.devices.clear();

        let amd_gpus = detect_amd_gpus()?;
        
        if let Some(ref zluda_path) = self.zluda_path {
            for (index, gpu_name) in amd_gpus.into_iter().enumerate() {
                self.devices.push(ZludaDevice {
                    index,
                    name: gpu_name,
                    zluda_path: zluda_path.to_string_lossy().to_string(),
                    available: true,
                });
            }
        }

        info!("Enumerated {} ZLUDA device(s)", self.devices.len());
        Ok(())
    }

    /// Initialize ZLUDA by setting environment variables
    pub fn initialize(&mut self) -> Result<bool> {
        if self.initialized {
            return Ok(true);
        }

        let zluda_path = match &self.zluda_path {
            Some(path) => path.clone(),
            None => {
                warn!("Cannot initialize ZLUDA: no installation path found");
                return Ok(false);
            }
        };

        if !self.is_valid_zluda_path(&zluda_path) {
            return Err(VrctError::ZludaRuntime(format!(
                "Invalid ZLUDA installation path: {:?}",
                zluda_path
            )));
        }

        info!("Initializing ZLUDA from {:?}", zluda_path);

        // Set CUDA_PATH to ZLUDA installation
        env::set_var("CUDA_PATH", &zluda_path);
        debug!("Set CUDA_PATH to {:?}", zluda_path);

        // Add ZLUDA binaries to library path
        let mut bin_dirs = vec![zluda_path.clone()];
        
        for subdir in &["bin", "lib", "lib64"] {
            let subdir_path = zluda_path.join(subdir);
            if subdir_path.exists() {
                bin_dirs.push(subdir_path);
            }
        }

        #[cfg(target_os = "windows")]
        {
            let current_path = env::var("PATH").unwrap_or_default();
            let new_paths: Vec<String> = bin_dirs
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .filter(|p| !current_path.contains(p))
                .collect();

            if !new_paths.is_empty() {
                let new_path = format!("{};{}", new_paths.join(";"), current_path);
                env::set_var("PATH", &new_path);
                debug!("Added ZLUDA directories to PATH: {:?}", new_paths);
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            let current_ld_path = env::var("LD_LIBRARY_PATH").unwrap_or_default();
            let new_paths: Vec<String> = bin_dirs
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .filter(|p| !current_ld_path.contains(p))
                .collect();

            if !new_paths.is_empty() {
                let new_ld_path = if current_ld_path.is_empty() {
                    new_paths.join(":")
                } else {
                    format!("{}:{}", new_paths.join(":"), current_ld_path)
                };
                env::set_var("LD_LIBRARY_PATH", &new_ld_path);
                debug!("Added ZLUDA directories to LD_LIBRARY_PATH: {:?}", new_paths);
            }
        }

        self.initialized = true;
        info!("ZLUDA initialization completed successfully");
        Ok(true)
    }

    /// Get the ZLUDA installation path
    pub fn get_path(&self) -> Option<&Path> {
        self.zluda_path.as_deref()
    }

    /// Get detected ZLUDA devices
    pub fn get_devices(&self) -> &[ZludaDevice] {
        &self.devices
    }

    /// Check if ZLUDA is available
    pub fn is_available(&self) -> bool {
        self.zluda_path.is_some() && !self.devices.is_empty()
    }

    /// Check if ZLUDA has been initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Set ZLUDA path from configuration
    pub fn set_path(&mut self, path: Option<String>) {
        if let Some(p) = path {
            let path_buf = PathBuf::from(&p);
            if self.is_valid_zluda_path(&path_buf) {
                self.zluda_path = Some(path_buf);
                info!("ZLUDA path set from configuration: {}", p);
            } else {
                warn!("Configured ZLUDA path is invalid: {}", p);
            }
        }
    }
}

impl Default for ZludaManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ZLUDA Fallback Logic
// ============================================================================

/// Fallback handler for ZLUDA operations
/// Provides utilities for attempting operations on ZLUDA and falling back to CPU
pub struct ZludaFallbackHandler {
    /// ZLUDA manager instance
    manager: ZludaManager,
    /// Callback for notifying about fallback events
    notification_callback: Option<FallbackNotificationCallback>,
}

impl ZludaFallbackHandler {
    /// Create a new fallback handler
    pub fn new() -> Self {
        Self {
            manager: ZludaManager::new(),
            notification_callback: None,
        }
    }

    /// Create a new fallback handler with a ZLUDA manager
    pub fn with_manager(manager: ZludaManager) -> Self {
        Self {
            manager,
            notification_callback: None,
        }
    }

    /// Set the notification callback for fallback events
    pub fn set_notification_callback<F>(&mut self, callback: F)
    where
        F: Fn(&str, &str) + Send + Sync + 'static,
    {
        self.notification_callback = Some(Box::new(callback));
    }

    /// Get a reference to the ZLUDA manager
    pub fn manager(&self) -> &ZludaManager {
        &self.manager
    }

    /// Get a mutable reference to the ZLUDA manager
    pub fn manager_mut(&mut self) -> &mut ZludaManager {
        &mut self.manager
    }

    /// Attempt an operation on ZLUDA device, falling back to CPU on failure
    /// 
    /// This function will:
    /// 1. Check if ZLUDA is available
    /// 2. If available, attempt the operation on ZLUDA
    /// 3. If ZLUDA fails, fall back to CPU and notify the user
    /// 4. If ZLUDA is not available, run on CPU directly
    /// 
    /// # Arguments
    /// * `zluda_op` - Operation to attempt on ZLUDA device
    /// * `cpu_op` - Fallback operation to run on CPU
    /// * `operation_name` - Name of the operation for logging/notification
    /// 
    /// # Returns
    /// Result containing the operation result and fallback information
    pub fn try_with_fallback<T, F, G>(
        &self,
        zluda_op: F,
        cpu_op: G,
        operation_name: &str,
    ) -> Result<ZludaOperationResult<T>>
    where
        F: FnOnce() -> Result<T>,
        G: FnOnce() -> Result<T>,
    {
        // Check if ZLUDA is available
        if !self.manager.is_available() {
            info!("{}: ZLUDA not available, using CPU", operation_name);
            let result = cpu_op()?;
            return Ok(ZludaOperationResult::success(result));
        }

        // Attempt ZLUDA operation
        info!("{}: Attempting operation on ZLUDA device", operation_name);
        match zluda_op() {
            Ok(result) => {
                info!("{}: ZLUDA operation successful", operation_name);
                Ok(ZludaOperationResult::success(result))
            }
            Err(e) => {
                // Check if this is a ZLUDA-specific error
                let error_msg = format!("{}", e);
                let is_zluda_error = matches!(e, VrctError::ZludaRuntime(_))
                    || error_msg.contains("ZLUDA")
                    || error_msg.contains("zluda")
                    || error_msg.contains("HIP")
                    || error_msg.contains("hip")
                    || error_msg.contains("out of memory")
                    || error_msg.contains("CUDA")
                    || error_msg.contains("cuda");

                if is_zluda_error {
                    warn!(
                        "{}: ZLUDA operation failed ({}), falling back to CPU",
                        operation_name, error_msg
                    );

                    // Set global fallback flag
                    ZLUDA_FALLBACK_OCCURRED.store(true, Ordering::SeqCst);

                    // Notify about fallback
                    self.notify_fallback(operation_name, &error_msg);

                    // Attempt CPU fallback
                    match cpu_op() {
                        Ok(result) => {
                            info!("{}: CPU fallback successful", operation_name);
                            Ok(ZludaOperationResult::with_fallback(
                                result,
                                error_msg,
                                "ZLUDA".to_string(),
                            ))
                        }
                        Err(cpu_err) => {
                            error!(
                                "{}: CPU fallback also failed: {}",
                                operation_name, cpu_err
                            );
                            Err(cpu_err)
                        }
                    }
                } else {
                    // Not a ZLUDA error, propagate it
                    Err(e)
                }
            }
        }
    }

    /// Async version of try_with_fallback
    pub async fn try_with_fallback_async<T, F, G, Fut1, Fut2>(
        &self,
        zluda_op: F,
        cpu_op: G,
        operation_name: &str,
    ) -> Result<ZludaOperationResult<T>>
    where
        F: FnOnce() -> Fut1,
        G: FnOnce() -> Fut2,
        Fut1: std::future::Future<Output = Result<T>>,
        Fut2: std::future::Future<Output = Result<T>>,
    {
        // Check if ZLUDA is available
        if !self.manager.is_available() {
            info!("{}: ZLUDA not available, using CPU", operation_name);
            let result = cpu_op().await?;
            return Ok(ZludaOperationResult::success(result));
        }

        // Attempt ZLUDA operation
        info!("{}: Attempting operation on ZLUDA device", operation_name);
        match zluda_op().await {
            Ok(result) => {
                info!("{}: ZLUDA operation successful", operation_name);
                Ok(ZludaOperationResult::success(result))
            }
            Err(e) => {
                // Check if this is a ZLUDA-specific error
                let error_msg = format!("{}", e);
                let is_zluda_error = matches!(e, VrctError::ZludaRuntime(_))
                    || error_msg.contains("ZLUDA")
                    || error_msg.contains("zluda")
                    || error_msg.contains("HIP")
                    || error_msg.contains("hip")
                    || error_msg.contains("out of memory")
                    || error_msg.contains("CUDA")
                    || error_msg.contains("cuda");

                if is_zluda_error {
                    warn!(
                        "{}: ZLUDA operation failed ({}), falling back to CPU",
                        operation_name, error_msg
                    );

                    // Set global fallback flag
                    ZLUDA_FALLBACK_OCCURRED.store(true, Ordering::SeqCst);

                    // Notify about fallback
                    self.notify_fallback(operation_name, &error_msg);

                    // Attempt CPU fallback
                    match cpu_op().await {
                        Ok(result) => {
                            info!("{}: CPU fallback successful", operation_name);
                            Ok(ZludaOperationResult::with_fallback(
                                result,
                                error_msg,
                                "ZLUDA".to_string(),
                            ))
                        }
                        Err(cpu_err) => {
                            error!(
                                "{}: CPU fallback also failed: {}",
                                operation_name, cpu_err
                            );
                            Err(cpu_err)
                        }
                    }
                } else {
                    // Not a ZLUDA error, propagate it
                    Err(e)
                }
            }
        }
    }

    /// Notify about a fallback event
    fn notify_fallback(&self, operation_name: &str, error_msg: &str) {
        if let Some(ref callback) = self.notification_callback {
            callback(operation_name, error_msg);
        }
    }
}

impl Default for ZludaFallbackHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if ZLUDA fallback has occurred during this session
pub fn has_zluda_fallback_occurred() -> bool {
    ZLUDA_FALLBACK_OCCURRED.load(Ordering::SeqCst)
}

/// Reset the ZLUDA fallback flag
pub fn reset_zluda_fallback_flag() {
    ZLUDA_FALLBACK_OCCURRED.store(false, Ordering::SeqCst);
}

/// Attempt to load a model with ZLUDA fallback support
/// 
/// This is a convenience function for model loading operations that should
/// attempt ZLUDA first and fall back to CPU on failure.
/// 
/// # Arguments
/// * `zluda_device_index` - ZLUDA device index to attempt
/// * `load_on_zluda` - Function to load model on ZLUDA
/// * `load_on_cpu` - Function to load model on CPU
/// * `model_name` - Name of the model for logging
/// 
/// # Returns
/// Tuple of (loaded model, fallback occurred, fallback reason)
pub fn load_model_with_fallback<T, F, G>(
    zluda_device_index: Option<usize>,
    load_on_zluda: F,
    load_on_cpu: G,
    model_name: &str,
) -> Result<ZludaOperationResult<T>>
where
    F: FnOnce(usize) -> Result<T>,
    G: FnOnce() -> Result<T>,
{
    match zluda_device_index {
        Some(index) => {
            info!("Attempting to load {} on ZLUDA device {}", model_name, index);
            
            match load_on_zluda(index) {
                Ok(model) => {
                    info!("{} loaded successfully on ZLUDA device {}", model_name, index);
                    Ok(ZludaOperationResult::success(model))
                }
                Err(e) => {
                    let error_msg = format!("{}", e);
                    warn!(
                        "Failed to load {} on ZLUDA device {}: {}. Falling back to CPU.",
                        model_name, index, error_msg
                    );
                    
                    // Set global fallback flag
                    ZLUDA_FALLBACK_OCCURRED.store(true, Ordering::SeqCst);
                    
                    // Attempt CPU fallback
                    let model = load_on_cpu()?;
                    info!("{} loaded successfully on CPU (fallback)", model_name);
                    
                    Ok(ZludaOperationResult::with_fallback(
                        model,
                        error_msg,
                        format!("ZLUDA device {}", index),
                    ))
                }
            }
        }
        None => {
            info!("Loading {} on CPU (no ZLUDA device specified)", model_name);
            let model = load_on_cpu()?;
            Ok(ZludaOperationResult::success(model))
        }
    }
}

/// Async version of load_model_with_fallback
pub async fn load_model_with_fallback_async<T, F, G, Fut1, Fut2>(
    zluda_device_index: Option<usize>,
    load_on_zluda: F,
    load_on_cpu: G,
    model_name: &str,
) -> Result<ZludaOperationResult<T>>
where
    F: FnOnce(usize) -> Fut1,
    G: FnOnce() -> Fut2,
    Fut1: std::future::Future<Output = Result<T>>,
    Fut2: std::future::Future<Output = Result<T>>,
{
    match zluda_device_index {
        Some(index) => {
            info!("Attempting to load {} on ZLUDA device {}", model_name, index);
            
            match load_on_zluda(index).await {
                Ok(model) => {
                    info!("{} loaded successfully on ZLUDA device {}", model_name, index);
                    Ok(ZludaOperationResult::success(model))
                }
                Err(e) => {
                    let error_msg = format!("{}", e);
                    warn!(
                        "Failed to load {} on ZLUDA device {}: {}. Falling back to CPU.",
                        model_name, index, error_msg
                    );
                    
                    // Set global fallback flag
                    ZLUDA_FALLBACK_OCCURRED.store(true, Ordering::SeqCst);
                    
                    // Attempt CPU fallback
                    let model = load_on_cpu().await?;
                    info!("{} loaded successfully on CPU (fallback)", model_name);
                    
                    Ok(ZludaOperationResult::with_fallback(
                        model,
                        error_msg,
                        format!("ZLUDA device {}", index),
                    ))
                }
            }
        }
        None => {
            info!("Loading {} on CPU (no ZLUDA device specified)", model_name);
            let model = load_on_cpu().await?;
            Ok(ZludaOperationResult::success(model))
        }
    }
}

/// Detect AMD GPUs in the system
#[cfg(target_os = "windows")]
pub fn detect_amd_gpus() -> Result<Vec<String>> {
    let mut gpus = Vec::new();

    // Use wmic to query video controllers
    let output = Command::new("wmic")
        .args(["path", "win32_VideoController", "get", "name"])
        .output()
        .map_err(|e| VrctError::ZludaRuntime(format!("Failed to run wmic: {}", e)))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        for line in stdout.lines() {
            let line_lower = line.to_lowercase();
            if line_lower.contains("amd") || line_lower.contains("radeon") || line_lower.contains("ati") {
                let gpu_name = line.trim().to_string();
                if !gpu_name.is_empty() && gpu_name.to_lowercase() != "name" {
                    gpus.push(gpu_name);
                }
            }
        }
    }

    if gpus.is_empty() {
        debug!("No AMD GPUs detected via wmic");
    } else {
        info!("Detected AMD GPUs: {:?}", gpus);
    }

    Ok(gpus)
}

/// Detect AMD GPUs in the system (Linux)
#[cfg(target_os = "linux")]
pub fn detect_amd_gpus() -> Result<Vec<String>> {
    use std::process::Command;
    
    let mut gpus = Vec::new();

    // Use lspci to check for AMD graphics
    let output = Command::new("lspci")
        .output()
        .map_err(|e| VrctError::ZludaRuntime(format!("Failed to run lspci: {}", e)))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        for line in stdout.lines() {
            let line_lower = line.to_lowercase();
            if (line_lower.contains("vga") || line_lower.contains("display") || line_lower.contains("3d"))
                && (line_lower.contains("amd") || line_lower.contains("ati") || line_lower.contains("radeon"))
            {
                // Extract GPU name from lspci output
                if let Some(name_start) = line.find(':') {
                    let gpu_name = line[name_start + 1..].trim().to_string();
                    if !gpu_name.is_empty() {
                        gpus.push(gpu_name);
                    }
                }
            }
        }
    }

    if gpus.is_empty() {
        debug!("No AMD GPUs detected via lspci");
    } else {
        info!("Detected AMD GPUs: {:?}", gpus);
    }

    Ok(gpus)
}

/// Detect AMD GPUs (fallback for other platforms)
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn detect_amd_gpus() -> Result<Vec<String>> {
    warn!("AMD GPU detection not supported on this platform");
    Ok(Vec::new())
}

/// Check if ZLUDA is installed (convenience function)
pub fn is_zluda_installed() -> bool {
    let mut manager = ZludaManager::new();
    manager.detect().unwrap_or(false)
}

/// Get ZLUDA path if installed (convenience function)
pub fn get_zluda_path() -> Option<PathBuf> {
    let mut manager = ZludaManager::new();
    if manager.detect().unwrap_or(false) {
        manager.zluda_path
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zluda_manager_creation() {
        let manager = ZludaManager::new();
        assert!(!manager.is_initialized());
        assert!(!manager.is_available());
        assert!(manager.get_devices().is_empty());
    }

    #[test]
    fn test_zluda_detection_no_crash() {
        // This test ensures detection doesn't crash even if ZLUDA isn't installed
        let mut manager = ZludaManager::new();
        let result = manager.detect();
        assert!(result.is_ok());
    }

    #[test]
    fn test_amd_gpu_detection_no_crash() {
        // This test ensures AMD GPU detection doesn't crash
        let result = detect_amd_gpus();
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_valid_zluda_path_nonexistent() {
        let manager = ZludaManager::new();
        assert!(!manager.is_valid_zluda_path(Path::new("/nonexistent/path")));
    }

    #[test]
    fn test_set_path_invalid() {
        let mut manager = ZludaManager::new();
        manager.set_path(Some("/nonexistent/path".to_string()));
        assert!(manager.zluda_path.is_none());
    }

    // ========================================================================
    // Fallback Logic Tests
    // ========================================================================

    #[test]
    fn test_zluda_operation_result_success() {
        let result: ZludaOperationResult<i32> = ZludaOperationResult::success(42);
        assert_eq!(result.value, 42);
        assert!(!result.fallback_occurred);
        assert!(result.fallback_reason.is_none());
        assert!(result.original_device.is_none());
    }

    #[test]
    fn test_zluda_operation_result_with_fallback() {
        let result: ZludaOperationResult<i32> = ZludaOperationResult::with_fallback(
            42,
            "ZLUDA error".to_string(),
            "ZLUDA device 0".to_string(),
        );
        assert_eq!(result.value, 42);
        assert!(result.fallback_occurred);
        assert_eq!(result.fallback_reason, Some("ZLUDA error".to_string()));
        assert_eq!(result.original_device, Some("ZLUDA device 0".to_string()));
    }

    #[test]
    fn test_fallback_handler_creation() {
        let handler = ZludaFallbackHandler::new();
        assert!(!handler.manager().is_available());
    }

    #[test]
    fn test_fallback_handler_with_manager() {
        let manager = ZludaManager::new();
        let handler = ZludaFallbackHandler::with_manager(manager);
        assert!(!handler.manager().is_available());
    }

    #[test]
    fn test_try_with_fallback_no_zluda() {
        // When ZLUDA is not available, should use CPU directly
        let handler = ZludaFallbackHandler::new();
        
        let result = handler.try_with_fallback(
            || Ok(1), // ZLUDA op (won't be called)
            || Ok(2), // CPU op
            "test_operation",
        );
        
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.value, 2); // Should use CPU result
        assert!(!result.fallback_occurred);
    }

    #[test]
    fn test_load_model_with_fallback_no_device() {
        // When no ZLUDA device is specified, should use CPU directly
        let result = load_model_with_fallback(
            None,
            |_| Ok("zluda_model"),
            || Ok("cpu_model"),
            "test_model",
        );
        
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.value, "cpu_model");
        assert!(!result.fallback_occurred);
    }

    #[test]
    fn test_load_model_with_fallback_zluda_success() {
        // When ZLUDA succeeds, should return ZLUDA result
        let result = load_model_with_fallback(
            Some(0),
            |_| Ok("zluda_model"),
            || Ok("cpu_model"),
            "test_model",
        );
        
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.value, "zluda_model");
        assert!(!result.fallback_occurred);
    }

    #[test]
    fn test_load_model_with_fallback_zluda_failure() {
        // Reset fallback flag before test
        reset_zluda_fallback_flag();
        
        // When ZLUDA fails, should fall back to CPU
        let result = load_model_with_fallback(
            Some(0),
            |_| Err(VrctError::ZludaRuntime("ZLUDA error".to_string())),
            || Ok("cpu_model"),
            "test_model",
        );
        
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.value, "cpu_model");
        assert!(result.fallback_occurred);
        assert!(result.fallback_reason.is_some());
        assert!(has_zluda_fallback_occurred());
        
        // Reset for other tests
        reset_zluda_fallback_flag();
    }

    #[test]
    fn test_load_model_with_fallback_both_fail() {
        // When both ZLUDA and CPU fail, should return error
        let result: Result<ZludaOperationResult<&str>> = load_model_with_fallback(
            Some(0),
            |_| Err(VrctError::ZludaRuntime("ZLUDA error".to_string())),
            || Err(VrctError::ModelLoading("CPU error".to_string())),
            "test_model",
        );
        
        assert!(result.is_err());
    }

    #[test]
    fn test_fallback_flag_operations() {
        // Test flag operations
        reset_zluda_fallback_flag();
        assert!(!has_zluda_fallback_occurred());
        
        ZLUDA_FALLBACK_OCCURRED.store(true, Ordering::SeqCst);
        assert!(has_zluda_fallback_occurred());
        
        reset_zluda_fallback_flag();
        assert!(!has_zluda_fallback_occurred());
    }

    #[test]
    fn test_fallback_handler_notification_callback() {
        use std::sync::atomic::AtomicUsize;
        use std::sync::Arc;
        
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();
        
        let mut handler = ZludaFallbackHandler::new();
        handler.set_notification_callback(move |_op, _err| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        });
        
        // Notification callback is called during fallback
        // Since ZLUDA is not available, no fallback will occur
        let _ = handler.try_with_fallback(
            || Ok(1),
            || Ok(2),
            "test",
        );
        
        // No fallback occurred since ZLUDA wasn't available
        assert_eq!(call_count.load(Ordering::SeqCst), 0);
    }
}
