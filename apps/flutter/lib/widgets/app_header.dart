import 'package:flutter/material.dart';
import '../bridge/iris_engine_controller.dart';
import '../theme/iris_theme.dart';

class AppHeader extends StatelessWidget {
  final IrisEngineController controller;
  final VoidCallback onOpenCommandPalette;

  const AppHeader({
    super.key,
    required this.controller,
    required this.onOpenCommandPalette,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      height: 48,
      decoration: const BoxDecoration(
        color: IrisTheme.surfaceDark,
        border: Border(bottom: BorderSide(color: IrisTheme.border, width: 1)),
      ),
      padding: const EdgeInsets.symmetric(horizontal: 12),
      child: Row(
        children: [
          // Studio Logo
          Row(
            children: [
              Container(
                width: 24,
                height: 24,
                decoration: BoxDecoration(
                  gradient: const LinearGradient(
                    colors: [IrisTheme.primaryCyan, IrisTheme.accentPurple],
                    begin: Alignment.topLeft,
                    end: Alignment.bottomRight,
                  ),
                  borderRadius: BorderRadius.circular(6),
                ),
                child: const Center(
                  child: Icon(Icons.blur_on, size: 18, color: Colors.black),
                ),
              ),
              const SizedBox(width: 8),
              const Text(
                'IRIS STUDIO',
                style: TextStyle(
                  color: IrisTheme.textPrimary,
                  fontSize: 12,
                  fontWeight: FontWeight.w700,
                  letterSpacing: 1.1,
                ),
              ),
            ],
          ),

          const SizedBox(width: 16),
          const VerticalDivider(width: 1, indent: 12, endIndent: 12),
          const SizedBox(width: 10),

          // Workspace Switcher Tabs
          Expanded(
            child: SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              child: Row(
                children: [
                  _buildWorkspaceTab(
                    context,
                    key: const ValueKey('tab_home'),
                    title: 'Home',
                    icon: Icons.home_outlined,
                    workspace: ActiveWorkspace.home,
                  ),
                  const SizedBox(width: 4),
                  _buildWorkspaceTab(
                    context,
                    key: const ValueKey('tab_photo'),
                    title: 'Photo',
                    icon: Icons.photo_filter_outlined,
                    workspace: ActiveWorkspace.photo,
                  ),
                  const SizedBox(width: 4),
                  _buildWorkspaceTab(
                    context,
                    key: const ValueKey('tab_video'),
                    title: 'Video',
                    icon: Icons.movie_filter_outlined,
                    workspace: ActiveWorkspace.video,
                  ),
                  const SizedBox(width: 4),
                  _buildWorkspaceTab(
                    context,
                    key: const ValueKey('tab_export'),
                    title: 'Export',
                    icon: Icons.output_outlined,
                    workspace: ActiveWorkspace.export,
                  ),
                ],
              ),
            ),
          ),

          // Right Controls: Quick Search, Undo/Redo, Project Title, Save
          Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              // Search / Command Palette
              IconButton(
                icon: const Icon(Icons.search, size: 16),
                tooltip: 'Command Palette (Ctrl+K)',
                onPressed: onOpenCommandPalette,
                splashRadius: 16,
              ),

              // Undo / Redo
              IconButton(
                icon: const Icon(Icons.undo, size: 16),
                tooltip: 'Undo (Ctrl+Z)',
                onPressed: controller.canUndo ? controller.undo : null,
                splashRadius: 16,
              ),
              IconButton(
                icon: const Icon(Icons.redo, size: 16),
                tooltip: 'Redo (Ctrl+Y)',
                onPressed: controller.canRedo ? controller.redo : null,
                splashRadius: 16,
              ),

              const SizedBox(width: 8),

              // Export Action Button
              ElevatedButton.icon(
                key: const ValueKey('btn_quick_export'),
                icon: const Icon(Icons.bolt, size: 14),
                label: const Text('Export'),
                style: ElevatedButton.styleFrom(
                  backgroundColor: IrisTheme.primaryCyan,
                  foregroundColor: Colors.black,
                  elevation: 0,
                  visualDensity: VisualDensity.compact,
                  padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 0),
                  textStyle: const TextStyle(fontSize: 11, fontWeight: FontWeight.w600),
                ),
                onPressed: () => controller.switchWorkspace(ActiveWorkspace.export),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildWorkspaceTab(
    BuildContext context, {
    required Key key,
    required String title,
    required IconData icon,
    required ActiveWorkspace workspace,
  }) {
    final isSelected = controller.currentWorkspace == workspace;
    return InkWell(
      key: key,
      onTap: () => controller.switchWorkspace(workspace),
      borderRadius: BorderRadius.circular(6),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
        decoration: BoxDecoration(
          color: isSelected ? IrisTheme.surfaceLight : Colors.transparent,
          borderRadius: BorderRadius.circular(6),
          border: isSelected
              ? Border.all(color: IrisTheme.primaryCyan.withValues(alpha: 0.3))
              : null,
        ),
        child: Row(
          children: [
            Icon(
              icon,
              size: 14,
              color: isSelected ? IrisTheme.primaryCyan : IrisTheme.textSecondary,
            ),
            const SizedBox(width: 6),
            Text(
              title,
              style: TextStyle(
                color: isSelected ? IrisTheme.textPrimary : IrisTheme.textSecondary,
                fontSize: 12,
                fontWeight: isSelected ? FontWeight.w600 : FontWeight.w400,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
