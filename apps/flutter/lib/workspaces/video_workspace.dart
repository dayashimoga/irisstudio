import 'dart:math' as math;
import 'package:flutter/material.dart';
import '../bridge/iris_engine_controller.dart';
import '../models/project_model.dart';
import '../theme/iris_theme.dart';
import '../widgets/audio_meters.dart';
import '../widgets/timecode_display.dart';

class VideoWorkspace extends StatelessWidget {
  final IrisEngineController controller;

  const VideoWorkspace({super.key, required this.controller});

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: controller,
      builder: (context, _) {
        return Column(
          children: [
            // Top Section: Preview Player (Left) + Inspector / Audio Meters (Right)
            Expanded(
              flex: 5,
              child: Row(
                children: [
                  // Player & Transport
                  Expanded(
                    flex: 7,
                    child: _buildPlayerSection(context),
                  ),

                  // Audio VU Meters
                  Padding(
                    padding: const EdgeInsets.symmetric(vertical: 8),
                    child: AudioMeters(
                      leftDbfs: controller.leftDbfs,
                      rightDbfs: controller.rightDbfs,
                      peakLeft: controller.peakLeft,
                      peakRight: controller.peakRight,
                      isVertical: true,
                    ),
                  ),

                  // Clip Inspector Panel
                  Container(
                    width: 290,
                    decoration: const BoxDecoration(
                      color: IrisTheme.surfaceDark,
                      border: Border(left: BorderSide(color: IrisTheme.border)),
                    ),
                    child: _buildInspectorPanel(context),
                  ),
                ],
              ),
            ),

            // Timeline Section (Multitrack Tracks, Playhead, Clips)
            Expanded(
              flex: 4,
              child: Container(
                decoration: const BoxDecoration(
                  color: IrisTheme.background,
                  border: Border(top: BorderSide(color: IrisTheme.border)),
                ),
                child: _buildTimelineSection(context),
              ),
            ),
          ],
        );
      },
    );
  }

  Widget _buildPlayerSection(BuildContext context) {
    return Container(
      color: IrisTheme.background,
      child: Column(
        children: [
          // Viewport Preview Canvas
          Expanded(
            child: Center(
              child: AspectRatio(
                aspectRatio: 16 / 9,
                child: Container(
                  margin: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: Colors.black,
                    borderRadius: BorderRadius.circular(4),
                    border: Border.all(color: IrisTheme.border),
                    boxShadow: [
                      BoxShadow(
                        color: Colors.black.withValues(alpha: 0.6),
                        blurRadius: 16,
                      ),
                    ],
                  ),
                  child: Stack(
                    fit: StackFit.expand,
                    children: [
                      // Video Frame Rendering (Synthetic SMPTE / active clips)
                      CustomPaint(
                        painter: _TimelinePreviewPainter(
                          playheadSeconds: controller.playheadSeconds,
                        ),
                      ),

                      // Timecode Stamp Overlay
                      Positioned(
                        bottom: 12,
                        left: 12,
                        child: TimecodeDisplay(
                          seconds: controller.playheadSeconds,
                          fps: controller.project.fps,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
          ),

          // Transport Toolbar
          Container(
            height: 44,
            decoration: const BoxDecoration(
              color: IrisTheme.surfaceDark,
              border: Border(top: BorderSide(color: IrisTheme.border)),
            ),
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Row(
              children: [
                // Step back 1 frame
                IconButton(
                  icon: const Icon(Icons.skip_previous, size: 18),
                  tooltip: 'Step Back 1 Frame',
                  onPressed: () => controller.stepFrame(-1),
                ),

                // Play / Pause
                IconButton(
                  icon: Icon(
                    controller.isPlaying ? Icons.pause : Icons.play_arrow,
                    size: 22,
                    color: IrisTheme.primaryCyan,
                  ),
                  tooltip: 'Play / Pause (Space)',
                  onPressed: controller.togglePlayPause,
                ),

                // Step forward 1 frame
                IconButton(
                  icon: const Icon(Icons.skip_next, size: 18),
                  tooltip: 'Step Forward 1 Frame',
                  onPressed: () => controller.stepFrame(1),
                ),

                const SizedBox(width: 16),
                const VerticalDivider(width: 1, indent: 10, endIndent: 10),
                const SizedBox(width: 16),

                // Split Clip Tool
                ElevatedButton.icon(
                  icon: const Icon(Icons.call_split, size: 14),
                  label: const Text('Split (S)', style: TextStyle(fontSize: 11)),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: IrisTheme.surfaceLight,
                    foregroundColor: IrisTheme.textPrimary,
                    elevation: 0,
                    visualDensity: VisualDensity.compact,
                  ),
                  onPressed: controller.splitClipAtPlayhead,
                ),

                const SizedBox(width: 8),

                // Ripple Delete
                IconButton(
                  icon: const Icon(Icons.delete_sweep, size: 18),
                  tooltip: 'Ripple Delete (Shift+Del)',
                  onPressed: controller.selectedClip != null
                      ? controller.rippleDeleteSelected
                      : null,
                ),

                const Spacer(),

                // Snapping toggle
                IconButton(
                  icon: Icon(
                    Icons.grid_on,
                    size: 18,
                    color: controller.isSnappingEnabled
                        ? IrisTheme.primaryCyan
                        : IrisTheme.textMuted,
                  ),
                  tooltip: 'Timeline Snapping',
                  onPressed: controller.toggleSnapping,
                ),

                const SizedBox(width: 12),

                // Zoom Slider
                const Icon(Icons.zoom_out, size: 14, color: IrisTheme.textMuted),
                SizedBox(
                  width: 90,
                  child: Slider(
                    value: controller.timelineZoom,
                    min: 0.2,
                    max: 5.0,
                    onChanged: controller.setTimelineZoom,
                  ),
                ),
                const Icon(Icons.zoom_in, size: 14, color: IrisTheme.textMuted),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildInspectorPanel(BuildContext context) {
    final clip = controller.selectedClip;
    if (clip == null) {
      return Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: const [
            Icon(Icons.touch_app, size: 32, color: IrisTheme.textMuted),
            SizedBox(height: 12),
            Text(
              'No Clip Selected',
              style: TextStyle(
                color: IrisTheme.textSecondary,
                fontSize: 13,
                fontWeight: FontWeight.w600,
              ),
            ),
            SizedBox(height: 6),
            Text(
              'Click any clip on the multitrack timeline below to inspect and edit transforms, audio gain, transitions, and keyframes.',
              textAlign: TextAlign.center,
              style: TextStyle(color: IrisTheme.textMuted, fontSize: 11, height: 1.4),
            ),
          ],
        ),
      );
    }

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              const Text(
                'CLIP INSPECTOR',
                style: TextStyle(
                  color: IrisTheme.textMuted,
                  fontSize: 10,
                  fontWeight: FontWeight.w700,
                  letterSpacing: 0.8,
                ),
              ),
              IconButton(
                icon: const Icon(Icons.close, size: 14),
                onPressed: () => controller.selectClip(null),
                splashRadius: 14,
              ),
            ],
          ),
          const SizedBox(height: 8),

          Text(
            clip.name,
            style: const TextStyle(
              color: IrisTheme.textPrimary,
              fontSize: 14,
              fontWeight: FontWeight.w600,
            ),
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
          ),
          const SizedBox(height: 4),
          Text(
            'In: ${clip.timelineStart.toStringAsFixed(2)}s • Dur: ${clip.duration.toStringAsFixed(2)}s',
            style: const TextStyle(color: IrisTheme.textMuted, fontSize: 11),
          ),
          const SizedBox(height: 16),
          const Divider(height: 1),
          const SizedBox(height: 16),

          // Transform Section
          const Text(
            'SPATIAL TRANSFORM',
            style: TextStyle(
              color: IrisTheme.textMuted,
              fontSize: 10,
              fontWeight: FontWeight.w700,
              letterSpacing: 0.8,
            ),
          ),
          const SizedBox(height: 12),

          _buildInspectorSlider(
            label: 'Scale',
            value: clip.scale,
            min: 0.1,
            max: 3.0,
            unit: 'x',
            onChanged: (val) {
              clip.scale = val;
              controller.updateClip(clip);
            },
          ),
          _buildInspectorSlider(
            label: 'Rotation',
            value: clip.rotation,
            min: -180.0,
            max: 180.0,
            unit: '°',
            onChanged: (val) {
              clip.rotation = val;
              controller.updateClip(clip);
            },
          ),
          _buildInspectorSlider(
            label: 'Opacity',
            value: clip.opacity * 100.0,
            min: 0.0,
            max: 100.0,
            unit: '%',
            onChanged: (val) {
              clip.opacity = val / 100.0;
              controller.updateClip(clip);
            },
          ),

          const SizedBox(height: 16),
          const Divider(height: 1),
          const SizedBox(height: 16),

          // Audio Volume Section
          const Text(
            'AUDIO GAIN & ENVELOPE',
            style: TextStyle(
              color: IrisTheme.textMuted,
              fontSize: 10,
              fontWeight: FontWeight.w700,
              letterSpacing: 0.8,
            ),
          ),
          const SizedBox(height: 12),
          _buildInspectorSlider(
            label: 'Volume',
            value: clip.volume * 100.0,
            min: 0.0,
            max: 200.0,
            unit: '%',
            onChanged: (val) {
              clip.volume = val / 100.0;
              controller.updateClip(clip);
            },
          ),

          const SizedBox(height: 16),
          const Divider(height: 1),
          const SizedBox(height: 16),

          // Transitions
          const Text(
            'TRANSITIONS',
            style: TextStyle(
              color: IrisTheme.textMuted,
              fontSize: 10,
              fontWeight: FontWeight.w700,
              letterSpacing: 0.8,
            ),
          ),
          const SizedBox(height: 8),
          Wrap(
            spacing: 6,
            runSpacing: 6,
            children: [
              _buildTransitionChip('Cross Dissolve'),
              _buildTransitionChip('Wipe Left'),
              _buildTransitionChip('Fade to Black'),
              _buildTransitionChip('Slide Right'),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildTransitionChip(String name) {
    return ActionChip(
      label: Text(name, style: const TextStyle(fontSize: 10)),
      backgroundColor: IrisTheme.surfaceLight,
      onPressed: () {},
    );
  }

  Widget _buildInspectorSlider({
    required String label,
    required double value,
    required double min,
    required double max,
    required String unit,
    required ValueChanged<double> onChanged,
  }) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Column(
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(label, style: const TextStyle(color: IrisTheme.textSecondary, fontSize: 11)),
              Text(
                '${value.toStringAsFixed(1)}$unit',
                style: const TextStyle(
                  color: IrisTheme.primaryCyan,
                  fontSize: 11,
                  fontFamily: 'monospace',
                ),
              ),
            ],
          ),
          Slider(
            value: value,
            min: min,
            max: max,
            onChanged: onChanged,
          ),
        ],
      ),
    );
  }

  Widget _buildTimelineSection(BuildContext context) {
    const pps = 60.0; // Pixels per second at 1.0x zoom
    final zoomPps = pps * controller.timelineZoom;
    final totalWidth = (controller.project.totalDuration + 5.0) * zoomPps;

    return Row(
      children: [
        // Track Headers (Left sidebar: Track Names, Solo, Mute)
        Container(
          width: 140,
          decoration: const BoxDecoration(
            color: IrisTheme.surfaceDark,
            border: Border(right: BorderSide(color: IrisTheme.border)),
          ),
          child: Column(
            children: [
              // Ruler spacer header
              Container(
                height: 28,
                decoration: const BoxDecoration(
                  border: Border(bottom: BorderSide(color: IrisTheme.border)),
                ),
                alignment: Alignment.centerLeft,
                padding: const EdgeInsets.symmetric(horizontal: 10),
                child: const Text('TRACKS', style: TextStyle(color: IrisTheme.textMuted, fontSize: 9)),
              ),

              // Tracks List
              Expanded(
                child: ListView.builder(
                  physics: const NeverScrollableScrollPhysics(),
                  itemCount: controller.project.tracks.length,
                  itemBuilder: (ctx, idx) {
                    final track = controller.project.tracks[idx];
                    return Container(
                      height: 52,
                      decoration: const BoxDecoration(
                        border: Border(bottom: BorderSide(color: IrisTheme.border)),
                      ),
                      padding: const EdgeInsets.symmetric(horizontal: 8),
                      child: Row(
                        children: [
                          Icon(
                            track.kind == TrackKindModel.video
                                ? Icons.videocam
                                : track.kind == TrackKindModel.audio
                                    ? Icons.volume_up
                                    : Icons.title,
                            size: 14,
                            color: IrisTheme.textSecondary,
                          ),
                          const SizedBox(width: 6),
                          Expanded(
                            child: Text(
                              track.name,
                              style: const TextStyle(fontSize: 11, color: IrisTheme.textPrimary),
                              overflow: TextOverflow.ellipsis,
                            ),
                          ),
                          IconButton(
                            icon: Icon(
                              track.isMuted ? Icons.volume_off : Icons.volume_up_outlined,
                              size: 14,
                              color: track.isMuted ? IrisTheme.accentRed : IrisTheme.textMuted,
                            ),
                            tooltip: 'Mute',
                            splashRadius: 12,
                            onPressed: () => controller.toggleTrackMute(track),
                          ),
                        ],
                      ),
                    );
                  },
                ),
              ),
            ],
          ),
        ),

        // Scrollable Timeline Tracks & Ruler
        Expanded(
          child: SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            child: SizedBox(
              width: totalWidth.clamp(600.0, 10000.0),
              child: Stack(
                children: [
                  Column(
                    children: [
                      // Timeline Ruler
                      Container(
                        height: 28,
                        decoration: const BoxDecoration(
                          color: IrisTheme.surfaceMid,
                          border: Border(bottom: BorderSide(color: IrisTheme.border)),
                        ),
                        child: CustomPaint(
                          painter: _TimelineRulerPainter(zoomPps: zoomPps),
                          size: Size(totalWidth, 28),
                        ),
                      ),

                      // Tracks
                      Expanded(
                        child: ListView.builder(
                          physics: const NeverScrollableScrollPhysics(),
                          itemCount: controller.project.tracks.length,
                          itemBuilder: (ctx, idx) {
                            final track = controller.project.tracks[idx];
                            return Container(
                              height: 52,
                              decoration: const BoxDecoration(
                                border: Border(bottom: BorderSide(color: IrisTheme.border)),
                              ),
                              child: Stack(
                                children: track.clips.map((clip) {
                                  final clipLeft = clip.timelineStart * zoomPps;
                                  final clipWidth = clip.duration * zoomPps;
                                  final isSelected = controller.selectedClip?.id == clip.id;

                                  final isVideo = track.kind == TrackKindModel.video;
                                  final isAudio = track.kind == TrackKindModel.audio;

                                  final clipColor = isVideo
                                      ? IrisTheme.accentPurple
                                      : isAudio
                                          ? IrisTheme.accentGreen
                                          : IrisTheme.primaryCyan;

                                  return Positioned(
                                    left: clipLeft,
                                    top: 4,
                                    bottom: 4,
                                    width: clipWidth,
                                    child: GestureDetector(
                                      onTap: () => controller.selectClip(clip),
                                      child: Container(
                                        decoration: BoxDecoration(
                                          color: clipColor.withValues(alpha: isSelected ? 0.35 : 0.2),
                                          borderRadius: BorderRadius.circular(4),
                                          border: Border.all(
                                            color: isSelected
                                                ? IrisTheme.primaryCyan
                                                : clipColor.withValues(alpha: 0.6),
                                            width: isSelected ? 2 : 1,
                                          ),
                                        ),
                                        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                                        child: Column(
                                          crossAxisAlignment: CrossAxisAlignment.start,
                                          children: [
                                            Text(
                                              clip.name,
                                              style: TextStyle(
                                                fontSize: 10,
                                                fontWeight: FontWeight.w600,
                                                color: isSelected
                                                    ? IrisTheme.textPrimary
                                                    : IrisTheme.textSecondary,
                                              ),
                                              maxLines: 1,
                                              overflow: TextOverflow.ellipsis,
                                            ),
                                            const Spacer(),
                                            if (isAudio)
                                              CustomPaint(
                                                painter: _WaveformMiniPainter(),
                                                size: Size(clipWidth - 12, 16),
                                              ),
                                          ],
                                        ),
                                      ),
                                    ),
                                  );
                                }).toList(),
                              ),
                            );
                          },
                        ),
                      ),
                    ],
                  ),

                  // Draggable Playhead Red/Cyan Needle
                  Positioned(
                    left: controller.playheadSeconds * zoomPps - 6,
                    top: 0,
                    bottom: 0,
                    child: GestureDetector(
                      onHorizontalDragUpdate: (details) {
                        final newTime = (controller.playheadSeconds * zoomPps + details.delta.dx) / zoomPps;
                        controller.seekTo(newTime);
                      },
                      child: Column(
                        children: [
                          Container(
                            width: 12,
                            height: 12,
                            decoration: const BoxDecoration(
                              color: IrisTheme.accentRed,
                              shape: BoxShape.circle,
                            ),
                          ),
                          Expanded(
                            child: Container(
                              width: 2,
                              color: IrisTheme.accentRed,
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ],
    );
  }
}

class _TimelinePreviewPainter extends CustomPainter {
  final double playheadSeconds;

  _TimelinePreviewPainter({required this.playheadSeconds});

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint();

    // Draw dynamic SMPTE color bars
    final colors = [
      const Color(0xFFEBEBEB),
      const Color(0xFFDBDB10),
      const Color(0xFF10DBDB),
      const Color(0xFF10DB10),
      const Color(0xFFDB10DB),
      const Color(0xFFDB1010),
      const Color(0xFF1010DB),
    ];

    final barWidth = size.width / colors.length;
    for (int i = 0; i < colors.length; i++) {
      paint.color = colors[i];
      canvas.drawRect(
        Rect.fromLTWH(i * barWidth, 0, barWidth, size.height * 0.75),
        paint,
      );
    }

    // Bottom gradient bar
    paint.color = const Color(0xFF181C24);
    canvas.drawRect(
      Rect.fromLTWH(0, size.height * 0.75, size.width, size.height * 0.25),
      paint,
    );
  }

  @override
  bool shouldRepaint(covariant _TimelinePreviewPainter oldDelegate) =>
      oldDelegate.playheadSeconds != playheadSeconds;
}

class _TimelineRulerPainter extends CustomPainter {
  final double zoomPps;

  _TimelineRulerPainter({required this.zoomPps});

  @override
  void paint(Canvas canvas, Size size) {
    final textPainter = TextPainter(textDirection: TextDirection.ltr);
    final linePaint = Paint()
      ..color = IrisTheme.borderHighlight
      ..strokeWidth = 1.0;

    final seconds = (size.width / zoomPps).ceil();
    for (int s = 0; s <= seconds; s++) {
      final x = s * zoomPps;
      // Second tick
      canvas.drawLine(Offset(x, 16), Offset(x, 28), linePaint);

      textPainter.text = TextSpan(
        text: '${s}s',
        style: const TextStyle(color: IrisTheme.textMuted, fontSize: 9, fontFamily: 'monospace'),
      );
      textPainter.layout();
      textPainter.paint(canvas, Offset(x + 2, 4));
    }
  }

  @override
  bool shouldRepaint(covariant _TimelineRulerPainter oldDelegate) =>
      oldDelegate.zoomPps != zoomPps;
}

class _WaveformMiniPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = IrisTheme.accentGreen.withValues(alpha: 0.8)
      ..strokeWidth = 1.0;

    final step = 4.0;
    final count = (size.width / step).floor();
    for (int i = 0; i < count; i++) {
      final x = i * step;
      final h = (math.sin(i * 0.4).abs() * size.height).clamp(2.0, size.height);
      final y0 = (size.height - h) / 2;
      canvas.drawLine(Offset(x, y0), Offset(x, y0 + h), paint);
    }
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}
