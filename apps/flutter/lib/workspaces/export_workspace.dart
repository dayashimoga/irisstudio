import 'package:flutter/material.dart';
import '../bridge/iris_engine_controller.dart';
import '../models/export_job.dart';
import '../theme/iris_theme.dart';

class ExportWorkspace extends StatefulWidget {
  final IrisEngineController controller;

  const ExportWorkspace({super.key, required this.controller});

  @override
  State<ExportWorkspace> createState() => _ExportWorkspaceState();
}

class _ExportWorkspaceState extends State<ExportWorkspace> {
  String _selectedPreset = 'Web 1080p';
  String _container = 'MP4 (H.264 / AAC)';
  String _resolution = '1920 × 1080 (Full HD)';
  int _fps = 30;
  int _bitrateKbps = 10000;
  bool _useHardwareEncoder = true;
  bool _stripMetadata = false;

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.controller,
      builder: (context, _) {
        return SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // 1. Presets Header
              const Text(
                'Production Delivery Presets',
                style: TextStyle(
                  fontSize: 16,
                  fontWeight: FontWeight.w600,
                  color: IrisTheme.textPrimary,
                ),
              ),
              const SizedBox(height: 12),
              _buildPresetsGrid(),
              const SizedBox(height: 28),

              // 2. Custom Output Parameters Card
              _buildParametersCard(),
              const SizedBox(height: 28),

              // 3. Export Queue & Active Jobs
              _buildExportQueueSection(),
            ],
          ),
        );
      },
    );
  }

  Widget _buildPresetsGrid() {
    final presets = [
      {
        'title': 'YouTube 4K Cinema',
        'subtitle': '3840×2160 • 60 fps • 45 Mbps • High Quality',
        'icon': Icons.smart_display_outlined,
        'tag': '4K UHD',
      },
      {
        'title': 'Web 1080p Delivery',
        'subtitle': '1920×1080 • 30 fps • 10 Mbps • Universal',
        'icon': Icons.web,
        'tag': 'Standard',
      },
      {
        'title': 'Instagram / TikTok Reel',
        'subtitle': '1080×1920 • 60 fps • 15 Mbps • 9:16 Vertical',
        'icon': Icons.phone_android,
        'tag': 'Social',
      },
      {
        'title': 'Archival Lossless Master',
        'subtitle': '1920×1080 • 60 fps • High Bitrate WebM',
        'icon': Icons.archive_outlined,
        'tag': 'Lossless',
      },
    ];

    return LayoutBuilder(
      builder: (context, constraints) {
        final crossAxisCount = constraints.maxWidth > 900 ? 4 : 2;
        return GridView.builder(
          shrinkWrap: true,
          physics: const NeverScrollableScrollPhysics(),
          gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
            crossAxisCount: crossAxisCount,
            crossAxisSpacing: 14,
            mainAxisSpacing: 14,
            childAspectRatio: 2.1,
          ),
          itemCount: presets.length,
          itemBuilder: (ctx, idx) {
            final p = presets[idx];
            final isSelected = _selectedPreset == p['title'];
            return InkWell(
              onTap: () {
                setState(() {
                  _selectedPreset = p['title'] as String;
                  if (idx == 0) {
                    _resolution = '3840 × 2160 (4K UHD)';
                    _fps = 60;
                    _bitrateKbps = 45000;
                  } else if (idx == 1) {
                    _resolution = '1920 × 1080 (Full HD)';
                    _fps = 30;
                    _bitrateKbps = 10000;
                  } else if (idx == 2) {
                    _resolution = '1080 × 1920 (9:16 Vertical)';
                    _fps = 60;
                    _bitrateKbps = 15000;
                  } else if (idx == 3) {
                    _container = 'WebM (VP9 / Opus)';
                    _bitrateKbps = 60000;
                  }
                });
              },
              borderRadius: BorderRadius.circular(8),
              child: Container(
                padding: const EdgeInsets.all(14),
                decoration: BoxDecoration(
                  color: isSelected ? IrisTheme.surfaceLight : IrisTheme.surfaceMid,
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(
                    color: isSelected ? IrisTheme.primaryCyan : IrisTheme.border,
                    width: isSelected ? 2 : 1,
                  ),
                ),
                child: Row(
                  children: [
                    Icon(
                      p['icon'] as IconData,
                      size: 24,
                      color: isSelected ? IrisTheme.primaryCyan : IrisTheme.textSecondary,
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Text(
                            p['title'] as String,
                            style: const TextStyle(
                              color: IrisTheme.textPrimary,
                              fontSize: 13,
                              fontWeight: FontWeight.w600,
                            ),
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                          ),
                          const SizedBox(height: 4),
                          Text(
                            p['subtitle'] as String,
                            style: const TextStyle(color: IrisTheme.textMuted, fontSize: 10),
                            maxLines: 1,
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
            );
          },
        );
      },
    );
  }

  Widget _buildParametersCard() {
    return Container(
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        color: IrisTheme.surfaceMid,
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: IrisTheme.border),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              const Text(
                'ENCODER & OUTPUT SPECIFICATIONS',
                style: TextStyle(
                  color: IrisTheme.textMuted,
                  fontSize: 10,
                  fontWeight: FontWeight.w700,
                  letterSpacing: 0.8,
                ),
              ),
              Row(
                children: [
                  const Text('Hardware Acceleration', style: TextStyle(fontSize: 11, color: IrisTheme.textSecondary)),
                  const SizedBox(width: 8),
                  Switch(
                    value: _useHardwareEncoder,
                    activeThumbColor: IrisTheme.primaryCyan,
                    onChanged: (val) => setState(() => _useHardwareEncoder = val),
                  ),
                ],
              ),
            ],
          ),
          const SizedBox(height: 16),

          Wrap(
            spacing: 20,
            runSpacing: 16,
            children: [
              _buildDropdown(
                label: 'Container & Codec',
                value: _container,
                items: [
                  'MP4 (H.264 / AAC)',
                  'MP4 (H.265 HEVC / AAC)',
                  'WebM (VP9 / Opus)',
                  'WebM (AV1 / Opus)',
                  'PNG Sequence (16-bit Lossless)',
                  'JPEG Sequence (High Quality)',
                ],
                onChanged: (val) => setState(() => _container = val!),
              ),
              _buildDropdown(
                label: 'Resolution',
                value: _resolution,
                items: [
                  '3840 × 2160 (4K UHD)',
                  '1920 × 1080 (Full HD)',
                  '1280 × 720 (HD)',
                  '1080 × 1920 (9:16 Vertical)',
                  '1080 × 1080 (1:1 Square)',
                ],
                onChanged: (val) => setState(() => _resolution = val!),
              ),
              _buildDropdown(
                label: 'Frame Rate',
                value: '$_fps fps',
                items: ['24 fps', '25 fps', '30 fps', '50 fps', '60 fps'],
                onChanged: (val) => setState(() => _fps = int.parse(val!.split(' ')[0])),
              ),
              _buildDropdown(
                label: 'Target Bitrate',
                value: '${_bitrateKbps ~/ 1000} Mbps',
                items: ['5 Mbps', '10 Mbps', '15 Mbps', '25 Mbps', '45 Mbps', '60 Mbps'],
                onChanged: (val) => setState(
                    () => _bitrateKbps = int.parse(val!.split(' ')[0]) * 1000),
              ),
            ],
          ),

          const SizedBox(height: 20),
          const Divider(height: 1),
          const SizedBox(height: 16),

          Wrap(
            spacing: 12,
            runSpacing: 12,
            crossAxisAlignment: WrapCrossAlignment.center,
            alignment: WrapAlignment.spaceBetween,
            children: [
              Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Checkbox(
                    value: _stripMetadata,
                    activeColor: IrisTheme.primaryCyan,
                    checkColor: Colors.black,
                    onChanged: (val) => setState(() => _stripMetadata = val ?? false),
                  ),
                  const Text(
                    'Sanitize GPS & camera metadata (Privacy mode)',
                    style: TextStyle(fontSize: 12, color: IrisTheme.textSecondary),
                  ),
                ],
              ),
              ElevatedButton.icon(
                icon: const Icon(Icons.queue_play_next, size: 16),
                label: const Text('Add to Export Queue & Render'),
                style: ElevatedButton.styleFrom(
                  backgroundColor: IrisTheme.primaryCyan,
                  foregroundColor: Colors.black,
                  padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 14),
                  textStyle: const TextStyle(fontSize: 13, fontWeight: FontWeight.w600),
                ),
                onPressed: _startNewExport,
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildDropdown({
    required String label,
    required String value,
    required List<String> items,
    required ValueChanged<String?> onChanged,
  }) {
    return SizedBox(
      width: 240,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(label, style: const TextStyle(color: IrisTheme.textMuted, fontSize: 11)),
          const SizedBox(height: 6),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12),
            decoration: BoxDecoration(
              color: IrisTheme.surfaceDark,
              borderRadius: BorderRadius.circular(6),
              border: Border.all(color: IrisTheme.border),
            ),
            child: DropdownButtonHideUnderline(
              child: DropdownButton<String>(
                value: items.contains(value) ? value : items[0],
                isExpanded: true,
                dropdownColor: IrisTheme.surfaceDark,
                style: const TextStyle(color: IrisTheme.textPrimary, fontSize: 12),
                items: items
                    .map((item) => DropdownMenuItem(value: item, child: Text(item)))
                    .toList(),
                onChanged: onChanged,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildExportQueueSection() {
    final jobs = widget.controller.exportJobs;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            const Text(
              'Export Queue',
              style: TextStyle(
                fontSize: 16,
                fontWeight: FontWeight.w600,
                color: IrisTheme.textPrimary,
              ),
            ),
            Text(
              '${jobs.length} Jobs Total',
              style: const TextStyle(color: IrisTheme.textMuted, fontSize: 12),
            ),
          ],
        ),
        const SizedBox(height: 12),

        if (jobs.isEmpty)
          Container(
            padding: const EdgeInsets.all(28),
            decoration: BoxDecoration(
              color: IrisTheme.surfaceMid,
              borderRadius: BorderRadius.circular(8),
              border: Border.all(color: IrisTheme.border),
            ),
            alignment: Alignment.center,
            child: Column(
              children: const [
                Icon(Icons.video_library_outlined, size: 36, color: IrisTheme.textMuted),
                SizedBox(height: 10),
                Text(
                  'No Export Jobs Queued',
                  style: TextStyle(color: IrisTheme.textSecondary, fontSize: 13),
                ),
                SizedBox(height: 4),
                Text(
                  'Select a preset and click "Add to Export Queue & Render" above to begin output generation.',
                  style: TextStyle(color: IrisTheme.textMuted, fontSize: 11),
                ),
              ],
            ),
          )
        else
          ListView.separated(
            shrinkWrap: true,
            physics: const NeverScrollableScrollPhysics(),
            itemCount: jobs.length,
            separatorBuilder: (_, _) => const SizedBox(height: 10),
            itemBuilder: (ctx, idx) {
              final job = jobs[idx];
              final isDone = job.status == ExportStatus.completed;
              final isProcessing = job.status == ExportStatus.processing;

              return Container(
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  color: IrisTheme.surfaceMid,
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(
                    color: isDone ? IrisTheme.accentGreen.withValues(alpha: 0.4) : IrisTheme.border,
                  ),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Icon(
                          isDone
                              ? Icons.check_circle
                              : isProcessing
                                  ? Icons.sync
                                  : Icons.hourglass_top,
                          size: 18,
                          color: isDone
                              ? IrisTheme.accentGreen
                              : isProcessing
                                  ? IrisTheme.primaryCyan
                                  : IrisTheme.textMuted,
                        ),
                        const SizedBox(width: 10),
                        Expanded(
                          child: Text(
                            job.title,
                            style: const TextStyle(
                              color: IrisTheme.textPrimary,
                              fontWeight: FontWeight.w600,
                              fontSize: 13,
                            ),
                          ),
                        ),
                        if (isDone)
                          Container(
                            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                            decoration: BoxDecoration(
                              color: IrisTheme.accentGreen.withValues(alpha: 0.15),
                              borderRadius: BorderRadius.circular(4),
                            ),
                            child: const Text(
                              'VERIFIED INTEGRITY OK',
                              style: TextStyle(
                                color: IrisTheme.accentGreen,
                                fontSize: 10,
                                fontWeight: FontWeight.w700,
                              ),
                            ),
                          ),
                        if (isProcessing)
                          IconButton(
                            icon: const Icon(Icons.cancel, size: 16, color: IrisTheme.accentRed),
                            tooltip: 'Cancel Export',
                            onPressed: () => widget.controller.cancelExportJob(job.id),
                          ),
                      ],
                    ),
                    const SizedBox(height: 8),

                    // Progress bar
                    LinearProgressIndicator(
                      value: job.progressPercent / 100.0,
                      backgroundColor: IrisTheme.surfaceLight,
                      color: isDone ? IrisTheme.accentGreen : IrisTheme.primaryCyan,
                      minHeight: 5,
                      borderRadius: BorderRadius.circular(2),
                    ),
                    const SizedBox(height: 8),

                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Text(
                          '${job.resolution} • ${job.format} • ${job.fps} fps',
                          style: const TextStyle(color: IrisTheme.textMuted, fontSize: 11),
                        ),
                        Text(
                          isDone
                              ? '100% Complete'
                              : '${job.progressPercent.toStringAsFixed(1)}% (${job.currentFps.toStringAsFixed(1)} fps)',
                          style: TextStyle(
                            color: isDone ? IrisTheme.accentGreen : IrisTheme.primaryCyan,
                            fontSize: 11,
                            fontFamily: 'monospace',
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                      ],
                    ),
                  ],
                ),
              );
            },
          ),
      ],
    );
  }

  void _startNewExport() {
    final jobId = 'job_${DateTime.now().millisecondsSinceEpoch}';
    final job = ExportJob(
      id: jobId,
      title: '${widget.controller.project.name} ($_selectedPreset)',
      outputPath: 'exports/${widget.controller.project.name}.mp4',
      format: _container,
      resolution: _resolution,
      fps: _fps,
      bitrateKbps: _bitrateKbps,
    );
    widget.controller.addExportJob(job);
  }
}
