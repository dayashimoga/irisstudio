import 'package:flutter/material.dart';
import '../bridge/iris_engine_controller.dart';
import '../theme/iris_theme.dart';

class CommandPaletteModal extends StatefulWidget {
  final IrisEngineController controller;

  const CommandPaletteModal({super.key, required this.controller});

  @override
  State<CommandPaletteModal> createState() => _CommandPaletteModalState();
}

class _CommandPaletteModalState extends State<CommandPaletteModal> {
  final TextEditingController _searchController = TextEditingController();
  String _query = '';

  final List<Map<String, dynamic>> _commands = [
    {
      'title': 'Switch to Photo Workspace',
      'shortcut': 'Ctrl+2',
      'category': 'Navigation',
      'icon': Icons.photo_filter,
      'action': (IrisEngineController c) => c.switchWorkspace(ActiveWorkspace.photo),
    },
    {
      'title': 'Switch to Video Workspace',
      'shortcut': 'Ctrl+3',
      'category': 'Navigation',
      'icon': Icons.movie_filter,
      'action': (IrisEngineController c) => c.switchWorkspace(ActiveWorkspace.video),
    },
    {
      'title': 'Switch to Export Workspace',
      'shortcut': 'Ctrl+4',
      'category': 'Navigation',
      'icon': Icons.output,
      'action': (IrisEngineController c) => c.switchWorkspace(ActiveWorkspace.export),
    },
    {
      'title': 'Split Clip at Current Playhead',
      'shortcut': 'S',
      'category': 'Timeline',
      'icon': Icons.call_split,
      'action': (IrisEngineController c) => c.splitClipAtPlayhead(),
    },
    {
      'title': 'Ripple Delete Selected Clip',
      'shortcut': 'Shift+Del',
      'category': 'Timeline',
      'icon': Icons.delete_sweep,
      'action': (IrisEngineController c) => c.rippleDeleteSelected(),
    },
    {
      'title': 'Toggle Before / After Split View',
      'shortcut': '\\',
      'category': 'Photo',
      'icon': Icons.compare,
      'action': (IrisEngineController c) => c.toggleBeforeAfterSplit(),
    },
    {
      'title': 'Reset All Adjustments',
      'shortcut': 'Ctrl+R',
      'category': 'Photo',
      'icon': Icons.restore,
      'action': (IrisEngineController c) => c.photoAdjustments.reset(),
    },
    {
      'title': 'Apply Vivid Preset',
      'shortcut': '',
      'category': 'Photo Presets',
      'icon': Icons.color_lens,
      'action': (IrisEngineController c) => c.applyPresetFilter('Vivid Landscape'),
    },
    {
      'title': 'Apply Cinematic Teal/Orange',
      'shortcut': '',
      'category': 'Photo Presets',
      'icon': Icons.movie,
      'action': (IrisEngineController c) => c.applyPresetFilter('Cinematic Teal/Orange'),
    },
  ];

  @override
  Widget build(BuildContext context) {
    final filtered = _commands.where((cmd) {
      final title = (cmd['title'] as String).toLowerCase();
      final cat = (cmd['category'] as String).toLowerCase();
      final q = _query.toLowerCase();
      return title.contains(q) || cat.contains(q);
    }).toList();

    return Dialog(
      backgroundColor: Colors.transparent,
      insetPadding: const EdgeInsets.symmetric(horizontal: 40, vertical: 60),
      child: Container(
        width: 540,
        decoration: BoxDecoration(
          color: IrisTheme.surfaceDark,
          borderRadius: BorderRadius.circular(12),
          border: Border.all(color: IrisTheme.borderHighlight),
          boxShadow: [
            BoxShadow(
              color: Colors.black.withValues(alpha: 0.6),
              blurRadius: 24,
              offset: const Offset(0, 12),
            ),
          ],
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            // Search Input Header
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
              decoration: const BoxDecoration(
                border: Border(bottom: BorderSide(color: IrisTheme.border)),
              ),
              child: Row(
                children: [
                  const Icon(Icons.search, color: IrisTheme.primaryCyan, size: 20),
                  const SizedBox(width: 12),
                  Expanded(
                    child: TextField(
                      controller: _searchController,
                      autofocus: true,
                      style: const TextStyle(color: IrisTheme.textPrimary, fontSize: 14),
                      decoration: const InputDecoration(
                        hintText: 'Type a command, tool, or shortcut...',
                        hintStyle: TextStyle(color: IrisTheme.textMuted),
                        border: InputBorder.none,
                        isDense: true,
                      ),
                      onChanged: (val) => setState(() => _query = val),
                    ),
                  ),
                  IconButton(
                    icon: const Icon(Icons.close, size: 18),
                    onPressed: () => Navigator.of(context).pop(),
                  ),
                ],
              ),
            ),

            // Command Items
            ConstrainedBox(
              constraints: const BoxConstraints(maxHeight: 340),
              child: ListView.separated(
                padding: const EdgeInsets.symmetric(vertical: 6),
                shrinkWrap: true,
                itemCount: filtered.length,
                separatorBuilder: (_, _) => const Divider(height: 1, indent: 48),
                itemBuilder: (ctx, idx) {
                  final cmd = filtered[idx];
                  return ListTile(
                    dense: true,
                    leading: Icon(cmd['icon'] as IconData, size: 18, color: IrisTheme.textSecondary),
                    title: Text(
                      cmd['title'] as String,
                      style: const TextStyle(fontSize: 13, color: IrisTheme.textPrimary),
                    ),
                    subtitle: Text(
                      cmd['category'] as String,
                      style: const TextStyle(fontSize: 10, color: IrisTheme.textMuted),
                    ),
                    trailing: cmd['shortcut'] != ''
                        ? Container(
                            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                            decoration: BoxDecoration(
                              color: IrisTheme.surfaceLight,
                              borderRadius: BorderRadius.circular(4),
                              border: Border.all(color: IrisTheme.border),
                            ),
                            child: Text(
                              cmd['shortcut'] as String,
                              style: const TextStyle(
                                fontSize: 10,
                                fontFamily: 'monospace',
                                color: IrisTheme.primaryCyan,
                              ),
                            ),
                          )
                        : null,
                    onTap: () {
                      Navigator.of(context).pop();
                      final act = cmd['action'] as Function(IrisEngineController);
                      act(widget.controller);
                    },
                  );
                },
              ),
            ),
          ],
        ),
      ),
    );
  }
}
