use iris_media::VideoFrame;
use serde::{Deserialize, Serialize};

/// Built-in motion graphics title templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TitleTemplate {
    /// Modern minimal title with an animated horizontal underline accent.
    MinimalClean,
    /// Broadcast news lower-third card positioned in the lower third with category tag.
    LowerThirdNews,
    /// Dramatic centered title with top and bottom framing rules.
    CinematicCenter,
    /// Sleek technical callout tag with border brackets.
    CalloutBox,
}

// 8x8 ASCII font bitmap table for printable characters 32..=126
#[rustfmt::skip]
static FONT_8X8: [[u8; 8]; 95] = [
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // 32 (space)
    [0x18, 0x3C, 0x3C, 0x18, 0x18, 0x00, 0x18, 0x00], // 33 !
    [0x66, 0x66, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00], // 34 "
    [0x6C, 0x6C, 0xFE, 0x6C, 0xFE, 0x6C, 0x6C, 0x00], // 35 #
    [0x18, 0x3E, 0x60, 0x3C, 0x06, 0x7C, 0x18, 0x00], // 36 $
    [0x00, 0x66, 0xA6, 0xD4, 0x28, 0x35, 0x66, 0x00], // 37 %
    [0x30, 0x48, 0x30, 0x6E, 0xCC, 0xC6, 0x7A, 0x00], // 38 &
    [0x18, 0x18, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00], // 39 '
    [0x0C, 0x18, 0x30, 0x30, 0x30, 0x18, 0x0C, 0x00], // 40 (
    [0x30, 0x18, 0x0C, 0x0C, 0x0C, 0x18, 0x30, 0x00], // 41 )
    [0x00, 0x66, 0x3C, 0xFF, 0x3C, 0x66, 0x00, 0x00], // 42 *
    [0x00, 0x18, 0x18, 0x7E, 0x18, 0x18, 0x00, 0x00], // 43 +
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x30], // 44 ,
    [0x00, 0x00, 0x00, 0x7E, 0x00, 0x00, 0x00, 0x00], // 45 -
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00], // 46 .
    [0x06, 0x0C, 0x18, 0x30, 0x60, 0xC0, 0x80, 0x00], // 47 /
    [0x3C, 0x66, 0x6E, 0x76, 0x66, 0x66, 0x3C, 0x00], // 48 0
    [0x18, 0x38, 0x18, 0x18, 0x18, 0x18, 0x7E, 0x00], // 49 1
    [0x3C, 0x66, 0x06, 0x1C, 0x30, 0x66, 0x7E, 0x00], // 50 2
    [0x3C, 0x66, 0x06, 0x1C, 0x06, 0x66, 0x3C, 0x00], // 51 3
    [0x0C, 0x1C, 0x3C, 0x6C, 0x7E, 0x0C, 0x0C, 0x00], // 52 4
    [0x7E, 0x60, 0x7C, 0x06, 0x06, 0x66, 0x3C, 0x00], // 53 5
    [0x1C, 0x30, 0x60, 0x7C, 0x66, 0x66, 0x3C, 0x00], // 54 6
    [0x7E, 0x66, 0x0C, 0x18, 0x18, 0x18, 0x18, 0x00], // 55 7
    [0x3C, 0x66, 0x66, 0x3C, 0x66, 0x66, 0x3C, 0x00], // 56 8
    [0x3C, 0x66, 0x66, 0x3E, 0x06, 0x0C, 0x38, 0x00], // 57 9
    [0x00, 0x18, 0x18, 0x00, 0x18, 0x18, 0x00, 0x00], // 58 :
    [0x00, 0x18, 0x18, 0x00, 0x18, 0x18, 0x30, 0x00], // 59 ;
    [0x0C, 0x18, 0x30, 0x60, 0x30, 0x18, 0x0C, 0x00], // 60 <
    [0x00, 0x00, 0x7E, 0x00, 0x7E, 0x00, 0x00, 0x00], // 61 =
    [0x30, 0x18, 0x0C, 0x06, 0x0C, 0x18, 0x30, 0x00], // 62 >
    [0x3C, 0x66, 0x06, 0x0C, 0x18, 0x00, 0x18, 0x00], // 63 ?
    [0x3C, 0x42, 0x99, 0xA5, 0xBD, 0x42, 0x3C, 0x00], // 64 @
    [0x18, 0x3C, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x00], // 65 A
    [0x7C, 0x66, 0x66, 0x7C, 0x66, 0x66, 0x7C, 0x00], // 66 B
    [0x3C, 0x66, 0x60, 0x60, 0x60, 0x66, 0x3C, 0x00], // 67 C
    [0x78, 0x6C, 0x66, 0x66, 0x66, 0x6C, 0x78, 0x00], // 68 D
    [0x7E, 0x60, 0x60, 0x78, 0x60, 0x60, 0x7E, 0x00], // 69 E
    [0x7E, 0x60, 0x60, 0x78, 0x60, 0x60, 0x60, 0x00], // 70 F
    [0x3C, 0x66, 0x60, 0x6E, 0x66, 0x66, 0x3E, 0x00], // 71 G
    [0x66, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x66, 0x00], // 72 H
    [0x3C, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00], // 73 I
    [0x1E, 0x0C, 0x0C, 0x0C, 0x0C, 0x6C, 0x38, 0x00], // 74 J
    [0x66, 0x6C, 0x78, 0x70, 0x78, 0x6C, 0x66, 0x00], // 75 K
    [0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x7E, 0x00], // 76 L
    [0x63, 0x77, 0x7F, 0x6B, 0x63, 0x63, 0x63, 0x00], // 77 M
    [0x66, 0x76, 0x7E, 0x7E, 0x6E, 0x66, 0x66, 0x00], // 78 N
    [0x3C, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00], // 79 O
    [0x7C, 0x66, 0x66, 0x7C, 0x60, 0x60, 0x60, 0x00], // 80 P
    [0x3C, 0x66, 0x66, 0x66, 0x6A, 0x6C, 0x36, 0x00], // 81 Q
    [0x7C, 0x66, 0x66, 0x7C, 0x6C, 0x66, 0x66, 0x00], // 82 R
    [0x3C, 0x66, 0x60, 0x3C, 0x06, 0x66, 0x3C, 0x00], // 83 S
    [0x7E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00], // 84 T
    [0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00], // 85 U
    [0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x00], // 86 V
    [0x63, 0x63, 0x63, 0x6B, 0x7F, 0x77, 0x63, 0x00], // 87 W
    [0x66, 0x66, 0x3C, 0x18, 0x3C, 0x66, 0x66, 0x00], // 88 X
    [0x66, 0x66, 0x66, 0x3C, 0x18, 0x18, 0x18, 0x00], // 89 Y
    [0x7E, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x7E, 0x00], // 90 Z
    [0x3C, 0x30, 0x30, 0x30, 0x30, 0x30, 0x3C, 0x00], // 91 [
    [0xC0, 0x60, 0x30, 0x18, 0x0C, 0x06, 0x02, 0x00], // 92 \
    [0x3C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x3C, 0x00], // 93 ]
    [0x10, 0x38, 0x6C, 0xC6, 0x00, 0x00, 0x00, 0x00], // 94 ^
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF], // 95 _
    [0x30, 0x18, 0x0C, 0x00, 0x00, 0x00, 0x00, 0x00], // 96 `
    [0x00, 0x00, 0x3C, 0x06, 0x3E, 0x66, 0x3B, 0x00], // 97 a
    [0x60, 0x60, 0x7C, 0x66, 0x66, 0x66, 0x7C, 0x00], // 98 b
    [0x00, 0x00, 0x3C, 0x66, 0x60, 0x66, 0x3C, 0x00], // 99 c
    [0x06, 0x06, 0x3E, 0x66, 0x66, 0x66, 0x3B, 0x00], // 100 d
    [0x00, 0x00, 0x3C, 0x66, 0x7E, 0x60, 0x3C, 0x00], // 101 e
    [0x1C, 0x30, 0x78, 0x30, 0x30, 0x30, 0x30, 0x00], // 102 f
    [0x00, 0x00, 0x3B, 0x66, 0x66, 0x3E, 0x06, 0x3C], // 103 g
    [0x60, 0x60, 0x7C, 0x66, 0x66, 0x66, 0x66, 0x00], // 104 h
    [0x18, 0x00, 0x38, 0x18, 0x18, 0x18, 0x3C, 0x00], // 105 i
    [0x06, 0x00, 0x0E, 0x06, 0x06, 0x66, 0x66, 0x3C], // 106 j
    [0x60, 0x60, 0x66, 0x6C, 0x78, 0x6C, 0x66, 0x00], // 107 k
    [0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00], // 108 l
    [0x00, 0x00, 0x66, 0x7F, 0x7F, 0x6B, 0x63, 0x00], // 109 m
    [0x00, 0x00, 0x7C, 0x66, 0x66, 0x66, 0x66, 0x00], // 110 n
    [0x00, 0x00, 0x3C, 0x66, 0x66, 0x66, 0x3C, 0x00], // 111 o
    [0x00, 0x00, 0x7C, 0x66, 0x66, 0x7C, 0x60, 0x60], // 112 p
    [0x00, 0x00, 0x3E, 0x66, 0x66, 0x3E, 0x06, 0x06], // 113 q
    [0x00, 0x00, 0x7C, 0x66, 0x60, 0x60, 0x60, 0x00], // 114 r
    [0x00, 0x00, 0x3E, 0x60, 0x3C, 0x06, 0x7C, 0x00], // 115 s
    [0x18, 0x18, 0x7E, 0x18, 0x18, 0x18, 0x0C, 0x00], // 116 t
    [0x00, 0x00, 0x66, 0x66, 0x66, 0x66, 0x3B, 0x00], // 117 u
    [0x00, 0x00, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x00], // 118 v
    [0x00, 0x00, 0x63, 0x6B, 0x7F, 0x3E, 0x36, 0x00], // 119 w
    [0x00, 0x00, 0x66, 0x3C, 0x18, 0x3C, 0x66, 0x00], // 120 x
    [0x00, 0x00, 0x66, 0x66, 0x66, 0x3E, 0x06, 0x3C], // 121 y
    [0x00, 0x00, 0x7E, 0x0C, 0x18, 0x30, 0x7E, 0x00], // 122 z
    [0x0C, 0x18, 0x18, 0x30, 0x18, 0x18, 0x0C, 0x00], // 123 {
    [0x18, 0x18, 0x18, 0x00, 0x18, 0x18, 0x18, 0x00], // 124 |
    [0x30, 0x18, 0x18, 0x0C, 0x18, 0x18, 0x30, 0x00], // 125 }
    [0x00, 0x00, 0x00, 0x36, 0x5C, 0x00, 0x00, 0x00], // 126 ~
];

/// Renders an animated motion title card onto `VideoFrame`.
/// `progress` is normalized $t \in [0.0, 1.0]$ across the title's duration.
pub fn render_motion_title(
    frame: &mut VideoFrame,
    title: &str,
    subtitle: &str,
    template: TitleTemplate,
    progress: f32,
) {
    let progress = progress.clamp(0.0, 1.0);
    let fw = frame.dimensions.width as isize;
    let fh = frame.dimensions.height as isize;

    // Temporal animation curve (Entry: 0.0 - 0.2, Steady: 0.2 - 0.8, Exit: 0.8 - 1.0)
    let (alpha, offset_y) = if progress < 0.2 {
        let t = progress / 0.2;
        let smooth_t = t * t * (3.0 - 2.0 * t);
        (smooth_t, ((1.0 - smooth_t) * 40.0) as isize)
    } else if progress > 0.8 {
        let t = (1.0 - progress) / 0.2;
        let smooth_t = t * t * (3.0 - 2.0 * t);
        (smooth_t, ((1.0 - smooth_t) * 40.0) as isize)
    } else {
        (1.0, 0isize)
    };

    if alpha <= 0.01 {
        return;
    }

    match template {
        TitleTemplate::LowerThirdNews => {
            render_lower_third_news(frame, title, subtitle, alpha, offset_y, fw, fh);
        }
        TitleTemplate::MinimalClean => {
            render_minimal_clean(frame, title, subtitle, alpha, offset_y, fw, fh);
        }
        TitleTemplate::CinematicCenter => {
            render_cinematic_center(frame, title, subtitle, alpha, offset_y, fw, fh);
        }
        TitleTemplate::CalloutBox => {
            render_callout_box(frame, title, subtitle, alpha, offset_y, fw, fh);
        }
    }
}

fn render_lower_third_news(
    frame: &mut VideoFrame,
    title: &str,
    subtitle: &str,
    alpha: f32,
    offset_y: isize,
    fw: isize,
    fh: isize,
) {
    let scale_title = (fw / 380).clamp(2, 5) as usize;
    let scale_sub = (scale_title * 2) / 3;

    let margin_x = (fw / 16).max(24);
    let margin_y = (fh - fh / 5 + offset_y).clamp(20, fh - 80);

    let t_len = title.len() as isize * (scale_title as isize * 8 + 2);
    let s_len = subtitle.len() as isize * (scale_sub as isize * 8 + 2);
    let box_w = (t_len.max(s_len) + 40).min(fw - margin_x * 2);
    let box_h = (scale_title * 8 + scale_sub * 8 + 28) as isize;

    // Dark glassmorphic background box
    fill_rect_alpha(
        frame,
        margin_x,
        margin_y,
        box_w,
        box_h,
        [15, 18, 26],
        0.85 * alpha,
    );

    // Vibrant Cyan Category Tag Pill on the left edge
    fill_rect_alpha(
        frame,
        margin_x,
        margin_y,
        6,
        box_h,
        [0, 215, 230], // Primary Cyan
        alpha,
    );

    // Title line
    draw_string(
        frame,
        title,
        margin_x + 18,
        margin_y + 10,
        scale_title,
        [245, 248, 252],
        alpha,
    );

    // Subtitle line
    if !subtitle.is_empty() {
        draw_string(
            frame,
            subtitle,
            margin_x + 18,
            margin_y + 14 + (scale_title * 8) as isize,
            scale_sub,
            [150, 165, 185],
            alpha,
        );
    }
}

fn render_minimal_clean(
    frame: &mut VideoFrame,
    title: &str,
    subtitle: &str,
    alpha: f32,
    offset_y: isize,
    fw: isize,
    fh: isize,
) {
    let scale = (fw / 320).clamp(2, 6) as usize;
    let sub_scale = (scale * 2) / 3;

    let start_x = (fw / 12).max(30);
    let start_y = (fh / 4 + offset_y).clamp(20, fh - 100);

    // Main title
    draw_string(
        frame,
        title,
        start_x,
        start_y,
        scale,
        [255, 255, 255],
        alpha,
    );

    // Accent line
    let t_width = title.len() as isize * (scale as isize * 8 + 2);
    let line_y = start_y + (scale * 8) as isize + 8;
    fill_rect_alpha(frame, start_x, line_y, t_width, 3, [0, 215, 230], alpha);

    // Subtitle
    if !subtitle.is_empty() {
        draw_string(
            frame,
            subtitle,
            start_x,
            line_y + 10,
            sub_scale,
            [180, 195, 210],
            alpha,
        );
    }
}

fn render_cinematic_center(
    frame: &mut VideoFrame,
    title: &str,
    subtitle: &str,
    alpha: f32,
    offset_y: isize,
    fw: isize,
    fh: isize,
) {
    let scale = (fw / 280).clamp(3, 7) as usize;
    let sub_scale = (scale * 2) / 3;

    let t_width = title.len() as isize * (scale as isize * 8 + 2);
    let start_x = ((fw - t_width) / 2).max(10);
    let start_y = (fh / 2 - 20 + offset_y).clamp(20, fh - 100);

    // Framing line top
    fill_rect_alpha(
        frame,
        start_x - 20,
        start_y - 12,
        t_width + 40,
        2,
        [210, 180, 120],
        alpha,
    );

    // Main title
    draw_string(
        frame,
        title,
        start_x,
        start_y,
        scale,
        [250, 245, 235],
        alpha,
    );

    // Framing line bottom
    let line_bot_y = start_y + (scale * 8) as isize + 10;
    fill_rect_alpha(
        frame,
        start_x - 20,
        line_bot_y,
        t_width + 40,
        2,
        [210, 180, 120],
        alpha,
    );

    // Subtitle centered below
    if !subtitle.is_empty() {
        let s_width = subtitle.len() as isize * (sub_scale as isize * 8 + 2);
        let s_x = ((fw - s_width) / 2).max(10);
        draw_string(
            frame,
            subtitle,
            s_x,
            line_bot_y + 12,
            sub_scale,
            [180, 175, 160],
            alpha,
        );
    }
}

fn render_callout_box(
    frame: &mut VideoFrame,
    title: &str,
    subtitle: &str,
    alpha: f32,
    offset_y: isize,
    fw: isize,
    fh: isize,
) {
    let scale = (fw / 400).clamp(2, 4) as usize;
    let sub_scale = scale.saturating_sub(1).max(1);

    let box_x = fw - fw / 3;
    let box_y = (fh / 6 + offset_y).clamp(20, fh - 120);
    let box_w = (title.len() as isize * (scale as isize * 8 + 2) + 24).min(fw - box_x - 10);
    let box_h = (scale * 8 + sub_scale * 8 + 24) as isize;

    // Semi-transparent box
    fill_rect_alpha(frame, box_x, box_y, box_w, box_h, [20, 24, 32], 0.8 * alpha);

    // Top-left and bottom-right corner brackets
    fill_rect_alpha(frame, box_x, box_y, 12, 2, [0, 215, 230], alpha);
    fill_rect_alpha(frame, box_x, box_y, 2, 12, [0, 215, 230], alpha);
    fill_rect_alpha(
        frame,
        box_x + box_w - 12,
        box_y + box_h - 2,
        12,
        2,
        [0, 215, 230],
        alpha,
    );
    fill_rect_alpha(
        frame,
        box_x + box_w - 2,
        box_y + box_h - 12,
        2,
        12,
        [0, 215, 230],
        alpha,
    );

    draw_string(
        frame,
        title,
        box_x + 10,
        box_y + 8,
        scale,
        [240, 245, 250],
        alpha,
    );
    if !subtitle.is_empty() {
        draw_string(
            frame,
            subtitle,
            box_x + 10,
            box_y + 10 + (scale * 8) as isize,
            sub_scale,
            [130, 150, 170],
            alpha,
        );
    }
}

fn fill_rect_alpha(
    frame: &mut VideoFrame,
    x: isize,
    y: isize,
    w: isize,
    h: isize,
    rgb: [u8; 3],
    alpha: f32,
) {
    if alpha <= 0.001 {
        return;
    }
    let fw = frame.dimensions.width as isize;
    let fh = frame.dimensions.height as isize;
    let inv_alpha = 1.0 - alpha;

    let x0 = x.clamp(0, fw);
    let x1 = (x + w).clamp(0, fw);
    let y0 = y.clamp(0, fh);
    let y1 = (y + h).clamp(0, fh);

    for ry in y0..y1 {
        let row_start = ry as usize * frame.stride;
        for rx in x0..x1 {
            let offset = row_start + rx as usize * 4;
            frame.data[offset] =
                (frame.data[offset] as f32 * inv_alpha + rgb[0] as f32 * alpha).round() as u8;
            frame.data[offset + 1] =
                (frame.data[offset + 1] as f32 * inv_alpha + rgb[1] as f32 * alpha).round() as u8;
            frame.data[offset + 2] =
                (frame.data[offset + 2] as f32 * inv_alpha + rgb[2] as f32 * alpha).round() as u8;
        }
    }
}

fn draw_string(
    frame: &mut VideoFrame,
    text: &str,
    start_x: isize,
    start_y: isize,
    scale: usize,
    rgb: [u8; 3],
    alpha: f32,
) {
    let mut cursor_x = start_x;
    let fw = frame.dimensions.width as isize;
    let fh = frame.dimensions.height as isize;

    for ch in text.chars() {
        let ascii = ch as usize;
        let glyph_idx = if (32..=126).contains(&ascii) {
            ascii - 32
        } else {
            0
        };
        let bitmap = FONT_8X8[glyph_idx];

        for (row, &byte) in bitmap.iter().enumerate() {
            for col in 0..8 {
                if (byte & (1 << (7 - col))) != 0 {
                    // Draw scaled pixel with subtle drop shadow
                    for sy in 0..scale {
                        for sx in 0..scale {
                            let px = cursor_x + (col * scale + sx) as isize;
                            let py = start_y + (row * scale + sy) as isize;

                            if px >= 0 && px < fw && py >= 0 && py < fh {
                                let offset = py as usize * frame.stride + px as usize * 4;
                                frame.data[offset] = (frame.data[offset] as f32 * (1.0 - alpha)
                                    + rgb[0] as f32 * alpha)
                                    .round()
                                    as u8;
                                frame.data[offset + 1] =
                                    (frame.data[offset + 1] as f32 * (1.0 - alpha)
                                        + rgb[1] as f32 * alpha)
                                        .round() as u8;
                                frame.data[offset + 2] =
                                    (frame.data[offset + 2] as f32 * (1.0 - alpha)
                                        + rgb[2] as f32 * alpha)
                                        .round() as u8;
                            }
                        }
                    }
                }
            }
        }

        cursor_x += (scale * 8 + 2) as isize;
    }
}
