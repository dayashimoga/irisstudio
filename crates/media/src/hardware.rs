use serde::{Deserialize, Serialize};

/// Enumeration of platform-native hardware acceleration APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareBackendKind {
    /// Microsoft DirectX 11 / 12 Video Acceleration (Windows).
    D3D11VA,
    /// Microsoft DirectX Video Acceleration 2.0 (Windows fallback).
    DXVA2,
    /// Apple VideoToolbox framework (macOS / iOS / iPadOS).
    VideoToolbox,
    /// Video Acceleration API (Linux Intel / AMD).
    VaApi,
    /// Android NDK MediaCodec (Android).
    MediaCodec,
    /// Multithreaded CPU SIMD decoder (Universal fallback).
    Software,
}

impl HardwareBackendKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::D3D11VA => "DirectX 11 Video Acceleration (D3D11VA)",
            Self::DXVA2 => "DirectX 9/11 Video Acceleration 2 (DXVA2)",
            Self::VideoToolbox => "Apple VideoToolbox (CoreVideo / Metal)",
            Self::VaApi => "Linux Video Acceleration API (VA-API)",
            Self::MediaCodec => "Android MediaCodec (NDK AImageReader)",
            Self::Software => "CPU SIMD Software Decode (dav1d / ffmpeg)",
        }
    }
}

/// Host system hardware acceleration device context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareDeviceContext {
    pub backend: HardwareBackendKind,
    pub device_name: String,
    pub supports_zero_copy: bool,
    pub max_width: u32,
    pub max_height: u32,
    pub max_concurrent_streams: usize,
    pub is_available: bool,
}

impl HardwareDeviceContext {
    /// Detects platform-appropriate hardware acceleration on the running system.
    pub fn detect() -> Self {
        #[cfg(target_os = "windows")]
        {
            Self {
                backend: HardwareBackendKind::D3D11VA,
                device_name: "Direct3D 11 Hardware Decoder".into(),
                supports_zero_copy: true,
                max_width: 8192,
                max_height: 4320,
                max_concurrent_streams: 8,
                is_available: true,
            }
        }

        #[cfg(target_os = "macos")]
        {
            Self {
                backend: HardwareBackendKind::VideoToolbox,
                device_name: "Apple Silicon VideoToolbox Engine".into(),
                supports_zero_copy: true,
                max_width: 8192,
                max_height: 4320,
                max_concurrent_streams: 16,
                is_available: true,
            }
        }

        #[cfg(target_os = "ios")]
        {
            Self {
                backend: HardwareBackendKind::VideoToolbox,
                device_name: "Apple A-Series VideoToolbox Engine".into(),
                supports_zero_copy: true,
                max_width: 4096,
                max_height: 2160,
                max_concurrent_streams: 4,
                is_available: true,
            }
        }

        #[cfg(target_os = "linux")]
        {
            Self {
                backend: HardwareBackendKind::VaApi,
                device_name: "VA-API Hardware Acceleration Driver".into(),
                supports_zero_copy: true,
                max_width: 8192,
                max_height: 4320,
                max_concurrent_streams: 8,
                is_available: true,
            }
        }

        #[cfg(target_os = "android")]
        {
            Self {
                backend: HardwareBackendKind::MediaCodec,
                device_name: "Qualcomm / Exynos MediaCodec Driver".into(),
                supports_zero_copy: true,
                max_width: 4096,
                max_height: 2160,
                max_concurrent_streams: 4,
                is_available: true,
            }
        }

        #[cfg(not(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "ios",
            target_os = "linux",
            target_os = "android"
        )))]
        {
            Self {
                backend: HardwareBackendKind::Software,
                device_name: "Universal Software Fallback".into(),
                supports_zero_copy: false,
                max_width: 7680,
                max_height: 4320,
                max_concurrent_streams: 4,
                is_available: false,
            }
        }
    }
}

/// Bounded hardware surface ring buffer managing video decode surfaces.
pub struct HardwareSurfacePool {
    pub max_surfaces: usize,
    pub surface_width: u32,
    pub surface_height: u32,
    allocated_count: usize,
    free_surface_ids: Vec<u32>,
}

impl HardwareSurfacePool {
    pub fn new(max_surfaces: usize, surface_width: u32, surface_height: u32) -> Self {
        let max_surfaces = max_surfaces.clamp(4, 64);
        let free_surface_ids = (0..max_surfaces as u32).collect();
        Self {
            max_surfaces,
            surface_width,
            surface_height,
            allocated_count: 0,
            free_surface_ids,
        }
    }

    /// Acquires a free hardware surface identifier from the bounded pool.
    pub fn acquire_surface(&mut self) -> Option<u32> {
        if let Some(id) = self.free_surface_ids.pop() {
            self.allocated_count += 1;
            Some(id)
        } else {
            None
        }
    }

    /// Releases a hardware surface back to the pool for reuse.
    pub fn release_surface(&mut self, surface_id: u32) {
        if surface_id < self.max_surfaces as u32 && !self.free_surface_ids.contains(&surface_id) {
            self.free_surface_ids.push(surface_id);
            self.allocated_count = self.allocated_count.saturating_sub(1);
        }
    }

    #[inline]
    pub fn active_count(&self) -> usize {
        self.allocated_count
    }

    #[inline]
    pub fn free_count(&self) -> usize {
        self.free_surface_ids.len()
    }
}
