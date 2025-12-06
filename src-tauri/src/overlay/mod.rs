// VR overlay subsystem module
// This module handles VR overlay rendering using OpenVR

pub mod renderer;

use crate::utils::error::{Result, VrctError};
use renderer::TextRenderer;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::warn;

#[cfg(feature = "vr-overlay")]
use tracing::info;

#[cfg(feature = "vr-overlay")]
use openvr::{Context, Overlay, System};

#[cfg(feature = "vr-overlay")]
use tracing::error;

/// Tracker types for overlay positioning
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tracker {
    HMD,
    LeftHand,
    RightHand,
}

#[cfg(feature = "vr-overlay")]
impl Tracker {
    fn to_tracking_device_index(&self, system: &System) -> u32 {
        match self {
            Tracker::HMD => openvr::tracked_device_index::HMD,
            Tracker::LeftHand => {
                // Find left hand controller
                system
                    .tracked_device_index_for_controller_role(
                        openvr::TrackedControllerRole::LeftHand,
                    )
                    .unwrap_or(openvr::tracked_device_index::HMD)
            }
            Tracker::RightHand => {
                // Find right hand controller
                system
                    .tracked_device_index_for_controller_role(
                        openvr::TrackedControllerRole::RightHand,
                    )
                    .unwrap_or(openvr::tracked_device_index::HMD)
            }
        }
    }
}

/// 3D vector for position
#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Transform for overlay positioning
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: f32,
}

/// Configuration for an overlay
#[derive(Debug, Clone)]
pub struct OverlayConfig {
    pub tracker: Tracker,
    pub position: Vec3,
    pub rotation: Vec3,
    pub opacity: f32,
    pub ui_scaling: f32,
    pub display_duration: u32,
    pub fadeout_duration: u32,
    pub width: u32,
    pub height: u32,
    pub font_size: f32,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            tracker: Tracker::HMD,
            position: Vec3 {
                x: 0.0,
                y: 0.0,
                z: -1.0,
            },
            rotation: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            opacity: 1.0,
            ui_scaling: 1.0,
            display_duration: 5000,
            fadeout_duration: 1000,
            width: 512,
            height: 256,
            font_size: 24.0,
        }
    }
}

/// Individual overlay instance
#[cfg(feature = "vr-overlay")]
struct OverlayInstance {
    handle: Overlay,
    config: OverlayConfig,
}

#[cfg(not(feature = "vr-overlay"))]
#[allow(dead_code)]
struct OverlayInstance {
    config: OverlayConfig,
}

/// Manager for VR overlays
pub struct OverlayManager {
    #[cfg(feature = "vr-overlay")]
    context: Option<Arc<Context>>,
    #[cfg(feature = "vr-overlay")]
    system: Option<System>,
    #[allow(dead_code)]
    overlays: Arc<Mutex<HashMap<String, OverlayInstance>>>,
    #[allow(dead_code)]
    renderer: Arc<TextRenderer>,
}

#[cfg(feature = "vr-overlay")]
impl OverlayManager {
    /// Create a new overlay manager
    /// This will attempt to initialize OpenVR, but will not fail if VR is not available
    pub fn new() -> Result<Self> {
        info!("Initializing overlay manager with OpenVR support");

        // Try to initialize OpenVR
        let (context, system) = match Self::init_openvr() {
            Ok((ctx, sys)) => {
                info!("OpenVR initialized successfully");
                (Some(Arc::new(ctx)), Some(sys))
            }
            Err(e) => {
                warn!("OpenVR initialization failed: {}. Overlay functionality will be disabled.", e);
                (None, None)
            }
        };

        let renderer = Arc::new(TextRenderer::new()?);

        Ok(Self {
            context,
            system,
            overlays: Arc::new(Mutex::new(HashMap::new())),
            renderer,
        })
    }

    /// Initialize OpenVR system
    fn init_openvr() -> Result<(Context, System)> {
        let context = unsafe {
            openvr::init(openvr::ApplicationType::Overlay).map_err(|e| {
                VrctError::Overlay(format!("Failed to initialize OpenVR: {:?}", e))
            })?
        };

        let system = context.system().map_err(|e| {
            VrctError::Overlay(format!("Failed to get OpenVR system: {:?}", e))
        })?;

        Ok((context, system))
    }

    /// Check if OpenVR is available
    pub fn is_available(&self) -> bool {
        self.context.is_some() && self.system.is_some()
    }

    /// Create a new overlay with the given name and configuration
    pub fn create_overlay(&self, name: &str, config: OverlayConfig) -> Result<()> {
        if !self.is_available() {
            return Err(VrctError::Overlay(
                "OpenVR not available, cannot create overlay".to_string(),
            ));
        }

        let context = self.context.as_ref().unwrap();

        info!("Creating overlay: {}", name);

        // Create overlay handle
        let key = format!("vrct.overlay.{}", name);
        let overlay = context
            .overlay()
            .map_err(|e| VrctError::Overlay(format!("Failed to get overlay interface: {:?}", e)))?
            .create_overlay(&key, name)
            .map_err(|e| VrctError::Overlay(format!("Failed to create overlay: {:?}", e)))?;

        // Set initial properties
        overlay
            .set_overlay_width_in_meters(config.ui_scaling)
            .map_err(|e| VrctError::Overlay(format!("Failed to set overlay width: {:?}", e)))?;

        overlay
            .set_overlay_alpha(config.opacity)
            .map_err(|e| VrctError::Overlay(format!("Failed to set overlay alpha: {:?}", e)))?;

        // Set initial position
        self.set_overlay_position_internal(&overlay, &config)?;

        // Show the overlay
        overlay
            .show_overlay()
            .map_err(|e| VrctError::Overlay(format!("Failed to show overlay: {:?}", e)))?;

        // Store overlay instance
        let instance = OverlayInstance {
            handle: overlay,
            config,
        };

        let mut overlays = self.overlays.lock().unwrap();
        overlays.insert(name.to_string(), instance);

        info!("Overlay created successfully: {}", name);
        Ok(())
    }

    /// Update overlay text
    pub fn update_overlay(&self, name: &str, text: &str) -> Result<()> {
        if !self.is_available() {
            return Err(VrctError::Overlay(
                "OpenVR not available, cannot update overlay".to_string(),
            ));
        }

        let overlays = self.overlays.lock().unwrap();
        let instance = overlays
            .get(name)
            .ok_or_else(|| VrctError::Overlay(format!("Overlay not found: {}", name)))?;

        // Render text to image
        let image = self.renderer.render_text(
            text,
            instance.config.width,
            instance.config.height,
            instance.config.font_size,
        )?;

        // Upload image to overlay
        instance
            .handle
            .set_overlay_from_file(&format!("temp_overlay_{}.png", name))
            .or_else(|_| {
                // If file method fails, try raw data method
                instance.handle.set_overlay_raw(
                    image.as_raw(),
                    instance.config.width,
                    instance.config.height,
                )
            })
            .map_err(|e| VrctError::Overlay(format!("Failed to update overlay texture: {:?}", e)))?;

        Ok(())
    }

    /// Set overlay position
    pub fn set_overlay_position(&self, name: &str, transform: Transform) -> Result<()> {
        if !self.is_available() {
            return Err(VrctError::Overlay(
                "OpenVR not available, cannot set overlay position".to_string(),
            ));
        }

        let mut overlays = self.overlays.lock().unwrap();
        let instance = overlays
            .get_mut(name)
            .ok_or_else(|| VrctError::Overlay(format!("Overlay not found: {}", name)))?;

        // Update config
        instance.config.position = transform.position;
        instance.config.rotation = transform.rotation;
        instance.config.ui_scaling = transform.scale;

        // Apply position
        self.set_overlay_position_internal(&instance.handle, &instance.config)?;

        // Apply scale
        instance
            .handle
            .set_overlay_width_in_meters(transform.scale)
            .map_err(|e| VrctError::Overlay(format!("Failed to set overlay width: {:?}", e)))?;

        Ok(())
    }

    /// Internal method to set overlay position
    fn set_overlay_position_internal(
        &self,
        overlay: &Overlay,
        config: &OverlayConfig,
    ) -> Result<()> {
        let system = self.system.as_ref().unwrap();
        let device_index = config.tracker.to_tracking_device_index(system);

        // Create transform matrix
        let transform = Self::create_transform_matrix(&config.position, &config.rotation);

        overlay
            .set_overlay_transform_tracked_device_relative(device_index, &transform)
            .map_err(|e| {
                VrctError::Overlay(format!("Failed to set overlay transform: {:?}", e))
            })?;

        Ok(())
    }

    /// Create a 3x4 transform matrix from position and rotation
    fn create_transform_matrix(position: &Vec3, rotation: &Vec3) -> [[f32; 4]; 3] {
        // Convert rotation from degrees to radians
        let rx = rotation.x.to_radians();
        let ry = rotation.y.to_radians();
        let rz = rotation.z.to_radians();

        // Calculate rotation matrix components
        let cos_x = rx.cos();
        let sin_x = rx.sin();
        let cos_y = ry.cos();
        let sin_y = ry.sin();
        let cos_z = rz.cos();
        let sin_z = rz.sin();

        // Combined rotation matrix (ZYX order)
        [
            [
                cos_y * cos_z,
                -cos_y * sin_z,
                sin_y,
                position.x,
            ],
            [
                cos_x * sin_z + sin_x * sin_y * cos_z,
                cos_x * cos_z - sin_x * sin_y * sin_z,
                -sin_x * cos_y,
                position.y,
            ],
            [
                sin_x * sin_z - cos_x * sin_y * cos_z,
                sin_x * cos_z + cos_x * sin_y * sin_z,
                cos_x * cos_y,
                position.z,
            ],
        ]
    }

    /// Update overlay settings
    pub fn update_overlay_settings(&self, name: &str, config: OverlayConfig) -> Result<()> {
        if !self.is_available() {
            return Err(VrctError::Overlay(
                "OpenVR not available, cannot update overlay settings".to_string(),
            ));
        }

        let mut overlays = self.overlays.lock().unwrap();
        let instance = overlays
            .get_mut(name)
            .ok_or_else(|| VrctError::Overlay(format!("Overlay not found: {}", name)))?;

        // Update opacity
        if (instance.config.opacity - config.opacity).abs() > f32::EPSILON {
            instance
                .handle
                .set_overlay_alpha(config.opacity)
                .map_err(|e| VrctError::Overlay(format!("Failed to set overlay alpha: {:?}", e)))?;
        }

        // Update position if changed
        if instance.config.tracker != config.tracker
            || instance.config.position.x != config.position.x
            || instance.config.position.y != config.position.y
            || instance.config.position.z != config.position.z
            || instance.config.rotation.x != config.rotation.x
            || instance.config.rotation.y != config.rotation.y
            || instance.config.rotation.z != config.rotation.z
        {
            self.set_overlay_position_internal(&instance.handle, &config)?;
        }

        // Update scale
        if (instance.config.ui_scaling - config.ui_scaling).abs() > f32::EPSILON {
            instance
                .handle
                .set_overlay_width_in_meters(config.ui_scaling)
                .map_err(|e| {
                    VrctError::Overlay(format!("Failed to set overlay width: {:?}", e))
                })?;
        }

        // Update config
        instance.config = config;

        Ok(())
    }

    /// Destroy an overlay
    pub fn destroy_overlay(&self, name: &str) -> Result<()> {
        let mut overlays = self.overlays.lock().unwrap();
        
        if let Some(instance) = overlays.remove(name) {
            // Hide overlay before destroying
            let _ = instance.handle.hide_overlay();
            info!("Overlay destroyed: {}", name);
        }

        Ok(())
    }

    /// Get list of active overlays
    pub fn list_overlays(&self) -> Vec<String> {
        let overlays = self.overlays.lock().unwrap();
        overlays.keys().cloned().collect()
    }
}

#[cfg(feature = "vr-overlay")]
impl Drop for OverlayManager {
    fn drop(&mut self) {
        // Clean up all overlays
        let overlays = self.overlays.lock().unwrap();
        for (name, instance) in overlays.iter() {
            let _ = instance.handle.hide_overlay();
            info!("Cleaned up overlay: {}", name);
        }
    }
}

// Stub implementation when VR overlay feature is disabled
#[cfg(not(feature = "vr-overlay"))]
impl OverlayManager {
    pub fn new() -> Result<Self> {
        warn!("Overlay manager created without OpenVR support (feature disabled)");
        let renderer = Arc::new(TextRenderer::new()?);
        Ok(Self {
            overlays: Arc::new(Mutex::new(HashMap::new())),
            renderer,
        })
    }

    pub fn is_available(&self) -> bool {
        false
    }

    pub fn create_overlay(&self, _name: &str, _config: OverlayConfig) -> Result<()> {
        Err(VrctError::Overlay(
            "OpenVR support not compiled in. Enable 'vr-overlay' feature.".to_string(),
        ))
    }

    pub fn update_overlay(&self, _name: &str, _text: &str) -> Result<()> {
        Err(VrctError::Overlay(
            "OpenVR support not compiled in. Enable 'vr-overlay' feature.".to_string(),
        ))
    }

    pub fn set_overlay_position(&self, _name: &str, _transform: Transform) -> Result<()> {
        Err(VrctError::Overlay(
            "OpenVR support not compiled in. Enable 'vr-overlay' feature.".to_string(),
        ))
    }

    pub fn update_overlay_settings(&self, _name: &str, _config: OverlayConfig) -> Result<()> {
        Err(VrctError::Overlay(
            "OpenVR support not compiled in. Enable 'vr-overlay' feature.".to_string(),
        ))
    }

    pub fn destroy_overlay(&self, _name: &str) -> Result<()> {
        Ok(())
    }

    pub fn list_overlays(&self) -> Vec<String> {
        Vec::new()
    }
}
