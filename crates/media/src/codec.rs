use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodecCapability {
    pub codec_name: String,
    pub is_hardware_accelerated: bool,
    pub supports_decoding: bool,
    pub supports_encoding: bool,
    pub max_width: u32,
    pub max_height: u32,
}

pub struct HardwareDecoder;

impl HardwareDecoder {
    /// Detects host system hardware acceleration capabilities.
    pub fn query_capabilities() -> Vec<CodecCapability> {
        let mut caps = Vec::new();

        #[cfg(target_os = "windows")]
        {
            caps.push(CodecCapability {
                codec_name: "H.264 / AVC".into(),
                is_hardware_accelerated: true,
                supports_decoding: true,
                supports_encoding: true,
                max_width: 4096,
                max_height: 2160,
            });
            caps.push(CodecCapability {
                codec_name: "H.265 / HEVC".into(),
                is_hardware_accelerated: true,
                supports_decoding: true,
                supports_encoding: true,
                max_width: 8192,
                max_height: 4320,
            });
        }

        #[cfg(target_os = "macos")]
        {
            caps.push(CodecCapability {
                codec_name: "Apple VideoToolbox H.264".into(),
                is_hardware_accelerated: true,
                supports_decoding: true,
                supports_encoding: true,
                max_width: 4096,
                max_height: 2160,
            });
            caps.push(CodecCapability {
                codec_name: "Apple VideoToolbox HEVC/ProRes".into(),
                is_hardware_accelerated: true,
                supports_decoding: true,
                supports_encoding: true,
                max_width: 8192,
                max_height: 4320,
            });
        }

        #[cfg(target_os = "linux")]
        {
            caps.push(CodecCapability {
                codec_name: "VA-API H.264".into(),
                is_hardware_accelerated: true,
                supports_decoding: true,
                supports_encoding: true,
                max_width: 4096,
                max_height: 2160,
            });
        }

        #[cfg(target_os = "android")]
        {
            caps.push(CodecCapability {
                codec_name: "Android MediaCodec AVC".into(),
                is_hardware_accelerated: true,
                supports_decoding: true,
                supports_encoding: true,
                max_width: 4096,
                max_height: 2160,
            });
        }

        // Software universal baseline
        caps.push(CodecCapability {
            codec_name: "AV1 (Software dav1d)".into(),
            is_hardware_accelerated: false,
            supports_decoding: true,
            supports_encoding: false,
            max_width: 7680,
            max_height: 4320,
        });

        caps.push(CodecCapability {
            codec_name: "VP9 (Software)".into(),
            is_hardware_accelerated: false,
            supports_decoding: true,
            supports_encoding: true,
            max_width: 3840,
            max_height: 2160,
        });

        caps
    }
}
