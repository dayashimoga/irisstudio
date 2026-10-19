//! Subtitle and caption data structures, parsers, and exporters.

use iris_core::{IrisError, IrisResult, RationalTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SubtitleAlignment {
    TopCenter,
    Center,
    #[default]
    BottomCenter,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubtitleStyle {
    pub font_size: u32,
    pub text_color: [u8; 4],
    pub bg_color: Option<[u8; 4]>,
    pub alignment: SubtitleAlignment,
    pub margin_bottom: u32,
}

impl Default for SubtitleStyle {
    fn default() -> Self {
        Self {
            font_size: 24,
            text_color: [255, 255, 255, 255],
            bg_color: Some([0, 0, 0, 180]),
            alignment: SubtitleAlignment::BottomCenter,
            margin_bottom: 40,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubtitleItem {
    pub id: String,
    pub start_time: RationalTime,
    pub end_time: RationalTime,
    pub text: String,
    pub style: Option<SubtitleStyle>,
}

impl SubtitleItem {
    pub fn new(
        id: impl Into<String>,
        start_time: RationalTime,
        end_time: RationalTime,
        text: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            start_time,
            end_time,
            text: text.into(),
            style: None,
        }
    }

    pub fn is_active_at(&self, time: RationalTime) -> bool {
        time >= self.start_time && time < self.end_time
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubtitleTrack {
    pub id: String,
    pub name: String,
    pub items: Vec<SubtitleItem>,
    pub default_style: SubtitleStyle,
    pub enabled: bool,
}

impl SubtitleTrack {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            items: Vec::new(),
            default_style: SubtitleStyle::default(),
            enabled: true,
        }
    }

    pub fn add_item(&mut self, item: SubtitleItem) {
        self.items.push(item);
        self.sort_items();
    }

    pub fn sort_items(&mut self) {
        self.items.sort_by_key(|item| item.start_time);
    }

    pub fn items_at(&self, time: RationalTime) -> Vec<&SubtitleItem> {
        if !self.enabled {
            return Vec::new();
        }
        self.items
            .iter()
            .filter(|item| item.is_active_at(time))
            .collect()
    }

    /// Parses a SubRip (.srt) subtitle string.
    pub fn parse_srt(srt_content: &str) -> IrisResult<Self> {
        let mut track = Self::new("srt_track", "Imported Subtitles");
        let normalized = srt_content.replace("\r\n", "\n").replace('\r', "\n");
        let blocks = normalized.split("\n\n");

        let mut item_index = 1;
        for block in blocks {
            let lines: Vec<&str> = block
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .collect();
            if lines.len() < 2 {
                continue;
            }

            // Line 0 is optionally sequence number (e.g. "1"), or timestamp directly
            let (timestamp_line, text_start_idx) = if lines[0].contains("-->") {
                (lines[0], 1)
            } else if lines.len() >= 2 && lines[1].contains("-->") {
                (lines[1], 2)
            } else {
                continue;
            };

            let (start, end) = Self::parse_timestamp_range(timestamp_line)?;
            let text = lines[text_start_idx..].join("\n");

            if !text.is_empty() {
                track.items.push(SubtitleItem::new(
                    format!("sub_{}", item_index),
                    start,
                    end,
                    text,
                ));
                item_index += 1;
            }
        }

        track.sort_items();
        Ok(track)
    }

    /// Serializes subtitle track to SubRip (.srt) format.
    pub fn to_srt(&self) -> String {
        let mut out = String::new();
        for (i, item) in self.items.iter().enumerate() {
            out.push_str(&format!("{}\n", i + 1));
            let start = Self::format_timestamp(item.start_time, ',');
            let end = Self::format_timestamp(item.end_time, ',');
            out.push_str(&format!("{} --> {}\n", start, end));
            out.push_str(&item.text);
            out.push_str("\n\n");
        }
        out
    }

    /// Parses a WebVTT (.vtt) subtitle string.
    pub fn parse_vtt(vtt_content: &str) -> IrisResult<Self> {
        let mut track = Self::new("vtt_track", "Imported WebVTT");
        let normalized = vtt_content.replace("\r\n", "\n").replace('\r', "\n");
        let blocks = normalized.split("\n\n");

        let mut item_index = 1;
        for block in blocks {
            let lines: Vec<&str> = block
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .collect();
            if lines.is_empty() {
                continue;
            }

            // Skip "WEBVTT" header block
            if lines[0].starts_with("WEBVTT") {
                continue;
            }

            let (timestamp_line, text_start_idx) = if lines[0].contains("-->") {
                (lines[0], 1)
            } else if lines.len() >= 2 && lines[1].contains("-->") {
                (lines[1], 2)
            } else {
                continue;
            };

            let (start, end) = Self::parse_timestamp_range(timestamp_line)?;
            let text = lines[text_start_idx..].join("\n");

            if !text.is_empty() {
                track.items.push(SubtitleItem::new(
                    format!("vtt_{}", item_index),
                    start,
                    end,
                    text,
                ));
                item_index += 1;
            }
        }

        track.sort_items();
        Ok(track)
    }

    /// Serializes subtitle track to WebVTT (.vtt) format.
    pub fn to_vtt(&self) -> String {
        let mut out = String::from("WEBVTT\n\n");
        for (i, item) in self.items.iter().enumerate() {
            out.push_str(&format!("{}\n", i + 1));
            let start = Self::format_timestamp(item.start_time, '.');
            let end = Self::format_timestamp(item.end_time, '.');
            out.push_str(&format!("{} --> {}\n", start, end));
            out.push_str(&item.text);
            out.push_str("\n\n");
        }
        out
    }

    fn parse_timestamp_range(range_str: &str) -> IrisResult<(RationalTime, RationalTime)> {
        let parts: Vec<&str> = range_str.split("-->").map(|s| s.trim()).collect();
        if parts.len() != 2 {
            return Err(IrisError::InvalidParameter(format!(
                "Invalid timestamp range format: {}",
                range_str
            )));
        }

        // Handle possible trailing cue settings in WebVTT like "00:00:01.000 --> 00:00:04.000 align:start"
        let end_token = parts[1].split_whitespace().next().unwrap_or(parts[1]);

        let start = Self::parse_single_timestamp(parts[0])?;
        let end = Self::parse_single_timestamp(end_token)?;

        Ok((start, end))
    }

    fn parse_single_timestamp(ts: &str) -> IrisResult<RationalTime> {
        let sanitized = ts.trim().replace(',', ".");
        let parts: Vec<&str> = sanitized.split(':').collect();

        let (hours, mins, secs_str) = match parts.len() {
            3 => (
                parts[0]
                    .parse::<u32>()
                    .map_err(|e| IrisError::InvalidParameter(e.to_string()))?,
                parts[1]
                    .parse::<u32>()
                    .map_err(|e| IrisError::InvalidParameter(e.to_string()))?,
                parts[2],
            ),
            2 => (
                0,
                parts[0]
                    .parse::<u32>()
                    .map_err(|e| IrisError::InvalidParameter(e.to_string()))?,
                parts[1],
            ),
            _ => {
                return Err(IrisError::InvalidParameter(format!(
                    "Unrecognized timestamp format: {}",
                    ts
                )));
            }
        };

        let secs_parts: Vec<&str> = secs_str.split('.').collect();
        let seconds = secs_parts[0]
            .parse::<u32>()
            .map_err(|e| IrisError::InvalidParameter(e.to_string()))?;
        let millis = if secs_parts.len() > 1 {
            let mut ms_str = secs_parts[1].to_string();
            while ms_str.len() < 3 {
                ms_str.push('0');
            }
            ms_str[..3]
                .parse::<u32>()
                .map_err(|e| IrisError::InvalidParameter(e.to_string()))?
        } else {
            0
        };

        let total_ms =
            (hours as i64 * 3600 + mins as i64 * 60 + seconds as i64) * 1000 + millis as i64;
        Ok(RationalTime::new(total_ms, 1000))
    }

    fn format_timestamp(time: RationalTime, separator: char) -> String {
        let total_ms = (time.to_seconds() * 1000.0).round().max(0.0) as i64;
        let hours = total_ms / 3600000;
        let minutes = (total_ms % 3600000) / 60000;
        let seconds = (total_ms % 60000) / 1000;
        let millis = total_ms % 1000;

        format!(
            "{:02}:{:02}:{:02}{}{:03}",
            hours, minutes, seconds, separator, millis
        )
    }
}
