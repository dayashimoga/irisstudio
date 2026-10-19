import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'bridge/iris_engine_controller.dart';
import 'theme/iris_theme.dart';
import 'widgets/app_header.dart';
import 'widgets/command_palette.dart';
import 'workspaces/export_workspace.dart';
import 'workspaces/home_workspace.dart';
import 'workspaces/photo_workspace.dart';
import 'workspaces/video_workspace.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const IrisStudioApp());
}

class IrisStudioApp extends StatelessWidget {
  const IrisStudioApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Iris Studio - Media Workstation',
      debugShowCheckedModeBanner: false,
      theme: IrisTheme.darkTheme,
      home: const IrisStudioShell(),
    );
  }
}

class IrisStudioShell extends StatefulWidget {
  const IrisStudioShell({super.key});

  @override
  State<IrisStudioShell> createState() => _IrisStudioShellState();
}

class _IrisStudioShellState extends State<IrisStudioShell> {
  late final IrisEngineController _controller;
  final FocusNode _focusNode = FocusNode();

  @override
  void initState() {
    super.initState();
    _controller = IrisEngineController();
  }

  @override
  void dispose() {
    _focusNode.dispose();
    _controller.dispose();
    super.dispose();
  }

  void _openCommandPalette() {
    showDialog(
      context: context,
      barrierColor: Colors.black.withValues(alpha: 0.6),
      builder: (ctx) => CommandPaletteModal(controller: _controller),
    );
  }

  KeyEventResult _handleKeyEvent(FocusNode node, KeyEvent event) {
    if (event is! KeyDownEvent) return KeyEventResult.ignored;

    final isCtrl = HardwareKeyboard.instance.isControlPressed ||
        HardwareKeyboard.instance.isMetaPressed;

    if (isCtrl && event.logicalKey == LogicalKeyboardKey.keyK) {
      _openCommandPalette();
      return KeyEventResult.handled;
    }

    if (isCtrl && event.logicalKey == LogicalKeyboardKey.digit1) {
      _controller.switchWorkspace(ActiveWorkspace.home);
      return KeyEventResult.handled;
    }
    if (isCtrl && event.logicalKey == LogicalKeyboardKey.digit2) {
      _controller.switchWorkspace(ActiveWorkspace.photo);
      return KeyEventResult.handled;
    }
    if (isCtrl && event.logicalKey == LogicalKeyboardKey.digit3) {
      _controller.switchWorkspace(ActiveWorkspace.video);
      return KeyEventResult.handled;
    }
    if (isCtrl && event.logicalKey == LogicalKeyboardKey.digit4) {
      _controller.switchWorkspace(ActiveWorkspace.export);
      return KeyEventResult.handled;
    }

    if (isCtrl && event.logicalKey == LogicalKeyboardKey.keyZ) {
      _controller.undo();
      return KeyEventResult.handled;
    }
    if (isCtrl && event.logicalKey == LogicalKeyboardKey.keyY) {
      _controller.redo();
      return KeyEventResult.handled;
    }

    // Space to play/pause in video workspace
    if (event.logicalKey == LogicalKeyboardKey.space &&
        _controller.currentWorkspace == ActiveWorkspace.video) {
      _controller.togglePlayPause();
      return KeyEventResult.handled;
    }

    // S to split clip in video workspace
    if (!isCtrl &&
        event.logicalKey == LogicalKeyboardKey.keyS &&
        _controller.currentWorkspace == ActiveWorkspace.video) {
      _controller.splitClipAtPlayhead();
      return KeyEventResult.handled;
    }

    // \ to toggle before/after in photo workspace
    if (event.logicalKey == LogicalKeyboardKey.backslash &&
        _controller.currentWorkspace == ActiveWorkspace.photo) {
      _controller.toggleBeforeAfterSplit();
      return KeyEventResult.handled;
    }

    return KeyEventResult.ignored;
  }

  @override
  Widget build(BuildContext context) {
    return Focus(
      focusNode: _focusNode,
      autofocus: true,
      onKeyEvent: _handleKeyEvent,
      child: ListenableBuilder(
        listenable: _controller,
        builder: (context, _) {
          return LayoutBuilder(
            builder: (context, constraints) {
              final isMobile = constraints.maxWidth < 650;

              return Scaffold(
                body: SafeArea(
                  child: Column(
                    children: [
                      // Desktop & Tablet Header
                      if (!isMobile)
                        AppHeader(
                          controller: _controller,
                          onOpenCommandPalette: _openCommandPalette,
                        ),

                      // Active Workspace View
                      Expanded(
                        child: AnimatedSwitcher(
                          duration: const Duration(milliseconds: 180),
                          child: _buildCurrentWorkspace(),
                        ),
                      ),
                    ],
                  ),
                ),

                // Mobile Thumb-Friendly Bottom Navigation Bar
                bottomNavigationBar: isMobile
                    ? NavigationBar(
                        selectedIndex: _controller.currentWorkspace.index,
                        backgroundColor: IrisTheme.surfaceDark,
                        indicatorColor: IrisTheme.primaryCyan.withValues(alpha: 0.2),
                        onDestinationSelected: (idx) {
                          _controller.switchWorkspace(ActiveWorkspace.values[idx]);
                        },
                        destinations: const [
                          NavigationDestination(
                            icon: Icon(Icons.home_outlined),
                            selectedIcon: Icon(Icons.home, color: IrisTheme.primaryCyan),
                            label: 'Home',
                          ),
                          NavigationDestination(
                            icon: Icon(Icons.photo_filter_outlined),
                            selectedIcon: Icon(Icons.photo_filter, color: IrisTheme.primaryCyan),
                            label: 'Photo',
                          ),
                          NavigationDestination(
                            icon: Icon(Icons.movie_filter_outlined),
                            selectedIcon: Icon(Icons.movie_filter, color: IrisTheme.primaryCyan),
                            label: 'Video',
                          ),
                          NavigationDestination(
                            icon: Icon(Icons.output_outlined),
                            selectedIcon: Icon(Icons.output, color: IrisTheme.primaryCyan),
                            label: 'Export',
                          ),
                        ],
                      )
                    : null,
              );
            },
          );
        },
      ),
    );
  }

  Widget _buildCurrentWorkspace() {
    switch (_controller.currentWorkspace) {
      case ActiveWorkspace.home:
        return HomeWorkspace(key: const ValueKey('home'), controller: _controller);
      case ActiveWorkspace.photo:
        return PhotoWorkspace(key: const ValueKey('photo'), controller: _controller);
      case ActiveWorkspace.video:
        return VideoWorkspace(key: const ValueKey('video'), controller: _controller);
      case ActiveWorkspace.export:
        return ExportWorkspace(key: const ValueKey('export'), controller: _controller);
    }
  }
}
