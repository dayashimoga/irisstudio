# Iris Studio: Presets & Templates Reference Specification

## 1. Overview & Classification Hierarchy

Iris Studio strictly segregates presets into four distinct, typed domains:
1. **Look Presets (`PresetKind::LookPreset`)**: Tone, color grading, 3D LUTs, vignette, sharpening, curves, and audio equalization parameters. Look presets never mutate sequence frame rate, aspect ratio, or canvas dimensions.
2. **Project Templates (`PresetKind::ProjectTemplate`)**: Aspect ratio, sequence dimensions, default track configurations, and title layouts.
3. **Export Presets (`PresetKind::ExportPreset`)**: Container, codec, resolution, bitrate, audio sample rate, and channels.
4. **Auto Adjust**: Dynamic content analysis proposing non-destructive adjustments without fixed filter values.

---

## 2. Built-in Video Look Presets

| Name | Category | Primary Adjustments | Vignette | EQ Bias | Description |
|:---|:---|:---|:---:|:---:|:---|
| **Natural** | General | Baseline (0.0 across all sliders) | 0.00 | Flat | Authentic, uncompressed color rendition preserving source dynamic range and skin accuracy. |
| **Cinema Neutral** | Cinematic | Contrast +12, Highlights -5, Shadows +5, Vibrance +5 | 0.08 | Flat | Standard theatrical S-curve with gentle highlight roll-off and preserved shadow depth. |
| **Cinema Warm** | Cinematic | Contrast +14, Temp +12, Highlights -8, Shadows +6, Sat +8 | 0.12 | Flat | Golden hour warmth with subtle teal-amber complementary contrast. |
| **Cinema Cool** | Cinematic | Contrast +16, Temp -10, Tint -4, Highlights +4, Shadows -6 | 0.10 | Flat | Moody European cinema aesthetic with cool shadows, restrained skin tones, and deep blues. |
| **Film-Inspired** | Film | Contrast +10, Temp +5, Highlights -12, Shadows +10, Sat -5 | 0.14 | Flat | Organic 35mm film stock response with soft rolled-off highlights and film grain. |
| **Documentary** | General | Contrast +6, Highlights -4, Shadows +6, Vibrance +4 | 0.00 | Flat | True-to-life broadcast grading preserving authentic environmental and skin tones. |
| **Sports / Action** | Action | Contrast +18, Highlights +5, Shadows +8, Sat +16, Vib +14 | 0.00 | Low +2.0dB, High +1.5dB | Punchy contrast, saturated team colors, and enhanced edge clarity for high-motion capture. |
| **Travel** | Lifestyle | Exposure +0.05, Contrast +15, Temp +6, Highlights -10, Sat +18 | 0.06 | Flat | Rich azure skies, golden earth tones, and punchy dynamic range for lifestyle footage. |
| **Landscape** | Nature | Contrast +16, Highlights -15, Shadows +15, Sat +14, Vib +18 | 0.00 | Flat | Deep verdant foliage, rich atmospheric blues, and micro-contrast. |
| **Portrait / Interview** | People | Contrast +6, Temp +4, Highlights -10, Shadows +8, Sat -2, Vib +8 | 0.16 | Mid +2.5dB | Flattering healthy skin tones, softened highlights, and subtle focus vignette. Mid EQ boosts vocal presence. |
| **Product** | Commercial | Exposure +0.08, Contrast +12, Highlights -5, Shadows +5, Sat +6 | 0.00 | Flat | High-key clarity, clean neutral whites, and crisp commercial edge definition. |
| **Food** | Commercial | Contrast +10, Temp +10, Highlights -6, Shadows +8, Sat +14, Vib +12 | 0.08 | Flat | Warm, mouthwatering grading highlighting appetizing reds, yellows, and warm tones. |
| **Night / Low-Light** | Ambient | Exposure +0.20, Contrast +8, Highlights -15, Shadows +18, Sat -4 | 0.00 | Flat | Clean shadow lift preserving black level while protecting against sensor noise amplification. |
| **Vintage** | Retro | Contrast +8, Temp +15, Tint +4, Highlights -18, Shadows +16, Sat -12 | 0.18 | Flat | Nostalgic golden fade with warm shadows and gentle analogue softness. |
| **Black & White** | Monochrome | Contrast +24, Highlights +8, Shadows -8, Saturation -100, Vibrance -100 | 0.12 | Flat | High-contrast dramatic monochrome with deep blacks and bright midtones. |

---

## 3. Built-in Photo Presets & Auto Enhance

| Name | Category | Primary Parameters | Focus & Safe Range |
|:---|:---|:---|:---|
| **Natural** | General | Baseline neutral (all 0.0) | Pristine raw pixel pass-through. |
| **Auto Enhance** | Smart | Dynamic analysis | Percentile-bounded exposure, Gray-World balance, highlight recovery. |
| **Portrait** | People | Contrast +5, Temp +5, Highlights -12, Shadows +10, Vib +10, Vignette 0.12 | Flattering skin tones and delicate highlight roll-off. |
| **Landscape** | Nature | Contrast +16, Highlights -16, Shadows +16, Sat +16, Vib +20, Sharp 0.15 | Vibrant skies and foliage without color clipping. |
| **Vivid** | Color | Contrast +20, Highlights -5, Shadows +5, Sat +24, Vib +20 | Saturated commercial pop with rich contrast. |
| **Soft** | Muted | Contrast -12, Highlights -8, Shadows +14, Sat -8, Vib +4 | Low-contrast pastel tones with soft shadows. |
| **Warm** | Temperature | Temp +22, Tint +4, Contrast +8, Highlights -6, Shadows +6 | Golden sunlight feel for golden-hour and outdoor photography. |
| **Cool** | Temperature | Temp -20, Tint -4, Contrast +10, Highlights +4, Shadows -4 | Crisp Nordic light and cool atmospheric tones. |
| **Golden Hour** | Creative | Temp +28, Tint +8, Contrast +12, Highlights -14, Shadows +12, Sat +10 | Rich late-afternoon golden rays with warm highlights. |
| **Night** | Low-Light | Exposure +0.25, Contrast +10, Highlights -20, Shadows +22, Denoise 0.15 | Gentle noise suppression and controlled shadow illumination. |
| **Product** | Commercial | Exposure +0.10, Contrast +14, Highlights -8, Shadows +8, Sat +8 | Accurate color reproduction with pristine whites. |
| **Food** | Culinary | Temp +12, Contrast +10, Highlights -5, Shadows +8, Sat +16 | Enhanced culinary warmth and appetizing tones. |
| **Document / Text** | Utility | Contrast +45, Highlights +25, Shadows -35, Sat -100 | High-contrast black and white for document readability. |
| **Vintage** | Retro | Contrast +8, Temp +14, Tint +6, Highlights -16, Shadows +14, Sat -15 | Classic faded print look with warm amber shadows. |
| **Matte** | Artistic | Contrast +12, Highlights -12, Shadows +24, LevelsBlack 20, Sat -6 | Crushed black lift with filmic matte texture. |
| **Black & White** | Monochrome | Contrast +28, Highlights +10, Shadows -12, Sat -100, Vib -100 | Dramatic monochrome with full tonal range. |

---

## 4. Custom Preset Schema (.irispreset)

All presets are serialized as portable, versioned JSON documents completely independent of platform architecture or binary paths:

```json
{
  "id": "custom_1792571971381",
  "schema_version": 1,
  "name": "My Cinematic Look",
  "category": "Custom",
  "description": "Subtle teal-orange contrast with warm highlights",
  "kind": "LookPreset",
  "media_compatibility": "Universal",
  "adjustments": {
    "exposure": 0.25,
    "contrast": 14.0,
    "temperature": 8.0,
    "tint": -2.0,
    "highlights": -8.0,
    "shadows": 6.0,
    "saturation": 8.0,
    "vibrance": 12.0
  },
  "vignette": 0.12,
  "sharpening": 0.05,
  "clarity": 0.10,
  "grain": 0.00,
  "lut_preset": null,
  "lut_intensity": 0.0,
  "levels": null,
  "audio_eq_low_gain": null,
  "audio_eq_mid_gain": null,
  "audio_eq_high_gain": null,
  "color_space_assumptions": "Rec.709 / Linear sRGB",
  "supported_intensity_behavior": "Linear",
  "is_builtin": false,
  "is_favorite": false,
  "created_at": "2026-10-21T14:00:00Z",
  "modified_at": "2026-10-21T14:00:00Z"
}
```

### Untrusted Validation Rules
1. `name`: Must be 1-64 characters, no script tags (`<script>`) or null bytes (`\0`).
2. `schema_version`: Must equal `CURRENT_SCHEMA_VERSION` (1). Future versions invoke migration handlers.
3. `adjustments`: Checked against strict hard limits (exposure `[-10.0, +10.0]`, contrast `[-150.0, +150.0]`).
4. Values outside safe limits or unrecognized keys reject import with actionable user feedback.
