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
                          subtitles: controller.project.subtitles,
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
      return _buildMasterInspector(context);
    }
    return _buildClipInspector(context, clip);
  }

  Widget _buildMasterInspector(BuildContext context) {
    final copyPct = controller.streamCopyPercentage;
    final speedup = controller.estimatedSpeedup;

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'PROJECT MASTER & DYNAMICS',
            style: TextStyle(
              color: IrisTheme.textMuted,
              fontSize: 10,
              fontWeight: FontWeight.w700,
              letterSpacing: 0.8,
            ),
          ),
          const SizedBox(height: 12),

          // Stream Copy Optimizer Card
          Container(
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: IrisTheme.surfaceLight,
              borderRadius: BorderRadius.circular(6),
              border: Border.all(
                color: copyPct > 0
                    ? IrisTheme.primaryCyan.withValues(alpha: 0.4)
                    : IrisTheme.border,
              ),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    const Expanded(
                      child: Row(
                        children: [
                          Icon(Icons.bolt, size: 14, color: IrisTheme.primaryCyan),
                          SizedBox(width: 4),
                          Expanded(
                            child: Text(
                              'STREAM COPY',
                              style: TextStyle(
                                fontSize: 10,
                                fontWeight: FontWeight.w700,
                                color: IrisTheme.primaryCyan,
                              ),
                              overflow: TextOverflow.ellipsis,
                            ),
                          ),
                        ],
                      ),
                    ),
                    const SizedBox(width: 6),
                    Text(
                      '${speedup.toStringAsFixed(1)}x Speedup',
                      style: const TextStyle(
                        fontSize: 10,
                        fontWeight: FontWeight.w600,
                        color: IrisTheme.accentGreen,
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 6),
                ClipRRect(
                  borderRadius: BorderRadius.circular(2),
                  child: LinearProgressIndicator(
                    value: copyPct / 100.0,
                    minHeight: 4,
                    backgroundColor: IrisTheme.surfaceMid,
                    valueColor: const AlwaysStoppedAnimation(IrisTheme.primaryCyan),
                  ),
                ),
                const SizedBox(height: 6),
                Text(
                  '${copyPct.toStringAsFixed(1)}% unedited cuts direct stream-copy passthrough.',
                  style: const TextStyle(fontSize: 10, color: IrisTheme.textMuted),
                ),
              ],
            ),
          ),

          const SizedBox(height: 16),
          const Divider(height: 1),
          const SizedBox(height: 16),

          // 3-Band Parametric Equalizer
          const Text(
            '3-BAND PARAMETRIC EQ',
            style: TextStyle(
              color: IrisTheme.textMuted,
              fontSize: 10,
              fontWeight: FontWeight.w700,
              letterSpacing: 0.8,
            ),
          ),
          const SizedBox(height: 8),
          _buildInspectorSlider(
            label: 'Low Shelf (120 Hz)',
            value: controller.project.eq.lowGainDb,
            min: -18.0,
            max: 18.0,
            unit: ' dB',
            onChanged: (val) => controller.setAudioEq(low: val),
          ),
          _buildInspectorSlider(
            label: 'Mid Peaking (1 kHz)',
            value: controller.project.eq.midGainDb,
            min: -18.0,
            max: 18.0,
            unit: ' dB',
            onChanged: (val) => controller.setAudioEq(mid: val),
          ),
          _buildInspectorSlider(
            label: 'High Shelf (8 kHz)',
            value: controller.project.eq.highGainDb,
            min: -18.0,
            max: 18.0,
            unit: ' dB',
            onChanged: (val) => controller.setAudioEq(high: val),
          ),

          const SizedBox(height: 16),
          const Divider(height: 1),
          const SizedBox(height: 16),

          // Dynamics Compressor
          const Text(
            'AUDIO DYNAMICS COMPRESSOR',
            style: TextStyle(
              color: IrisTheme.textMuted,
              fontSize: 10,
              fontWeight: FontWeight.w700,
              letterSpacing: 0.8,
            ),
          ),
          const SizedBox(height: 8),
          _buildInspectorSlider(
            label: 'Threshold',
            value: controller.project.compressor.thresholdDbfs,
            min: -30.0,
            max: 0.0,
            unit: ' dBFS',
            onChanged: (val) => controller.setAudioCompressor(threshold: val),
          ),
          _buildInspectorSlider(
            label: 'Ratio',
            value: controller.project.compressor.ratio,
            min: 1.0,
            max: 20.0,
            unit: ':1',
            onChanged: (val) => controller.setAudioCompressor(ratio: val),
          ),
          _buildInspectorSlider(
            label: 'Makeup Gain',
            value: controller.project.compressor.makeupGainDb,
            min: 0.0,
            max: 24.0,
            unit: ' dB',
            onChanged: (val) => controller.setAudioCompressor(makeup: val),
          ),

          const SizedBox(height: 16),
          const Divider(height: 1),
          const SizedBox(height: 16),

          // Subtitles & Captions Track
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              const Expanded(
                child: Text(
                  'SUBTITLES & CAPTIONS',
                  style: TextStyle(
                    color: IrisTheme.textMuted,
                    fontSize: 10,
                    fontWeight: FontWeight.w700,
                    letterSpacing: 0.8,
                  ),
                ),
              ),
              IconButton(
                icon: const Icon(Icons.add, size: 16, color: IrisTheme.primaryCyan),
                tooltip: 'Add Caption at Playhead',
                splashRadius: 14,
                onPressed: () {
                  controller.addSubtitle(
                    text: 'Caption @ ${controller.playheadSeconds.toStringAsFixed(1)}s',
                  );
                },
              ),
            ],
          ),
          const SizedBox(height: 6),
          Wrap(
            spacing: 6,
            runSpacing: 6,
            children: [
              OutlinedButton.icon(
                icon: const Icon(Icons.file_upload_outlined, size: 12),
                label: const Text('Import SRT', style: TextStyle(fontSize: 10)),
                style: OutlinedButton.styleFrom(
                  visualDensity: VisualDensity.compact,
                  foregroundColor: IrisTheme.textSecondary,
                  side: const BorderSide(color: IrisTheme.border),
                ),
                onPressed: () {
                  controller.importSrt(
                    '1\n00:00:01,000 --> 00:00:04,000\nIris Studio Engine\n\n2\n00:00:05,000 --> 00:00:08,000\nHigh-Performance Video Editing\n\n',
                  );
                },
              ),
            ],
          ),
          const SizedBox(height: 8),

          // Subtitle Items List
          ...controller.project.subtitles.map((sub) {
            return Container(
              margin: const EdgeInsets.only(bottom: 6),
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
              decoration: BoxDecoration(
                color: IrisTheme.surfaceLight,
                borderRadius: BorderRadius.circular(4),
              ),
              child: Row(
                children: [
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          sub.text,
                          style: const TextStyle(
                            fontSize: 11,
                            fontWeight: FontWeight.w500,
                            color: IrisTheme.textPrimary,
                          ),
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                        Text(
                          '${sub.startTime.toStringAsFixed(1)}s - ${sub.endTime.toStringAsFixed(1)}s',
                          style: const TextStyle(
                            fontSize: 9,
                            color: IrisTheme.textMuted,
                          ),
                        ),
                      ],
                    ),
                  ),
                  IconButton(
                    icon: const Icon(Icons.delete_outline, size: 14, color: IrisTheme.textMuted),
                    onPressed: () => controller.removeSubtitle(sub.id),
                    splashRadius: 12,
                  ),
                ],
              ),
            );
          }),
        ],
      ),
    );
  }

  Widget _buildClipInspector(BuildContext context, ClipModel clip) {
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

          // Spatial Transform Section
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

          // Keyframe Easing Section
          const Text(
            'KEYFRAME EASING CURVE',
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
              'Linear',
              'Ease In',
              'Ease Out',
              'Ease In-Out',
              'Cubic Bezier',
            ].map((easing) {
              final isSelected = controller.selectedEasing == easing;
              return ChoiceChip(
                selected: isSelected,
                label: Text(easing, style: const TextStyle(fontSize: 10)),
                backgroundColor: IrisTheme.surfaceLight,
                selectedColor: IrisTheme.primaryCyan.withValues(alpha: 0.25),
                onSelected: (_) => controller.setEasing(easing),
              );
            }).toList(),
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

          // Extended Transitions (10 presets)
          const Text(
            'TRANSITIONS (EXTENDED PRESETS)',
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
              'Cross Dissolve',
              'Iris Round',
              'Zoom In',
              'Zoom Out',
              'Fade to Black',
              'Dip to Color',
              'Wipe Left',
              'Wipe Right',
              'Slide',
              'Push Right',
            ].map((name) => _buildTransitionChip(name, clip)).toList(),
          ),
        ],
      ),
    );
  }

  Widget _buildTransitionChip(String name, ClipModel clip) {
    final hasThisTransition = controller.project.transitions.any(
      (t) => t.kindName == name && (t.clipAId == clip.id || t.clipBId == clip.id),
    );

    return FilterChip(
      selected: hasThisTransition,
      label: Text(name, style: const TextStyle(fontSize: 10)),
      backgroundColor: IrisTheme.surfaceLight,
      selectedColor: IrisTheme.accentAmber.withValues(alpha: 0.3),
      checkmarkColor: IrisTheme.accentAmber,
      onSelected: (_) {
        controller.addTransition(kindName: name, clipAId: clip.id);
      },
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
              Expanded(
                child: Text(
                  label,
                  style: const TextStyle(color: IrisTheme.textSecondary, fontSize: 11),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              const SizedBox(width: 8),
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
                child: ListView(
                  physics: const NeverScrollableScrollPhysics(),
                  children: [
                    ...controller.project.tracks.map((track) {
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
                    }),
                    if (controller.project.subtitles.isNotEmpty)
                      Container(
                        height: 38,
                        decoration: const BoxDecoration(
                          color: Color(0xFF141820),
                          border: Border(bottom: BorderSide(color: IrisTheme.border)),
                        ),
                        padding: const EdgeInsets.symmetric(horizontal: 8),
                        child: const Row(
                          children: [
                            Icon(Icons.subtitles, size: 14, color: IrisTheme.primaryCyan),
                            SizedBox(width: 6),
                            Expanded(
                              child: Text(
                                'SUB Captions',
                                style: TextStyle(fontSize: 10, color: IrisTheme.primaryCyan),
                                overflow: TextOverflow.ellipsis,
                              ),
                            ),
                          ],
                        ),
                      ),
                  ],
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
                        child: ListView(
                          physics: const NeverScrollableScrollPhysics(),
                          children: [
                            ...controller.project.tracks.map((track) {
                              final isVideo = track.kind == TrackKindModel.video;
                              final isAudio = track.kind == TrackKindModel.audio;

                              final clipColor = isVideo
                                  ? IrisTheme.accentPurple
                                  : isAudio
                                      ? IrisTheme.accentGreen
                                      : IrisTheme.primaryCyan;

                              return Container(
                                height: 52,
                                decoration: const BoxDecoration(
                                  border: Border(bottom: BorderSide(color: IrisTheme.border)),
                                ),
                                child: Stack(
                                  children: [
                                    ...track.clips.map((clip) {
                                      final clipLeft = clip.timelineStart * zoomPps;
                                      final clipWidth = clip.duration * zoomPps;
                                      final isSelected = controller.selectedClip?.id == clip.id;

                                      return Positioned(
                                        left: clipLeft,
                                        top: 4,
                                        bottom: 4,
                                        width: clipWidth,
                                        child: GestureDetector(
                                          onTap: () => controller.selectClip(clip),
                                          child: Container(
                                            decoration: BoxDecoration(
                                              color: clipColor.withValues(
                                                alpha: isSelected ? 0.35 : 0.2,
                                              ),
                                              borderRadius: BorderRadius.circular(4),
                                              border: Border.all(
                                                color: isSelected
                                                    ? IrisTheme.primaryCyan
                                                    : clipColor.withValues(alpha: 0.6),
                                                width: isSelected ? 2 : 1,
                                              ),
                                            ),
                                            padding: const EdgeInsets.symmetric(
                                              horizontal: 6,
                                              vertical: 2,
                                            ),
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
                                    }),
                                    if (isVideo)
                                      ...controller.project.transitions.map((trans) {
                                        final tLeft = trans.startTime * zoomPps;
                                        final tWidth = trans.duration * zoomPps;
                                        return Positioned(
                                          left: tLeft,
                                          top: 14,
                                          bottom: 14,
                                          width: tWidth.clamp(20.0, 160.0),
                                          child: Container(
                                            decoration: BoxDecoration(
                                              color: IrisTheme.accentAmber.withValues(alpha: 0.75),
                                              borderRadius: BorderRadius.circular(3),
                                              border: Border.all(color: IrisTheme.accentAmber),
                                            ),
                                            padding: const EdgeInsets.symmetric(horizontal: 4),
                                            alignment: Alignment.center,
                                            child: Text(
                                              '⚡ ${trans.kindName}',
                                              style: const TextStyle(
                                                fontSize: 8,
                                                fontWeight: FontWeight.bold,
                                                color: Colors.black,
                                              ),
                                              maxLines: 1,
                                              overflow: TextOverflow.ellipsis,
                                            ),
                                          ),
                                        );
                                      }),
                                  ],
                                ),
                              );
                            }),
                            if (controller.project.subtitles.isNotEmpty)
                              Container(
                                height: 38,
                                decoration: const BoxDecoration(
                                  color: Color(0xFF141820),
                                  border: Border(bottom: BorderSide(color: IrisTheme.border)),
                                ),
                                child: Stack(
                                  children: controller.project.subtitles.map((sub) {
                                    final sLeft = sub.startTime * zoomPps;
                                    final sWidth = sub.duration * zoomPps;
                                    return Positioned(
                                      left: sLeft,
                                      top: 4,
                                      bottom: 4,
                                      width: sWidth.clamp(24.0, 5000.0),
                                      child: Container(
                                        decoration: BoxDecoration(
                                          color: Colors.blueGrey.withValues(alpha: 0.45),
                                          borderRadius: BorderRadius.circular(3),
                                          border: Border.all(
                                            color: IrisTheme.primaryCyan.withValues(alpha: 0.6),
                                          ),
                                        ),
                                        padding: const EdgeInsets.symmetric(horizontal: 6),
                                        alignment: Alignment.centerLeft,
                                        child: Text(
                                          '💬 ${sub.text}',
                                          style: const TextStyle(
                                            fontSize: 9,
                                            color: Colors.white70,
                                          ),
                                          maxLines: 1,
                                          overflow: TextOverflow.ellipsis,
                                        ),
                                      ),
                                    );
                                  }).toList(),
                                ),
                              ),
                          ],
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
  final List<SubtitleItemModel> subtitles;

  _TimelinePreviewPainter({
    required this.playheadSeconds,
    this.subtitles = const [],
  });

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

    // Render active subtitle overlay if present
    for (final sub in subtitles) {
      if (playheadSeconds >= sub.startTime && playheadSeconds <= sub.endTime) {
        final tp = TextPainter(
          text: TextSpan(
            text: sub.text,
            style: const TextStyle(
              color: Colors.white,
              fontSize: 12,
              fontWeight: FontWeight.w600,
            ),
          ),
          textDirection: TextDirection.ltr,
        )..layout(maxWidth: size.width - 40);

        final bgRect = Rect.fromCenter(
          center: Offset(size.width / 2, size.height - 30),
          width: tp.width + 16,
          height: tp.height + 8,
        );
        canvas.drawRRect(
          RRect.fromRectAndRadius(bgRect, const Radius.circular(4)),
          Paint()..color = Colors.black.withValues(alpha: 0.75),
        );
        tp.paint(
          canvas,
          Offset(size.width / 2 - tp.width / 2, size.height - 30 - tp.height / 2),
        );
      }
    }
  }

  @override
  bool shouldRepaint(covariant _TimelinePreviewPainter oldDelegate) =>
      oldDelegate.playheadSeconds != playheadSeconds ||
      oldDelegate.subtitles != subtitles;
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
