# Iris Studio: User Guide

## Welcome to Iris Studio

Iris Studio is a modern, lightweight, non-destructive photo and video workstation designed to deliver studio-quality media editing without cloud subscriptions, mandatory logins, advertisements, or telemetry.

---

## 1. Workspaces Overview

Iris Studio features four unified workspaces accessible via the top navigation bar or keyboard shortcuts (`Ctrl+1` through `Ctrl+4`):

1. **Home / Projects (`Ctrl+1`)**:
   - Manage your local editing projects.
   - Quick-start templates (Photo Canvas, 16:9 YouTube/Cinema, 9:16 Shorts/Reels, 1:1 Square).
   - Instant project recovery if your previous session was interrupted.
2. **Photo Workspace (`Ctrl+2`)**:
   - Fine-art non-destructive photo editing.
   - Interactive zoom/pan canvas with sub-pixel precision.
   - Crop, straighten, and perspective adjustments.
   - Pro color grading: Exposure, Contrast, White Balance, Highlights/Shadows, Curves, Levels, Sharpening, Blur, and Vignette.
   - Before/After split comparison slider.
3. **Video Workspace (`Ctrl+3`)**:
   - Multitrack timeline editing for video, audio, voiceover, and title cards.
   - Frame-accurate trim handles, razor blade split tool, ripple delete.
   - Live audio waveforms with dBFS stereo peak meters and volume envelopes.
   - Real-time transitions (Cross Dissolve, Wipe, Fade).
4. **Export Workspace (`Ctrl+4`)**:
   - Production delivery presets: YouTube 4K, Web 1080p, Instagram Reels 60fps, Lossless Master.
   - Granular codec, container, bitrate, and resolution settings.
   - Background export queue with live FPS and progress tracking.

---

## 2. Essential Keyboard Shortcuts

| Shortcut | Action | Scope |
| :--- | :--- | :--- |
| `Space` | Play / Pause Preview | Video Workspace |
| `J` / `K` / `L` | Shuttle Reverse / Pause / Fast Forward | Video Workspace |
| `S` | Split Clip at Current Playhead | Video Workspace |
| `Delete` / `Backspace` | Delete Selected Clip | Video Workspace |
| `Shift + Delete` | Ripple Delete (Close Gap) | Video Workspace |
| `Ctrl + Z` | Undo Last Edit | All Workspaces |
| `Ctrl + Shift + Z` / `Ctrl + Y`| Redo Edit | All Workspaces |
| `Ctrl + S` | Save Project (`.iris`) | All Workspaces |
| `Ctrl + E` | Open Export Panel | All Workspaces |
| `\` (Backslash) | Toggle Before / After View | Photo Workspace |
| `C` | Crop Tool | Photo Workspace |
| `0` | Reset Zoom to Fit Canvas | Photo / Video Workspaces |
| `1` | Zoom 100% (1:1 Pixels) | Photo / Video Workspaces |

---

## 3. Video Loading & Timeline Ingest Workflows

Iris Studio provides two distinct, unambiguous workflows for importing and sequencing media:

### 3.1 "Open Video / Edit Video" (Direct Sequencing)
- **Primary Use**: Immediately starting an edit from a video file without manual track setup.
- **Entry Points**: 
  - Home workspace: Click the hero button **"Open Video / Edit"**.
  - Video workspace: Click **"Open Video"** in the playback transport toolbar.
- **Automated Pipeline**:
  1. **Container Probing**: Native ISO-BMFF parser inspects the video container, extracting native dimensions, framerate timescale, duration, and track structures.
  2. **Sequence Creation**: Automatically creates a source-matched sequence (e.g. 1920×1080 @ 30fps) matching source geometry and timebase.
  3. **Track & Clip Population**: Adds linked video and audio tracks, inserts the media clip starting at `00:00:00.000`, links corresponding audio waveforms, and selects the clip.
  4. **First Frame Display**: Decodes and displays the authentic first decodable frame in the Program Monitor (eliminating hardcoded SMPTE color bars).
  5. **Timeline Fit**: Adjusts the timeline zoom to fit the imported clip's full duration.

### 3.2 "Import Media" (Bin Management)
- **Primary Use**: Ingesting B-roll, music, voiceovers, or alternate takes into the Media Pool without disturbing an existing timeline sequence.
- **Behavior**:
  - Adds the asset into the Media Pool bin with real decoded thumbnails and duration badges.
  - Generates real audio waveform peak envelopes in the background.
  - **Does NOT** alter, overwrite, or mutate any clips or tracks already placed on the timeline.
- **Source Preview & Timeline Insertion**:
  - **Selecting a Media Item**: Clicking a thumbnail in the Media Pool switches the monitor to **Source Preview** mode, displaying the asset's source frame without moving the timeline playhead.
  - **Add to Timeline**: Click **"Add to Timeline"** (or use drag-and-drop) to append or insert the selected clip at the current timeline playhead position on the active track.

---

## 4. Video Look Presets & Grading

Iris Studio features an integrated Preset Engine supporting 15 built-in professional cinematic looks and custom user-authored styles:

### 4.1 Applying a Look Preset
1. Open the **Video Workspace** (`Ctrl+3`).
2. In the right Inspector panel, switch to the **PRESETS** tab.
3. Browse presets by category (*All*, *Cinema*, *Style*, *Portrait*, *Creative*, *Custom*), or use the search field.
4. Click any preset card (e.g. **Cinema Warm**, **Sports / Action**, **Film-Inspired**, **Vintage**).
5. The preset is applied atomically as one undoable action (`Ctrl+Z` to revert).

### 4.2 Intensity Blending & Manual Overrides
- **Intensity Slider**: Adjust the effect strength smoothly from `0%` to `100%` using linear interpolation across exposure, contrast, saturation, temperature, tint, and vignette parameters.
- **Manual Overrides**: 
  - Switch to the **INSPECTOR** tab to fine-tune individual sliders (e.g. adjust contrast or exposure).
  - The preset badge immediately reflects your changes by displaying **`[Preset Name] — Modified`**, indicating that settings have deviated from the default preset baseline.
- **Resetting**:
  - Click **"Reset Look"** on the active preset banner to revert all parameters to neutral (`0.0`).

---

## 5. Custom Presets: Creation, Management & Portability

Create, save, and share your own personalized looks across Windows, macOS, Linux, and mobile:

### 5.1 Saving a Custom Preset
1. In the Photo or Video workspace, adjust grading or effect sliders to your desired appearance.
2. Under the Presets panel, click **"Save Current as Preset"** (or the save icon).
3. Enter a preset name, category (e.g., *Cinema*, *Social*, *Custom*), and optional description.
4. Click **"Save Preset"**. Your preset is saved to the local persistent store (`.iris_custom_presets.json`) and appears under the **Custom** category with a custom tag badge.

### 5.2 Managing & Favoriting Custom Presets
- **Favorites**: Click the heart icon on any preset card to mark it as a favorite for quick filtering under the **Favorites** category.
- **Duplicate**: Click the copy icon to duplicate a preset and make variations.
- **Delete**: Click the trash icon to remove custom presets (built-in factory presets are write-protected and cannot be deleted).

### 5.3 Offline Export & Import (`.irispreset`)
- **Exporting**: Click the export icon on any custom preset card to export a self-contained, offline JSON file with the `.irispreset` extension. No cloud connection, account, or private metadata is included.
- **Importing**: Click **"Import Preset"** at the top of the Presets panel and select an `.irispreset` file. The engine validates schema version, value ranges, and compatibility before importing.

---

## 6. Photo Workspace: Presets & Intelligent Auto Enhance

The Photo Workspace features 16 built-in presets and a bounded content-analysis engine:

### 6.1 Bounded Auto Enhance
- Click the **"Auto Enhance"** banner at the top of the Photo Presets panel.
- **Content Analysis**:
  - Computes a 256-bin luminance histogram to measure dark shadows ($P_{0.01}$) and bright highlights ($P_{0.99}$).
  - Applies Gray-World white balance balancing chromatic red and blue channels against green average luminance.
  - Recovers underexposed shadows and clips blown highlights with smooth clamping:
    - Exposure bounded to $\pm 1.25\,\text{EV}$.
    - Contrast bounded to $\pm 0.35$.
    - Temperature bounded to $\pm 25.0\,\text{K}$.
    - Highlights recovery bounded to $-0.45$.
    - Shadows boost bounded to $+0.40$.
- Displays live analysis feedback and allows manual fine-tuning afterward.

