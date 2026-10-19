import 'package:flutter/material.dart';
import '../bridge/iris_engine_controller.dart';
import '../bridge/iris_ffi_bindings.dart';
import '../theme/iris_theme.dart';

class HomeWorkspace extends StatelessWidget {
  final IrisEngineController controller;

  const HomeWorkspace({super.key, required this.controller});

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 1. Crash Recovery Alert Banner (if unrecovered journal detected)
          if (controller.hasUnrecoveredCrash) _buildCrashRecoveryBanner(context),

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

          // 4. Recent Projects
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
              TextButton.icon(
                icon: const Icon(Icons.folder_open, size: 16),
                label: const Text('Open From Disk...'),
                style: TextButton.styleFrom(foregroundColor: IrisTheme.primaryCyan),
                onPressed: () {},
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
            onPressed: controller.dismissCrashRecovery,
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
            onPressed: controller.restoreCrashSession,
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
            onPressed: () => controller.switchWorkspace(ActiveWorkspace.video),
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
        'workspace': ActiveWorkspace.video,
      },
      {
        'title': 'Photo Fine-Art Canvas',
        'subtitle': 'High-Res • 16-bit Color • Raw Tiling',
        'icon': Icons.camera_alt_outlined,
        'workspace': ActiveWorkspace.photo,
      },
      {
        'title': 'Mobile 9:16 Shorts / Reels',
        'subtitle': '1080×1920 • 60 fps • Smart Frame',
        'icon': Icons.phone_android_outlined,
        'workspace': ActiveWorkspace.video,
      },
      {
        'title': 'Social 1:1 Square Feed',
        'subtitle': '1080×1080 • Aspect Presets',
        'icon': Icons.crop_square,
        'workspace': ActiveWorkspace.photo,
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
              onTap: () => controller.switchWorkspace(t['workspace'] as ActiveWorkspace),
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
    final recentProjects = [
      {
        'title': 'Golden Hour Coastline Documentary',
        'type': 'Video (16:9)',
        'duration': '02:45.000',
        'modified': 'Today, 11:20 AM',
        'color': IrisTheme.primaryCyan,
      },
      {
        'title': 'Alpine Peak Panorama Grading',
        'type': 'Photo (RAW)',
        'duration': '48 Megapixels',
        'modified': 'Yesterday',
        'color': IrisTheme.accentPurple,
      },
      {
        'title': 'Product Teaser (Social Reel)',
        'type': 'Video (9:16)',
        'duration': '00:30.000',
        'modified': 'Oct 17, 2026',
        'color': IrisTheme.accentGreen,
      },
    ];

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
          itemCount: recentProjects.length,
          itemBuilder: (ctx, idx) {
            final p = recentProjects[idx];
            return InkWell(
              onTap: () => controller.switchWorkspace(ActiveWorkspace.video),
              borderRadius: BorderRadius.circular(10),
              child: Container(
                padding: const EdgeInsets.all(16),
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
                        Container(
                          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                          decoration: BoxDecoration(
                            color: (p['color'] as Color).withValues(alpha: 0.15),
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: Text(
                            p['type'] as String,
                            style: TextStyle(
                              color: p['color'] as Color,
                              fontSize: 10,
                              fontWeight: FontWeight.w600,
                            ),
                          ),
                        ),
                        Text(
                          p['duration'] as String,
                          style: const TextStyle(
                            color: IrisTheme.textMuted,
                            fontSize: 11,
                            fontFamily: 'monospace',
                          ),
                        ),
                      ],
                    ),
                    const Spacer(),
                    Text(
                      p['title'] as String,
                      style: const TextStyle(
                        color: IrisTheme.textPrimary,
                        fontSize: 14,
                        fontWeight: FontWeight.w600,
                      ),
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                    ),
                    const SizedBox(height: 6),
                    Text(
                      'Last modified: ${p['modified']}',
                      style: const TextStyle(color: IrisTheme.textMuted, fontSize: 11),
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

  Widget _buildHardwareDiagnosticsCard(BuildContext context) {
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
            children: const [
              Icon(Icons.memory, color: IrisTheme.primaryCyan, size: 20),
              SizedBox(width: 10),
              Text(
                'Hardware Acceleration & Memory Diagnostics',
                style: TextStyle(
                  color: IrisTheme.textPrimary,
                  fontSize: 14,
                  fontWeight: FontWeight.w600,
                ),
              ),
            ],
          ),
          const SizedBox(height: 16),
          Wrap(
            spacing: 24,
            runSpacing: 12,
            children: [
              _buildDiagItem('Hardware Decoder', 'Windows Media Foundation (Active)'),
              _buildDiagItem('GPU Interop Surface', 'Direct3D 11 Shared Texture Registrar'),
              _buildDiagItem('Cache Memory Budget', '512 MB LRU Bounded (RAM: 118 MB idle)'),
              _buildDiagItem('Thread Pool', 'Rayon SIMD Multi-Core (Parallelism: Active)'),
              _buildDiagItem('Crash Protection', 'Write-Ahead-Log Journaling (.iris-journal)'),
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
