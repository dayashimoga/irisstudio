import 'package:flutter/material.dart';
import '../bridge/iris_engine_controller.dart';
import '../bridge/iris_ffi_bindings.dart';
import '../models/recent_project_item.dart';
import '../theme/iris_theme.dart';
import '../widgets/media_picker_dialog.dart';

class HomeWorkspace extends StatefulWidget {
  final IrisEngineController controller;

  const HomeWorkspace({super.key, required this.controller});

  @override
  State<HomeWorkspace> createState() => _HomeWorkspaceState();
}

class _HomeWorkspaceState extends State<HomeWorkspace> {
  late Future<List<RecentProjectItem>> _recentProjectsFuture;

  @override
  void initState() {
    super.initState();
    _refreshRecentProjects();
  }

  void _refreshRecentProjects() {
    setState(() {
      _recentProjectsFuture = RecentProjectItem.loadRecentProjects();
    });
  }

  Future<void> _openProjectFromDisk() async {
    final paths = await MediaPickerDialog.show(
      context,
      title: 'Open Iris Studio Project',
      initialFilter: FileFilterMode.projects,
      allowMultiple: false,
    );

    if (paths != null && paths.isNotEmpty) {
      final ok = await widget.controller.openProjectFile(paths.first);
      if (ok && mounted) {
        widget.controller.switchWorkspace(ActiveWorkspace.video);
      } else if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Failed to open project file: invalid format or corrupt schema')),
        );
      }
    }
  }

  void _showNewProjectDialog() {
    showDialog(
      context: context,
      builder: (ctx) {
        return AlertDialog(
          backgroundColor: IrisTheme.surfaceDark,
          title: const Text(
            'Create New Project',
            style: TextStyle(color: IrisTheme.textPrimary, fontSize: 16),
          ),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              ListTile(
                leading: const Icon(Icons.movie_outlined, color: IrisTheme.primaryCyan),
                title: const Text('New Video Project', style: TextStyle(color: IrisTheme.textPrimary, fontSize: 13)),
                subtitle: const Text('1920×1080 30fps multitrack video + audio sequence (Starts Empty)', style: TextStyle(color: IrisTheme.textMuted, fontSize: 11)),
                onTap: () {
                  Navigator.pop(ctx);
                  widget.controller.createNewProject(name: 'Untitled Video Project', type: 'video');
                  widget.controller.switchWorkspace(ActiveWorkspace.video);
                },
              ),
              ListTile(
                leading: const Icon(Icons.image_outlined, color: IrisTheme.accentPurple),
                title: const Text('New Photo Project', style: TextStyle(color: IrisTheme.textPrimary, fontSize: 13)),
                subtitle: const Text('Nondestructive RAW/JPEG canvas with parametric grading', style: TextStyle(color: IrisTheme.textMuted, fontSize: 11)),
                onTap: () {
                  Navigator.pop(ctx);
                  widget.controller.createNewProject(name: 'Untitled Photo Project', type: 'photo');
                  widget.controller.switchWorkspace(ActiveWorkspace.photo);
                },
              ),
              ListTile(
                leading: const Icon(Icons.audiotrack, color: IrisTheme.accentGreen),
                title: const Text('New Audio-Only Project', style: TextStyle(color: IrisTheme.textPrimary, fontSize: 13)),
                subtitle: const Text('Multichannel audio mixing without dummy video tracks', style: TextStyle(color: IrisTheme.textMuted, fontSize: 11)),
                onTap: () {
                  Navigator.pop(ctx);
                  widget.controller.createNewProject(name: 'Untitled Audio Project', type: 'audio');
                  widget.controller.switchWorkspace(ActiveWorkspace.video);
                },
              ),
              const Divider(color: IrisTheme.border),
              ListTile(
                leading: const Icon(Icons.auto_awesome, color: IrisTheme.accentAmber),
                title: const Text('Load Showcase Demo Project', style: TextStyle(color: IrisTheme.accentAmber, fontSize: 13, fontWeight: FontWeight.w600)),
                subtitle: const Text('Pre-seeds sample 4K and audio clips for feature verification', style: TextStyle(color: IrisTheme.textMuted, fontSize: 11)),
                onTap: () {
                  Navigator.pop(ctx);
                  widget.controller.loadDemoProject();
                  widget.controller.switchWorkspace(ActiveWorkspace.video);
                },
              ),
            ],
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(ctx),
              child: const Text('Cancel'),
            ),
          ],
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 1. Crash Recovery Alert Banner
          if (widget.controller.hasUnrecoveredCrash) _buildCrashRecoveryBanner(context),

          // 2. Welcome & Hero Header
          _buildHeroHeader(context),
          const SizedBox(height: 32),

          // 3. New Project Templates
          const Text(
            'New Project Templates',
            style: TextStyle(
              fontSize: 16,
              fontWeight: FontWeight.w600,
              color: IrisTheme.textPrimary,
            ),
          ),
          const SizedBox(height: 14),
          _buildProjectTemplates(context),
          const SizedBox(height: 36),

          // 4. Recent Projects Section
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              const Text(
                'Recent Projects',
                style: TextStyle(
                  fontSize: 16,
                  fontWeight: FontWeight.w600,
                  color: IrisTheme.textPrimary,
                ),
              ),
              Row(
                children: [
                  TextButton.icon(
                    icon: const Icon(Icons.refresh, size: 14),
                    label: const Text('Refresh'),
                    style: TextButton.styleFrom(foregroundColor: IrisTheme.textSecondary),
                    onPressed: _refreshRecentProjects,
                  ),
                  const SizedBox(width: 8),
                  TextButton.icon(
                    icon: const Icon(Icons.folder_open, size: 16),
                    label: const Text('Open From Disk...'),
                    style: TextButton.styleFrom(foregroundColor: IrisTheme.primaryCyan),
                    onPressed: _openProjectFromDisk,
                  ),
                ],
              ),
            ],
          ),
          const SizedBox(height: 14),
          _buildRecentProjectsGrid(context),
          const SizedBox(height: 36),

          // 5. System Diagnostics & Hardware Acceleration Card
          _buildHardwareDiagnosticsCard(context),
        ],
      ),
    );
  }

  Widget _buildCrashRecoveryBanner(BuildContext context) {
    return Container(
      margin: const EdgeInsets.only(bottom: 24),
      padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 16),
      decoration: BoxDecoration(
        color: IrisTheme.accentAmber.withValues(alpha: 0.12),
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: IrisTheme.accentAmber.withValues(alpha: 0.5)),
      ),
      child: Row(
        children: [
          const Icon(Icons.warning_amber_rounded, color: IrisTheme.accentAmber, size: 28),
          const SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: const [
                Text(
                  'Crash Recovery Checkpoint Found',
                  style: TextStyle(
                    color: IrisTheme.textPrimary,
                    fontSize: 14,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                SizedBox(height: 4),
                Text(
                  'Iris Studio detected an uncommitted editing session in the Write-Ahead-Log (.iris-journal). You can restore all unsaved operations.',
                  style: TextStyle(color: IrisTheme.textSecondary, fontSize: 12),
                ),
              ],
            ),
          ),
          const SizedBox(width: 16),
          OutlinedButton(
            style: OutlinedButton.styleFrom(
              foregroundColor: IrisTheme.textSecondary,
              side: const BorderSide(color: IrisTheme.border),
            ),
            onPressed: widget.controller.dismissCrashRecovery,
            child: const Text('Discard'),
          ),
          const SizedBox(width: 10),
          ElevatedButton.icon(
            icon: const Icon(Icons.restore, size: 16),
            label: const Text('Restore Session'),
            style: ElevatedButton.styleFrom(
              backgroundColor: IrisTheme.accentAmber,
              foregroundColor: Colors.black,
            ),
            onPressed: widget.controller.restoreCrashSession,
          ),
        ],
      ),
    );
  }

  Widget _buildHeroHeader(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(24),
      decoration: BoxDecoration(
        gradient: LinearGradient(
          colors: [
            IrisTheme.surfaceDark,
            IrisTheme.surfaceMid,
            IrisTheme.surfaceLight.withValues(alpha: 0.4),
          ],
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
        ),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: IrisTheme.border),
      ),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Wrap(
                  spacing: 10,
                  runSpacing: 6,
                  crossAxisAlignment: WrapCrossAlignment.center,
                  children: [
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                      decoration: BoxDecoration(
                        color: IrisTheme.primaryCyan.withValues(alpha: 0.15),
                        borderRadius: BorderRadius.circular(4),
                        border: Border.all(color: IrisTheme.primaryCyan.withValues(alpha: 0.3)),
                      ),
                      child: const Text(
                        'OFFLINE STUDIO WORKSTATION',
                        style: TextStyle(
                          color: IrisTheme.primaryCyan,
                          fontSize: 10,
                          fontWeight: FontWeight.w700,
                          letterSpacing: 1.0,
                        ),
                      ),
                    ),
                    Text(
                      IrisFfiBindings.engineVersion,
                      style: const TextStyle(color: IrisTheme.textMuted, fontSize: 11),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ],
                ),
                const SizedBox(height: 12),
                const Text(
                  'Welcome to Iris Studio',
                  style: TextStyle(
                    fontSize: 24,
                    fontWeight: FontWeight.w700,
                    color: IrisTheme.textPrimary,
                    letterSpacing: -0.5,
                  ),
                ),
                const SizedBox(height: 8),
                const Text(
                  'Nondestructive photo and multitrack video workstation with native hardware acceleration, sub-50ms preview latency, zero cloud requirements, and private offline persistence.',
                  style: TextStyle(color: IrisTheme.textSecondary, fontSize: 13, height: 1.4),
                ),
              ],
            ),
          ),
          const SizedBox(width: 24),
          ElevatedButton.icon(
            icon: const Icon(Icons.add, size: 18),
            label: const Text('New Project'),
            style: ElevatedButton.styleFrom(
              backgroundColor: IrisTheme.primaryCyan,
              foregroundColor: Colors.black,
              padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 16),
              textStyle: const TextStyle(fontSize: 14, fontWeight: FontWeight.w600),
            ),
            onPressed: _showNewProjectDialog,
          ),
        ],
      ),
    );
  }

  Widget _buildProjectTemplates(BuildContext context) {
    final templates = [
      {
        'title': 'Video 16:9 Cinema / 4K',
        'subtitle': '3840×2160 • 60 fps • Multitrack',
        'icon': Icons.video_camera_back_outlined,
        'type': 'video',
        'workspace': ActiveWorkspace.video,
      },
      {
        'title': 'Photo Fine-Art Canvas',
        'subtitle': 'High-Res • 16-bit Color • Raw Tiling',
        'icon': Icons.camera_alt_outlined,
        'type': 'photo',
        'workspace': ActiveWorkspace.photo,
      },
      {
        'title': 'Audio-Only Podcast / Mix',
        'subtitle': '48 kHz Stereo • Pure Audio Session',
        'icon': Icons.mic_none_outlined,
        'type': 'audio',
        'workspace': ActiveWorkspace.video,
      },
      {
        'title': 'Mobile 9:16 Shorts / Reels',
        'subtitle': '1080×1920 • 60 fps • Smart Frame',
        'icon': Icons.phone_android_outlined,
        'type': 'video',
        'workspace': ActiveWorkspace.video,
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
            crossAxisSpacing: 16,
            mainAxisSpacing: 16,
            childAspectRatio: 2.2,
          ),
          itemCount: templates.length,
          itemBuilder: (ctx, idx) {
            final t = templates[idx];
            return InkWell(
              onTap: () {
                final type = t['type'] as String;
                final ws = t['workspace'] as ActiveWorkspace;
                widget.controller.createNewProject(name: t['title'] as String, type: type);
                widget.controller.switchWorkspace(ws);
              },
              borderRadius: BorderRadius.circular(10),
              child: Container(
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  color: IrisTheme.surfaceMid,
                  borderRadius: BorderRadius.circular(10),
                  border: Border.all(color: IrisTheme.border),
                ),
                child: Row(
                  children: [
                    Container(
                      padding: const EdgeInsets.all(10),
                      decoration: BoxDecoration(
                        color: IrisTheme.surfaceLight,
                        borderRadius: BorderRadius.circular(8),
                      ),
                      child: Icon(t['icon'] as IconData, color: IrisTheme.primaryCyan, size: 22),
                    ),
                    const SizedBox(width: 14),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Text(
                            t['title'] as String,
                            style: const TextStyle(
                              color: IrisTheme.textPrimary,
                              fontWeight: FontWeight.w600,
                              fontSize: 13,
                            ),
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                          ),
                          const SizedBox(height: 4),
                          Text(
                            t['subtitle'] as String,
                            style: const TextStyle(color: IrisTheme.textMuted, fontSize: 11),
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

  Widget _buildRecentProjectsGrid(BuildContext context) {
    return FutureBuilder<List<RecentProjectItem>>(
      future: _recentProjectsFuture,
      builder: (context, snapshot) {
        if (snapshot.connectionState == ConnectionState.waiting) {
          return const Center(child: Padding(padding: EdgeInsets.all(32), child: CircularProgressIndicator()));
        }

        final items = snapshot.data ?? [];
        if (items.isEmpty) {
          return Container(
            width: double.infinity,
            padding: const EdgeInsets.all(28),
            decoration: BoxDecoration(
              color: IrisTheme.surfaceMid,
              borderRadius: BorderRadius.circular(10),
              border: Border.all(color: IrisTheme.border),
            ),
            child: Column(
              children: [
                const Icon(Icons.folder_open, size: 36, color: IrisTheme.textMuted),
                const SizedBox(height: 12),
                const Text(
                  'No Saved Projects Yet',
                  style: TextStyle(color: IrisTheme.textPrimary, fontSize: 14, fontWeight: FontWeight.w600),
                ),
                const SizedBox(height: 4),
                const Text(
                  'Projects saved to disk will appear here with instant reopening and relinking.',
                  style: TextStyle(color: IrisTheme.textMuted, fontSize: 12),
                ),
                const SizedBox(height: 16),
                Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    OutlinedButton.icon(
                      icon: const Icon(Icons.folder_open, size: 14),
                      label: const Text('Open Project From Disk'),
                      style: OutlinedButton.styleFrom(
                        foregroundColor: IrisTheme.primaryCyan,
                        side: const BorderSide(color: IrisTheme.primaryCyan),
                      ),
                      onPressed: _openProjectFromDisk,
                    ),
                    const SizedBox(width: 12),
                    ElevatedButton.icon(
                      icon: const Icon(Icons.play_circle_outline, size: 14),
                      label: const Text('Explore Showcase Demo'),
                      style: ElevatedButton.styleFrom(
                        backgroundColor: IrisTheme.primaryCyan,
                        foregroundColor: Colors.black,
                      ),
                      onPressed: () {
                        widget.controller.loadDemoProject();
                        widget.controller.switchWorkspace(ActiveWorkspace.video);
                      },
                    ),
                  ],
                ),
              ],
            ),
          );
        }

        return LayoutBuilder(
          builder: (context, constraints) {
            final crossAxisCount = constraints.maxWidth > 900 ? 3 : 1;
            return GridView.builder(
              shrinkWrap: true,
              physics: const NeverScrollableScrollPhysics(),
              gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                crossAxisCount: crossAxisCount,
                crossAxisSpacing: 16,
                mainAxisSpacing: 16,
                childAspectRatio: 1.8,
              ),
              itemCount: items.length,
              itemBuilder: (ctx, idx) {
                final p = items[idx];
                return InkWell(
                  onTap: () async {
                    if (p.isMissing) {
                      ScaffoldMessenger.of(context).showSnackBar(
                        SnackBar(content: Text('Cannot open ${p.projectName}: project file is missing from ${p.projectPath}')),
                      );
                      return;
                    }
                    final ok = await widget.controller.openProjectFile(p.projectPath);
                    if (ok && mounted) {
                      widget.controller.switchWorkspace(
                        p.projectType == 'photo' ? ActiveWorkspace.photo : ActiveWorkspace.video,
                      );
                    }
                  },
                  borderRadius: BorderRadius.circular(10),
                  child: Container(
                    padding: const EdgeInsets.all(16),
                    decoration: BoxDecoration(
                      color: IrisTheme.surfaceMid,
                      borderRadius: BorderRadius.circular(10),
                      border: Border.all(
                        color: p.isMissing ? IrisTheme.accentAmber : IrisTheme.border,
                      ),
                    ),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Row(
                          mainAxisAlignment: MainAxisAlignment.spaceBetween,
                          children: [
                            Container(
                              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                              decoration: BoxDecoration(
                                color: (p.projectType == 'photo' ? IrisTheme.accentPurple : IrisTheme.primaryCyan)
                                    .withValues(alpha: 0.15),
                                borderRadius: BorderRadius.circular(4),
                              ),
                              child: Text(
                                p.projectType.toUpperCase(),
                                style: TextStyle(
                                  color: p.projectType == 'photo' ? IrisTheme.accentPurple : IrisTheme.primaryCyan,
                                  fontSize: 10,
                                  fontWeight: FontWeight.bold,
                                ),
                              ),
                            ),
                            if (p.isMissing)
                              const Text(
                                'MISSING ON DISK',
                                style: TextStyle(color: IrisTheme.accentAmber, fontSize: 10, fontWeight: FontWeight.bold),
                              )
                            else
                              Text(
                                '${p.assetCount} assets',
                                style: const TextStyle(color: IrisTheme.textMuted, fontSize: 11),
                              ),
                          ],
                        ),
                        const Spacer(),
                        Text(
                          p.projectName,
                          style: TextStyle(
                            color: p.isMissing ? IrisTheme.accentAmber : IrisTheme.textPrimary,
                            fontSize: 14,
                            fontWeight: FontWeight.w600,
                          ),
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                        const SizedBox(height: 4),
                        Text(
                          p.projectPath,
                          style: const TextStyle(color: IrisTheme.textMuted, fontSize: 10, fontFamily: 'monospace'),
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                        const SizedBox(height: 4),
                        Text(
                          'Modified: ${p.lastModified.toLocal().toString().split(".")[0]}',
                          style: const TextStyle(color: IrisTheme.textSecondary, fontSize: 11),
                        ),
                      ],
                    ),
                  ),
                );
              },
            );
          },
        );
      },
    );
  }

  Widget _buildHardwareDiagnosticsCard(BuildContext context) {
    final hw = widget.controller.hardwareInfo;

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
            children: [
              const Icon(Icons.memory, color: IrisTheme.primaryCyan, size: 20),
              const SizedBox(width: 10),
              const Text(
                'Hardware Acceleration & Memory Diagnostics',
                style: TextStyle(
                  color: IrisTheme.textPrimary,
                  fontSize: 14,
                  fontWeight: FontWeight.w600,
                ),
              ),
              const Spacer(),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                decoration: BoxDecoration(
                  color: hw.isAvailable ? IrisTheme.accentGreen.withValues(alpha: 0.15) : IrisTheme.accentAmber.withValues(alpha: 0.15),
                  borderRadius: BorderRadius.circular(4),
                  border: Border.all(
                    color: hw.isAvailable ? IrisTheme.accentGreen : IrisTheme.accentAmber,
                  ),
                ),
                child: Text(
                  hw.isAvailable ? 'ACTIVE' : 'SOFTWARE FALLBACK',
                  style: TextStyle(
                    color: hw.isAvailable ? IrisTheme.accentGreen : IrisTheme.accentAmber,
                    fontSize: 10,
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: 16),
          Wrap(
            spacing: 24,
            runSpacing: 12,
            children: [
              _buildDiagItem('Detected GPU', hw.deviceName),
              _buildDiagItem('Hardware API', hw.backendName),
              _buildDiagItem('Zero-Copy Surface', hw.supportsZeroCopy ? 'Active (DirectX Surface Interop)' : 'Emulated (CPU Buffers)'),
              _buildDiagItem('Max Stream Limit', '${hw.maxStreams} concurrent streams'),
              _buildDiagItem('Thread Pool', 'Rayon SIMD Multi-Core (Active)'),
              _buildDiagItem('Crash Resilience', 'Write-Ahead-Log Journal (.iris-journal)'),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildDiagItem(String label, String value) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label, style: const TextStyle(color: IrisTheme.textMuted, fontSize: 11)),
        const SizedBox(height: 2),
        Text(
          value,
          style: const TextStyle(
            color: IrisTheme.textSecondary,
            fontSize: 12,
            fontWeight: FontWeight.w500,
          ),
        ),
      ],
    );
  }
}
