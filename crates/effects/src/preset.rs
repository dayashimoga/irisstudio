use crate::adjustment::AdjustmentParameters;
use crate::levels::LevelsParameters;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresetKind {
    LookPreset,
    PhotoPreset,
    ProjectTemplate,
    ExportPreset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaCompatibility {
    Universal,
    VideoOnly,
    PhotoOnly,
    AudioOnly,
}

/// Strongly-typed, versioned preset specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Preset {
    pub id: String,
    pub schema_version: u32,
    pub name: String,
    pub category: String,
    pub description: String,
    pub kind: PresetKind,
    pub media_compatibility: MediaCompatibility,
    pub adjustments: AdjustmentParameters,
    pub vignette: f32,   // 0.0 to 1.0 (amount)
    pub sharpening: f32, // 0.0 to 1.0
    pub clarity: f32,    // -100.0 to +100.0
    pub grain: f32,      // 0.0 to 1.0
    pub lut_preset: Option<String>,
    pub lut_intensity: f32, // 0.0 to 1.0
    pub levels: Option<LevelsParameters>,
    pub audio_eq_low_gain: Option<f32>,
    pub audio_eq_mid_gain: Option<f32>,
    pub audio_eq_high_gain: Option<f32>,
    pub color_space_assumptions: String,
    pub supported_intensity_behavior: String,
    pub is_builtin: bool,
    pub created_at: String,
    pub modified_at: String,
}

impl Preset {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    /// Validates untrusted imported preset data against range boundaries and safety constraints.
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Preset name cannot be empty".into());
        }
        if self.schema_version > Self::CURRENT_SCHEMA_VERSION {
            return Err(format!(
                "Unsupported preset schema version {}. Current version is {}",
                self.schema_version,
                Self::CURRENT_SCHEMA_VERSION
            ));
        }
        if self.name.contains('<') || self.name.contains('>') || self.name.contains('\0') {
            return Err("Invalid characters in preset name".into());
        }
        if self.adjustments.exposure < -10.0 || self.adjustments.exposure > 10.0 {
            return Err("Exposure out of safe range [-10.0, 10.0]".into());
        }
        if self.adjustments.contrast < -150.0 || self.adjustments.contrast > 150.0 {
            return Err("Contrast out of safe range [-150.0, 150.0]".into());
        }
        if self.vignette < 0.0 || self.vignette > 1.0 {
            return Err("Vignette out of safe range [0.0, 1.0]".into());
        }
        if self.sharpening < 0.0 || self.sharpening > 1.0 {
            return Err("Sharpening out of safe range [0.0, 1.0]".into());
        }
        if self.lut_intensity < 0.0 || self.lut_intensity > 1.0 {
            return Err("LUT intensity out of safe range [0.0, 1.0]".into());
        }
        Ok(())
    }

    /// Blends preset parameters linearly by intensity in [0.0, 1.0].
    pub fn blend_with_intensity(&self, intensity: f32) -> Self {
        let t = intensity.clamp(0.0, 1.0);
        let mut b = self.clone();

        b.adjustments.exposure *= t;
        b.adjustments.contrast *= t;
        b.adjustments.temperature *= t;
        b.adjustments.tint *= t;
        b.adjustments.highlights *= t;
        b.adjustments.shadows *= t;
        b.adjustments.saturation *= t;
        b.adjustments.vibrance *= t;

        b.vignette *= t;
        b.sharpening *= t;
        b.clarity *= t;
        b.grain *= t;
        b.lut_intensity *= t;

        if let Some(eq) = &mut b.audio_eq_low_gain {
            *eq *= t;
        }
        if let Some(eq) = &mut b.audio_eq_mid_gain {
            *eq *= t;
        }
        if let Some(eq) = &mut b.audio_eq_high_gain {
            *eq *= t;
        }

        b
    }

    /// Serializes preset to portable JSON (.irispreset).
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("Serialization error: {}", e))
    }

    /// Deserializes and validates an untrusted preset JSON string.
    pub fn from_json(json: &str) -> Result<Self, String> {
        let preset: Self =
            serde_json::from_str(json).map_err(|e| format!("Deserialization error: {}", e))?;
        preset.validate()?;
        Ok(preset)
    }

    // =========================================================================
    // Built-in Video Look Presets (Phase 4)
    // =========================================================================

    pub fn builtin_video_looks() -> Vec<Preset> {
        vec![
            Self::make_video_look(
                "video_look_natural",
                "Natural",
                "General",
                "Faithful, uncompressed color rendition preserving source dynamic range and skin accuracy.",
                AdjustmentParameters {
                    exposure: 0.0,
                    contrast: 0.0,
                    temperature: 0.0,
                    tint: 0.0,
                    highlights: 0.0,
                    shadows: 0.0,
                    saturation: 0.0,
                    vibrance: 0.0,
                },
                0.0, 0.0, None, 0.0, None,
            ),
            Self::make_video_look(
                "video_look_cinema_neutral",
                "Cinema Neutral",
                "Cinema",
                "Classic filmic contrast curve with soft highlight compression and clean mid-tones.",
                AdjustmentParameters {
                    exposure: 0.0,
                    contrast: 15.0,
                    temperature: 2.0,
                    tint: -1.0,
                    highlights: -18.0,
                    shadows: 8.0,
                    saturation: -5.0,
                    vibrance: 5.0,
                },
                0.15, 0.1, Some("bleach_bypass"), 0.25, Some((1.5, 0.0, 0.5)),
            ),
            Self::make_video_look(
                "video_look_cinema_warm",
                "Cinema Warm",
                "Cinema",
                "Hollywood Teal & Orange aesthetic with rich golden skin tones and deep atmospheric shadows.",
                AdjustmentParameters {
                    exposure: 0.1,
                    contrast: 22.0,
                    temperature: 18.0,
                    tint: 4.0,
                    highlights: -15.0,
                    shadows: 12.0,
                    saturation: 8.0,
                    vibrance: 16.0,
                },
                0.25, 0.15, Some("teal_and_orange"), 0.70, Some((2.0, 0.5, 1.0)),
            ),
            Self::make_video_look(
                "video_look_cinema_cool",
                "Cinema Cool",
                "Cinema",
                "Moody modern sci-fi look with slate shadows, cool mid-tones, and restrained highlights.",
                AdjustmentParameters {
                    exposure: -0.1,
                    contrast: 18.0,
                    temperature: -20.0,
                    tint: -6.0,
                    highlights: -12.0,
                    shadows: 6.0,
                    saturation: -8.0,
                    vibrance: 6.0,
                },
                0.20, 0.15, Some("faded_film"), 0.35, Some((0.5, 0.0, 1.5)),
            ),
            Self::make_video_look(
                "video_look_film_inspired",
                "Film-Inspired",
                "Film",
                "Organic 35mm film stock emulsion with elevated toe, gentle highlight roll-off, and subtle grain.",
                AdjustmentParameters {
                    exposure: 0.05,
                    contrast: 12.0,
                    temperature: 8.0,
                    tint: 2.0,
                    highlights: -22.0,
                    shadows: 15.0,
                    saturation: -6.0,
                    vibrance: 8.0,
                },
                0.30, 0.10, Some("warm_vintage"), 0.50, Some((1.0, 0.0, -0.5)),
            ),
            Self::make_video_look(
                "video_look_documentary",
                "Documentary",
                "Documentary",
                "Punchy realism with enhanced micro-contrast, crisp speech EQ, and lifelike skin tones.",
                AdjustmentParameters {
                    exposure: 0.0,
                    contrast: 14.0,
                    temperature: 0.0,
                    tint: 0.0,
                    highlights: -8.0,
                    shadows: 10.0,
                    saturation: 4.0,
                    vibrance: 12.0,
                },
                0.05, 0.20, None, 0.0, Some((-1.0, 2.5, 1.0)),
            ),
            Self::make_video_look(
                "video_look_sports_action",
                "Sports/Action",
                "Sports",
                "Ultra-vivid colors and crisp edge clarity tailored for fast motion, preserving source framerate.",
                AdjustmentParameters {
                    exposure: 0.15,
                    contrast: 24.0,
                    temperature: 4.0,
                    tint: 0.0,
                    highlights: -10.0,
                    shadows: 14.0,
                    saturation: 18.0,
                    vibrance: 25.0,
                },
                0.10, 0.35, None, 0.0, Some((3.0, 1.0, 2.5)),
            ),
            Self::make_video_look(
                "video_look_travel",
                "Travel",
                "Travel",
                "Vibrant blues and lush emerald foliage with lifted warm sunlight for outdoor exploration.",
                AdjustmentParameters {
                    exposure: 0.10,
                    contrast: 18.0,
                    temperature: 12.0,
                    tint: -2.0,
                    highlights: -14.0,
                    shadows: 12.0,
                    saturation: 16.0,
                    vibrance: 22.0,
                },
                0.15, 0.15, Some("teal_and_orange"), 0.30, Some((1.5, 0.5, 1.5)),
            ),
            Self::make_video_look(
                "video_look_landscape",
                "Landscape",
                "Landscape",
                "Deep shadow richness, sky highlight preservation, and enhanced nature color separation.",
                AdjustmentParameters {
                    exposure: 0.0,
                    contrast: 20.0,
                    temperature: -4.0,
                    tint: -2.0,
                    highlights: -25.0,
                    shadows: 16.0,
                    saturation: 14.0,
                    vibrance: 20.0,
                },
                0.15, 0.25, None, 0.0, Some((2.0, -0.5, 2.0)),
            ),
            Self::make_video_look(
                "video_look_portrait",
                "Portrait/Interview",
                "Portrait",
                "Soft skin-tone smoothing, gentle highlight roll-off, and dialogue-optimized audio EQ.",
                AdjustmentParameters {
                    exposure: 0.08,
                    contrast: 6.0,
                    temperature: 6.0,
                    tint: 3.0,
                    highlights: -16.0,
                    shadows: 10.0,
                    saturation: 2.0,
                    vibrance: 6.0,
                },
                0.12, 0.08, None, 0.0, Some((-1.5, 3.0, 1.2)),
            ),
            Self::make_video_look(
                "video_look_product",
                "Product",
                "Commercial",
                "Studio neutral white balance, true-to-life colors, and sharp detail without color casts.",
                AdjustmentParameters {
                    exposure: 0.05,
                    contrast: 12.0,
                    temperature: 0.0,
                    tint: 0.0,
                    highlights: -6.0,
                    shadows: 6.0,
                    saturation: 6.0,
                    vibrance: 10.0,
                },
                0.0, 0.25, None, 0.0, Some((0.0, 1.0, 1.5)),
            ),
            Self::make_video_look(
                "video_look_food",
                "Food",
                "Commercial",
                "Warm appetizing amber tones with saturated reds and greens for culinary presentations.",
                AdjustmentParameters {
                    exposure: 0.12,
                    contrast: 16.0,
                    temperature: 20.0,
                    tint: 5.0,
                    highlights: -10.0,
                    shadows: 14.0,
                    saturation: 18.0,
                    vibrance: 24.0,
                },
                0.18, 0.15, Some("warm_vintage"), 0.35, Some((1.5, 1.0, 1.0)),
            ),
            Self::make_video_look(
                "video_look_night",
                "Night/Low-Light",
                "Creative",
                "Clean shadow illumination with restrained highlights, avoiding noise amplification.",
                AdjustmentParameters {
                    exposure: 0.25,
                    contrast: 10.0,
                    temperature: -8.0,
                    tint: -2.0,
                    highlights: -20.0,
                    shadows: 22.0,
                    saturation: -5.0,
                    vibrance: 8.0,
                },
                0.20, 0.05, None, 0.0, Some((1.0, 0.0, -1.0)),
            ),
            Self::make_video_look(
                "video_look_vintage",
                "Vintage",
                "Vintage",
                "Nostalgic 1970s warm sepia cast, compressed dynamic range, and gentle vignette.",
                AdjustmentParameters {
                    exposure: -0.05,
                    contrast: -8.0,
                    temperature: 28.0,
                    tint: 8.0,
                    highlights: -25.0,
                    shadows: 20.0,
                    saturation: -18.0,
                    vibrance: -10.0,
                },
                0.40, 0.05, Some("warm_vintage"), 0.65, Some((2.0, -1.0, -2.0)),
            ),
            Self::make_video_look(
                "video_look_bw",
                "Black & White",
                "Monochrome",
                "High-contrast classic monochrome conversion with deep blacks and bright silvery whites.",
                AdjustmentParameters {
                    exposure: 0.0,
                    contrast: 28.0,
                    temperature: 0.0,
                    tint: 0.0,
                    highlights: -10.0,
                    shadows: 12.0,
                    saturation: -100.0,
                    vibrance: -100.0,
                },
                0.25, 0.20, Some("monochrome_contrast"), 0.90, Some((1.0, 0.5, 1.0)),
            ),
        ]
    }

    // =========================================================================
    // Built-in Photo Presets (Phase 5)
    // =========================================================================

    pub fn builtin_photo_presets() -> Vec<Preset> {
        vec![
            Self::make_photo_preset(
                "photo_preset_natural",
                "Natural",
                "General",
                "Pure true-to-life color reproduction without artificial enhancement.",
                AdjustmentParameters::default(),
                0.0, 0.0,
            ),
            Self::make_photo_preset(
                "photo_preset_portrait",
                "Portrait",
                "Portrait",
                "Flattering skin warmth (+8), soft contrast, and highlight roll-off.",
                AdjustmentParameters {
                    exposure: 0.05,
                    contrast: 6.0,
                    temperature: 8.0,
                    tint: 3.0,
                    highlights: -18.0,
                    shadows: 12.0,
                    saturation: 2.0,
                    vibrance: 8.0,
                },
                0.15, 0.08,
            ),
            Self::make_photo_preset(
                "photo_preset_landscape",
                "Landscape",
                "Landscape",
                "Deep azure skies, punchy emerald foliage, and rich shadow contrast.",
                AdjustmentParameters {
                    exposure: 0.0,
                    contrast: 22.0,
                    temperature: -4.0,
                    tint: -2.0,
                    highlights: -24.0,
                    shadows: 14.0,
                    saturation: 15.0,
                    vibrance: 24.0,
                },
                0.10, 0.25,
            ),
            Self::make_photo_preset(
                "photo_preset_vivid",
                "Vivid",
                "Creative",
                "Punchy dynamic range with rich, lively color saturation and micro-contrast.",
                AdjustmentParameters {
                    exposure: 0.10,
                    contrast: 20.0,
                    temperature: 2.0,
                    tint: 0.0,
                    highlights: -12.0,
                    shadows: 15.0,
                    saturation: 20.0,
                    vibrance: 32.0,
                },
                0.08, 0.22,
            ),
            Self::make_photo_preset(
                "photo_preset_soft",
                "Soft",
                "Portrait",
                "Dreamy pastel aesthetic with relaxed contrast and bright, airy exposure.",
                AdjustmentParameters {
                    exposure: 0.18,
                    contrast: -16.0,
                    temperature: 4.0,
                    tint: 2.0,
                    highlights: -20.0,
                    shadows: 20.0,
                    saturation: -8.0,
                    vibrance: 6.0,
                },
                0.05, 0.0,
            ),
            Self::make_photo_preset(
                "photo_preset_warm",
                "Warm",
                "Color",
                "Inviting golden tones with gentle amber highlights.",
                AdjustmentParameters {
                    exposure: 0.05,
                    contrast: 10.0,
                    temperature: 24.0,
                    tint: 4.0,
                    highlights: -10.0,
                    shadows: 10.0,
                    saturation: 8.0,
                    vibrance: 15.0,
                },
                0.12, 0.10,
            ),
            Self::make_photo_preset(
                "photo_preset_cool",
                "Cool",
                "Color",
                "Crisp Nordic blue temperature with pristine clean whites.",
                AdjustmentParameters {
                    exposure: -0.05,
                    contrast: 14.0,
                    temperature: -24.0,
                    tint: -5.0,
                    highlights: -10.0,
                    shadows: 6.0,
                    saturation: -6.0,
                    vibrance: 8.0,
                },
                0.10, 0.12,
            ),
            Self::make_photo_preset(
                "photo_preset_golden_hour",
                "Golden Hour",
                "Landscape",
                "Rich late-afternoon sunlight with glowing amber highlights and lifted shadows.",
                AdjustmentParameters {
                    exposure: 0.15,
                    contrast: 18.0,
                    temperature: 32.0,
                    tint: 6.0,
                    highlights: -20.0,
                    shadows: 22.0,
                    saturation: 14.0,
                    vibrance: 25.0,
                },
                0.22, 0.15,
            ),
            Self::make_photo_preset(
                "photo_preset_night",
                "Night",
                "Creative",
                "Controlled shadow exposure lift without blowing out artificial light sources.",
                AdjustmentParameters {
                    exposure: 0.35,
                    contrast: 12.0,
                    temperature: -6.0,
                    tint: 0.0,
                    highlights: -30.0,
                    shadows: 25.0,
                    saturation: 4.0,
                    vibrance: 12.0,
                },
                0.18, 0.08,
            ),
            Self::make_photo_preset(
                "photo_preset_product",
                "Product",
                "Commercial",
                "Studio neutral lighting balance, accurate saturation, and high micro-clarity.",
                AdjustmentParameters {
                    exposure: 0.06,
                    contrast: 14.0,
                    temperature: 0.0,
                    tint: 0.0,
                    highlights: -8.0,
                    shadows: 8.0,
                    saturation: 5.0,
                    vibrance: 10.0,
                },
                0.0, 0.30,
            ),
            Self::make_photo_preset(
                "photo_preset_food",
                "Food",
                "Commercial",
                "Warm appetizing glow with rich reds, yellows, and fresh greens.",
                AdjustmentParameters {
                    exposure: 0.14,
                    contrast: 16.0,
                    temperature: 22.0,
                    tint: 4.0,
                    highlights: -12.0,
                    shadows: 15.0,
                    saturation: 18.0,
                    vibrance: 25.0,
                },
                0.15, 0.18,
            ),
            Self::make_photo_preset(
                "photo_preset_document",
                "Document/Text",
                "Utility",
                "High contrast black-and-white curve with clipped paper whites for maximum legibility.",
                AdjustmentParameters {
                    exposure: 0.30,
                    contrast: 45.0,
                    temperature: 0.0,
                    tint: 0.0,
                    highlights: 25.0,
                    shadows: -25.0,
                    saturation: -90.0,
                    vibrance: -90.0,
                },
                0.0, 0.40,
            ),
            Self::make_photo_preset(
                "photo_preset_vintage",
                "Vintage",
                "Vintage",
                "1970s print film look with lifted blacks, muted saturation, and soft edge vignette.",
                AdjustmentParameters {
                    exposure: -0.05,
                    contrast: -10.0,
                    temperature: 26.0,
                    tint: 8.0,
                    highlights: -24.0,
                    shadows: 22.0,
                    saturation: -22.0,
                    vibrance: -12.0,
                },
                0.35, 0.05,
            ),
            Self::make_photo_preset(
                "photo_preset_matte",
                "Matte",
                "Film",
                "Editorial matte finish with lifted black floor and compressed highlights.",
                AdjustmentParameters {
                    exposure: 0.05,
                    contrast: -8.0,
                    temperature: 4.0,
                    tint: 2.0,
                    highlights: -28.0,
                    shadows: 24.0,
                    saturation: -12.0,
                    vibrance: 4.0,
                },
                0.20, 0.06,
            ),
            Self::make_photo_preset(
                "photo_preset_bw",
                "Black & White",
                "Monochrome",
                "Fine-art monochrome with rich deep blacks and silvery luminous whites.",
                AdjustmentParameters {
                    exposure: 0.0,
                    contrast: 30.0,
                    temperature: 0.0,
                    tint: 0.0,
                    highlights: -12.0,
                    shadows: 14.0,
                    saturation: -100.0,
                    vibrance: -100.0,
                },
                0.20, 0.20,
            ),
        ]
    }

    #[allow(clippy::too_many_arguments)]
    fn make_video_look(
        id: &str,
        name: &str,
        category: &str,
        description: &str,
        adjustments: AdjustmentParameters,
        vignette: f32,
        sharpening: f32,
        lut_preset: Option<&str>,
        lut_intensity: f32,
        audio_eq: Option<(f32, f32, f32)>,
    ) -> Self {
        Self {
            id: id.into(),
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            name: name.into(),
            category: category.into(),
            description: description.into(),
            kind: PresetKind::LookPreset,
            media_compatibility: MediaCompatibility::VideoOnly,
            adjustments,
            vignette,
            sharpening,
            clarity: 0.0,
            grain: 0.0,
            lut_preset: lut_preset.map(|s| s.to_string()),
            lut_intensity,
            levels: None,
            audio_eq_low_gain: audio_eq.map(|(l, _, _)| l),
            audio_eq_mid_gain: audio_eq.map(|(_, m, _)| m),
            audio_eq_high_gain: audio_eq.map(|(_, _, h)| h),
            color_space_assumptions: "Rec.709 / Linear sRGB".into(),
            supported_intensity_behavior: "linear_interpolation".into(),
            is_builtin: true,
            created_at: "2026-10-21T00:00:00Z".into(),
            modified_at: "2026-10-21T00:00:00Z".into(),
        }
    }

    fn make_photo_preset(
        id: &str,
        name: &str,
        category: &str,
        description: &str,
        adjustments: AdjustmentParameters,
        vignette: f32,
        sharpening: f32,
    ) -> Self {
        Self {
            id: id.into(),
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            name: name.into(),
            category: category.into(),
            description: description.into(),
            kind: PresetKind::PhotoPreset,
            media_compatibility: MediaCompatibility::PhotoOnly,
            adjustments,
            vignette,
            sharpening,
            clarity: 0.0,
            grain: 0.0,
            lut_preset: None,
            lut_intensity: 0.0,
            levels: None,
            audio_eq_low_gain: None,
            audio_eq_mid_gain: None,
            audio_eq_high_gain: None,
            color_space_assumptions: "sRGB / Linear sRGB".into(),
            supported_intensity_behavior: "linear_interpolation".into(),
            is_builtin: true,
            created_at: "2026-10-21T00:00:00Z".into(),
            modified_at: "2026-10-21T00:00:00Z".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_video_looks_validation() {
        let looks = Preset::builtin_video_looks();
        assert_eq!(looks.len(), 15);
        for look in &looks {
            assert!(look.validate().is_ok());
            assert_eq!(look.kind, PresetKind::LookPreset);
            assert_eq!(look.schema_version, 1);
            assert!(look.is_builtin);
        }
    }

    #[test]
    fn test_builtin_photo_presets_validation() {
        let presets = Preset::builtin_photo_presets();
        assert_eq!(presets.len(), 15);
        for p in &presets {
            assert!(p.validate().is_ok());
            assert_eq!(p.kind, PresetKind::PhotoPreset);
            assert_eq!(p.schema_version, 1);
            assert!(p.is_builtin);
        }
    }

    #[test]
    fn test_preset_intensity_blending() {
        let look = &Preset::builtin_video_looks()[2]; // Cinema Warm
        let half = look.blend_with_intensity(0.5);
        assert!((half.adjustments.contrast - look.adjustments.contrast * 0.5).abs() < 1e-4);
        assert!((half.vignette - look.vignette * 0.5).abs() < 1e-4);

        let zero = look.blend_with_intensity(0.0);
        assert_eq!(zero.adjustments.contrast, 0.0);
        assert_eq!(zero.vignette, 0.0);
    }

    #[test]
    fn test_preset_json_roundtrip_and_safety() {
        let p = &Preset::builtin_video_looks()[1];
        let json = p.to_json().unwrap();
        let loaded = Preset::from_json(&json).unwrap();
        assert_eq!(loaded.name, p.name);
        assert_eq!(loaded.adjustments.contrast, p.adjustments.contrast);

        // Safety: malicious payload rejection
        let malicious_json = json.replace("\"Cinema Neutral\"", "\"<script>alert(1)</script>\"");
        assert!(Preset::from_json(&malicious_json).is_err());
    }
}
