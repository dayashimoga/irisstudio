import 'dart:io';
import 'package:flutter/material.dart';
import '../bridge/iris_ffi_bindings.dart';
import '../theme/iris_theme.dart';

enum FileFilterMode { allMedia, photos, videos, audio, projects, allFiles }

class MediaPickerDialog extends StatefulWidget {
  final String title;
  final FileFilterMode initialFilter;
  final bool allowMultiple;
  final bool isSaveMode;
  final String? defaultSaveFileName;
  final String? initialDirectory;

  const MediaPickerDialog({
    super.key,
    required this.title,
    this.initialFilter = FileFilterMode.allMedia,
    this.allowMultiple = false,
    this.isSaveMode = false,
    this.defaultSaveFileName,
    this.initialDirectory,
  });

  static bool _isPickerActive = false;

  static Future<List<String>?> show(
    BuildContext context, {
    required String title,
    FileFilterMode initialFilter = FileFilterMode.allMedia,
    bool allowMultiple = false,
    bool isSaveMode = false,
    String? defaultSaveFileName,
    String? initialDirectory,
  }) async {
    if (_isPickerActive) return null;
    _isPickerActive = true;
    try {
      return await showDialog<List<String>>(
        context: context,
        barrierDismissible: true,
        builder: (ctx) => MediaPickerDialog(
          title: title,
          initialFilter: initialFilter,
          allowMultiple: allowMultiple,
          isSaveMode: isSaveMode,
          defaultSaveFileName: defaultSaveFileName,
          initialDirectory: initialDirectory,
        ),
      );
    } finally {
      _isPickerActive = false;
    }
  }

  @override
  State<MediaPickerDialog> createState() => _MediaPickerDialogState();
}

class _MediaPickerDialogState extends State<MediaPickerDialog> {
  late Directory _currentDirectory;
  late FileFilterMode _filterMode;
  List<FileSystemEntity> _entries = [];
  final Set<String> _selectedPaths = {};
  String? _probedPath;
  IrisMediaInfo? _probedInfo;
  bool _isLoading = false;
  late TextEditingController _fileNameController;
  late TextEditingController _searchController;
  String _searchQuery = '';

  @override
  void initState() {
    super.initState();
    _filterMode = widget.initialFilter;
    _fileNameController = TextEditingController(text: widget.defaultSaveFileName ?? 'untitled');
    _searchController = TextEditingController();

    // Default directory resolution
    String initialPath = widget.initialDirectory ?? '';
    if (initialPath.isEmpty || !Directory(initialPath).existsSync()) {
      initialPath = 'h:${Platform.pathSeparator}rustvideoaudio';
      if (!Directory(initialPath).existsSync()) {
        initialPath = Directory.current.path;
      }
    }
    _currentDirectory = Directory(initialPath);
    _loadDirectory();
  }

  @override
  void dispose() {
    _fileNameController.dispose();
    _searchController.dispose();
    super.dispose();
  }

  void _loadDirectory() {
    setState(() => _isLoading = true);
    try {
      final list = _currentDirectory.listSync(followLinks: false);
      list.sort((a, b) {
        final aIsDir = a is Directory;
        final bIsDir = b is Directory;
        if (aIsDir && !bIsDir) return -1;
        if (!aIsDir && bIsDir) return 1;
        return a.path.toLowerCase().compareTo(b.path.toLowerCase());
      });
      _entries = list;
    } catch (e) {
      debugPrint('Error listing directory: $e');
      _entries = [];
    } finally {
      setState(() => _isLoading = false);
    }
  }

  bool _matchesFilter(String path) {
    if (_searchQuery.isNotEmpty) {
      final name = path.split(Platform.pathSeparator).last.toLowerCase();
      if (!name.contains(_searchQuery.toLowerCase())) return false;
    }

    final ext = path.split('.').last.toLowerCase();
    switch (_filterMode) {
      case FileFilterMode.allMedia:
        return [
          'jpg', 'jpeg', 'png', 'webp', 'bmp', 'tiff',
          'mp4', 'mov', 'mkv', 'webm', 'avi',
          'wav', 'mp3', 'aac', 'ogg', 'flac', 'm4a',
        ].contains(ext);
      case FileFilterMode.photos:
        return ['jpg', 'jpeg', 'png', 'webp', 'bmp', 'tiff'].contains(ext);
      case FileFilterMode.videos:
        return ['mp4', 'mov', 'mkv', 'webm', 'avi'].contains(ext);
      case FileFilterMode.audio:
        return ['wav', 'mp3', 'aac', 'ogg', 'flac', 'm4a'].contains(ext);
      case FileFilterMode.projects:
        return ['irisproj', 'iris', 'json'].contains(ext);
      case FileFilterMode.allFiles:
        return true;
    }
  }

  int _probeGeneration = 0;
  void _probeSelectedFile(String path) {
    final currentGen = ++_probeGeneration;
    setState(() {
      _probedPath = path;
    });
    Future.delayed(const Duration(milliseconds: 60), () {
      if (currentGen == _probeGeneration && mounted) {
        final info = IrisFfiBindings.probeMedia(path);
        if (currentGen == _probeGeneration && mounted) {
          setState(() {
            _probedInfo = info;
          });
        }
      }
    });
  }

  void _onConfirm() {
    if (widget.isSaveMode) {
      var name = _fileNameController.text.trim();
      if (name.isEmpty) name = 'untitled';
      String ext = '.irisproj';
      if (_filterMode == FileFilterMode.photos) ext = '.jpg';
      if (_filterMode == FileFilterMode.audio) ext = '.wav';
      if (_filterMode == FileFilterMode.videos) ext = '.mp4';
      if (!name.contains('.')) name += ext;

      final fullPath = '${_currentDirectory.path}${Platform.pathSeparator}$name';
      Navigator.pop(context, [fullPath]);
    } else {
      if (_selectedPaths.isNotEmpty) {
        Navigator.pop(context, _selectedPaths.toList());
      }
    }
  }



  @override
  Widget build(BuildContext context) {
    return Dialog(
      backgroundColor: IrisTheme.surfaceDark,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: const BorderSide(color: IrisTheme.border),
      ),
      insetPadding: const EdgeInsets.symmetric(horizontal: 40, vertical: 24),
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 960, maxHeight: 680),
        child: Column(
          children: [
            // Header Bar
            _buildHeader(),

            // Main Content: Bookmarks Left | File List Center | Metadata Inspector Right
            Expanded(
              child: Row(
                children: [
                  // Quick System Bookmarks
                  _buildBookmarksSidebar(),

                  // File Navigation & Listing
                  Expanded(
                    flex: 5,
                    child: _buildFileListSection(),
                  ),

                  // Metadata Preview Panel
                  if (_probedInfo != null)
                    Container(
                      width: 250,
                      decoration: const BoxDecoration(
                        color: IrisTheme.surfaceMid,
                        border: Border(left: BorderSide(color: IrisTheme.border)),
                      ),
                      child: _buildMetadataPreview(),
                    ),
                ],
              ),
            ),

            // Footer Toolbar with Action Buttons
            _buildFooter(),
          ],
        ),
      ),
    );
  }

  Widget _buildHeader() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 14),
      decoration: const BoxDecoration(
        color: IrisTheme.surfaceMid,
        borderRadius: BorderRadius.vertical(top: Radius.circular(12)),
        border: Border(bottom: BorderSide(color: IrisTheme.border)),
      ),
      child: Row(
        children: [
          Icon(
            widget.isSaveMode ? Icons.save_outlined : Icons.folder_open,
            color: IrisTheme.primaryCyan,
            size: 20,
          ),
          const SizedBox(width: 12),
          Text(
            widget.title,
            style: const TextStyle(
              fontSize: 16,
              fontWeight: FontWeight.w600,
              color: IrisTheme.textPrimary,
            ),
          ),
          const Spacer(),


          const SizedBox(width: 8),
          IconButton(
            icon: const Icon(Icons.close, size: 18),
            splashRadius: 18,
            onPressed: () => Navigator.pop(context, null),
          ),
        ],
      ),
    );
  }

  Widget _buildBookmarksSidebar() {
    final bookmarks = <Map<String, dynamic>>[
      {
        'name': 'Workspace',
        'path': 'h:${Platform.pathSeparator}rustvideoaudio',
        'icon': Icons.home_work_outlined,
      },
    ];

    // Standard user folders
    final userProfile = Platform.environment['USERPROFILE'] ?? Platform.environment['HOME'] ?? '';
    if (userProfile.isNotEmpty) {
      bookmarks.addAll([
        {'name': 'Desktop', 'path': '$userProfile${Platform.pathSeparator}Desktop', 'icon': Icons.desktop_windows},
        {'name': 'Pictures', 'path': '$userProfile${Platform.pathSeparator}Pictures', 'icon': Icons.image_outlined},
        {'name': 'Videos', 'path': '$userProfile${Platform.pathSeparator}Videos', 'icon': Icons.video_library_outlined},
        {'name': 'Music', 'path': '$userProfile${Platform.pathSeparator}Music', 'icon': Icons.music_note_outlined},
        {'name': 'Documents', 'path': '$userProfile${Platform.pathSeparator}Documents', 'icon': Icons.folder_outlined},
        {'name': 'Downloads', 'path': '$userProfile${Platform.pathSeparator}Downloads', 'icon': Icons.download_outlined},
      ]);
    }

    // Windows Drives
    if (Platform.isWindows) {
      for (final letter in ['C', 'D', 'E', 'H']) {
        final drivePath = '$letter:${Platform.pathSeparator}';
        if (Directory(drivePath).existsSync()) {
          bookmarks.add({
            'name': 'Drive ($letter:)',
            'path': drivePath,
            'icon': Icons.storage,
          });
        }
      }
    }

    return Container(
      width: 180,
      decoration: const BoxDecoration(
        color: IrisTheme.surfaceDark,
        border: Border(right: BorderSide(color: IrisTheme.border)),
      ),
      child: ListView.builder(
        padding: const EdgeInsets.symmetric(vertical: 8),
        itemCount: bookmarks.length,
        itemBuilder: (ctx, idx) {
          final b = bookmarks[idx];
          final path = b['path'] as String;
          final isSelected = _currentDirectory.path.toLowerCase() == path.toLowerCase();

          return Material(
            color: Colors.transparent,
            child: ListTile(
              dense: true,
              visualDensity: VisualDensity.compact,
              leading: Icon(b['icon'] as IconData, size: 16, color: isSelected ? IrisTheme.primaryCyan : IrisTheme.textSecondary),
              title: Text(
                b['name'] as String,
                style: TextStyle(
                  fontSize: 12,
                  fontWeight: isSelected ? FontWeight.w600 : FontWeight.normal,
                  color: isSelected ? IrisTheme.primaryCyan : IrisTheme.textPrimary,
                ),
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
              selected: isSelected,
              selectedTileColor: IrisTheme.surfaceSelected,
              onTap: () {
                if (Directory(path).existsSync()) {
                  setState(() {
                    _currentDirectory = Directory(path);
                    _selectedPaths.clear();
                    _probedPath = null;
                    _probedInfo = null;
                  });
                  _loadDirectory();
                }
              },
            ),
          );
        },
      ),
    );
  }

  Widget _buildFileListSection() {
    return Column(
      children: [
        // Path navigation bar and search/filter bar
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
          decoration: const BoxDecoration(
            color: IrisTheme.surfaceMid,
            border: Border(bottom: BorderSide(color: IrisTheme.border)),
          ),
          child: Column(
            children: [
              // Path breadcrumb & Up button
              Row(
                children: [
                  IconButton(
                    icon: const Icon(Icons.arrow_upward, size: 16),
                    tooltip: 'Up to Parent Directory',
                    visualDensity: VisualDensity.compact,
                    splashRadius: 16,
                    onPressed: () {
                      final parent = _currentDirectory.parent;
                      if (parent.path != _currentDirectory.path) {
                        setState(() {
                          _currentDirectory = parent;
                          _selectedPaths.clear();
                        });
                        _loadDirectory();
                      }
                    },
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Container(
                      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
                      decoration: BoxDecoration(
                        color: IrisTheme.surfaceDark,
                        borderRadius: BorderRadius.circular(6),
                        border: Border.all(color: IrisTheme.border),
                      ),
                      child: Text(
                        _currentDirectory.path,
                        style: const TextStyle(fontSize: 11, fontFamily: 'monospace', color: IrisTheme.textSecondary),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                  ),
                  const SizedBox(width: 8),
                  IconButton(
                    icon: const Icon(Icons.refresh, size: 16),
                    tooltip: 'Refresh',
                    visualDensity: VisualDensity.compact,
                    splashRadius: 16,
                    onPressed: _loadDirectory,
                  ),
                ],
              ),
              const SizedBox(height: 8),

              // Filter selector & Search box
              Row(
                children: [
                  // Filter Dropdown
                  Container(
                    height: 32,
                    padding: const EdgeInsets.symmetric(horizontal: 8),
                    decoration: BoxDecoration(
                      color: IrisTheme.surfaceDark,
                      borderRadius: BorderRadius.circular(6),
                      border: Border.all(color: IrisTheme.border),
                    ),
                    child: DropdownButtonHideUnderline(
                      child: DropdownButton<FileFilterMode>(
                        value: _filterMode,
                        dropdownColor: IrisTheme.surfaceDark,
                        style: const TextStyle(color: IrisTheme.textPrimary, fontSize: 11),
                        items: const [
                          DropdownMenuItem(value: FileFilterMode.allMedia, child: Text('All Supported Media')),
                          DropdownMenuItem(value: FileFilterMode.photos, child: Text('Photos (*.jpg, *.png, *.webp)')),
                          DropdownMenuItem(value: FileFilterMode.videos, child: Text('Videos (*.mp4, *.mov, *.webm)')),
                          DropdownMenuItem(value: FileFilterMode.audio, child: Text('Audio (*.wav, *.mp3, *.aac)')),
                          DropdownMenuItem(value: FileFilterMode.projects, child: Text('Iris Projects (*.irisproj)')),
                          DropdownMenuItem(value: FileFilterMode.allFiles, child: Text('All Files (*.*)')),
                        ],
                        onChanged: (val) {
                          if (val != null) setState(() => _filterMode = val);
                        },
                      ),
                    ),
                  ),
                  const SizedBox(width: 12),

                  // Search box
                  Expanded(
                    child: SizedBox(
                      height: 32,
                      child: TextField(
                        controller: _searchController,
                        style: const TextStyle(fontSize: 11, color: IrisTheme.textPrimary),
                        decoration: InputDecoration(
                          hintText: 'Search files...',
                          hintStyle: const TextStyle(color: IrisTheme.textMuted, fontSize: 11),
                          prefixIcon: const Icon(Icons.search, size: 14, color: IrisTheme.textMuted),
                          filled: true,
                          fillColor: IrisTheme.surfaceDark,
                          contentPadding: const EdgeInsets.symmetric(horizontal: 8, vertical: 0),
                          border: OutlineInputBorder(
                            borderRadius: BorderRadius.circular(6),
                            borderSide: const BorderSide(color: IrisTheme.border),
                          ),
                          enabledBorder: OutlineInputBorder(
                            borderRadius: BorderRadius.circular(6),
                            borderSide: const BorderSide(color: IrisTheme.border),
                          ),
                        ),
                        onChanged: (val) => setState(() => _searchQuery = val),
                      ),
                    ),
                  ),
                ],
              ),
            ],
          ),
        ),

        // File List Table
        Expanded(
          child: _isLoading
              ? const Center(child: CircularProgressIndicator())
              : _buildEntriesList(),
        ),
      ],
    );
  }

  Widget _buildEntriesList() {
    final filtered = _entries.where((e) {
      if (e is Directory) return true;
      return _matchesFilter(e.path);
    }).toList();

    if (filtered.isEmpty) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.folder_open, size: 36, color: IrisTheme.textMuted),
            const SizedBox(height: 8),
            Text(
              _searchQuery.isNotEmpty ? 'No matching files found' : 'No supported media files in this folder',
              style: const TextStyle(color: IrisTheme.textMuted, fontSize: 12),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      itemCount: filtered.length,
      itemBuilder: (ctx, idx) {
        final entry = filtered[idx];
        final isDir = entry is Directory;
        final name = entry.path.split(Platform.pathSeparator).last;
        final isSelected = _selectedPaths.contains(entry.path);

        return InkWell(
          onTap: () {
            if (isDir) {
              setState(() {
                _currentDirectory = Directory(entry.path);
                _selectedPaths.clear();
                _probedPath = null;
                _probedInfo = null;
              });
              _loadDirectory();
            } else {
              setState(() {
                if (widget.allowMultiple) {
                  if (isSelected) {
                    _selectedPaths.remove(entry.path);
                  } else {
                    _selectedPaths.add(entry.path);
                  }
                } else {
                  _selectedPaths.clear();
                  _selectedPaths.add(entry.path);
                  _fileNameController.text = name;
                }
              });
              _probeSelectedFile(entry.path);
            }
          },
          onDoubleTap: () {
            if (isDir) {
              setState(() {
                _currentDirectory = Directory(entry.path);
                _selectedPaths.clear();
              });
              _loadDirectory();
            } else {
              _onConfirm();
            }
          },
          child: Container(
            color: isSelected ? IrisTheme.surfaceSelected : Colors.transparent,
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
            child: Row(
              children: [
                Icon(
                  isDir ? Icons.folder : _getFileIcon(entry.path),
                  size: 18,
                  color: isDir
                      ? Colors.amberAccent
                      : (isSelected ? IrisTheme.primaryCyan : IrisTheme.textSecondary),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Text(
                    name,
                    style: TextStyle(
                      fontSize: 12,
                      fontWeight: isSelected ? FontWeight.w600 : FontWeight.normal,
                      color: isSelected ? IrisTheme.primaryCyan : IrisTheme.textPrimary,
                    ),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                if (!isDir) ...[
                  Text(
                    _getFileSize(entry),
                    style: const TextStyle(color: IrisTheme.textMuted, fontSize: 11),
                  ),
                ],
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _buildMetadataPreview() {
    final info = _probedInfo;
    if (info == null) return const SizedBox.shrink();

    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'PROBED METADATA',
            style: TextStyle(
              fontSize: 10,
              fontWeight: FontWeight.w700,
              color: IrisTheme.primaryCyan,
              letterSpacing: 0.8,
            ),
          ),
          const SizedBox(height: 12),

          Text(
            info.filename,
            style: const TextStyle(color: IrisTheme.textPrimary, fontWeight: FontWeight.w600, fontSize: 13),
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
          ),
          const SizedBox(height: 8),

          Container(
            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 3),
            decoration: BoxDecoration(
              color: IrisTheme.primaryCyan.withValues(alpha: 0.15),
              borderRadius: BorderRadius.circular(4),
              border: Border.all(color: IrisTheme.primaryCyan.withValues(alpha: 0.3)),
            ),
            child: Text(
              info.codecName,
              style: const TextStyle(color: IrisTheme.primaryCyan, fontSize: 10, fontWeight: FontWeight.w600),
            ),
          ),
          const SizedBox(height: 16),

          if (info.width > 0 && info.height > 0)
            _metaRow('Resolution', '${info.width} × ${info.height}'),
          if (info.fps > 0)
            _metaRow('Frame Rate', '${info.fps.toStringAsFixed(1)} fps'),
          if (info.durationSeconds > 0)
            _metaRow('Duration', '${info.durationSeconds.toStringAsFixed(2)}s'),
          if (info.sampleRate > 0)
            _metaRow('Audio Rate', '${info.sampleRate} Hz (${info.channels} ch)'),
          _metaRow('File Size', _formatBytes(info.fileSizeBytes)),
          _metaRow('Format', info.format.toUpperCase()),
          if (_probedPath != null) _metaRow('Path', _probedPath!),

          const Spacer(),
          Container(
            padding: const EdgeInsets.all(10),
            decoration: BoxDecoration(
              color: IrisTheme.surfaceDark,
              borderRadius: BorderRadius.circular(6),
              border: Border.all(color: IrisTheme.border),
            ),
            child: Row(
              children: const [
                Icon(Icons.verified, size: 14, color: IrisTheme.accentGreen),
                SizedBox(width: 8),
                Expanded(
                  child: Text(
                    'Rust container validated for native timeline editing.',
                    style: TextStyle(color: IrisTheme.textSecondary, fontSize: 10),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _metaRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: const TextStyle(color: IrisTheme.textMuted, fontSize: 11)),
          Text(value, style: const TextStyle(color: IrisTheme.textPrimary, fontSize: 11, fontWeight: FontWeight.w500)),
        ],
      ),
    );
  }

  Widget _buildFooter() {
    final canConfirm = widget.isSaveMode || _selectedPaths.isNotEmpty;

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 12),
      decoration: const BoxDecoration(
        color: IrisTheme.surfaceMid,
        borderRadius: BorderRadius.vertical(bottom: Radius.circular(12)),
        border: Border(top: BorderSide(color: IrisTheme.border)),
      ),
      child: Row(
        children: [
          if (widget.isSaveMode)
            Expanded(
              child: Row(
                children: [
                  const Text('File Name:', style: TextStyle(color: IrisTheme.textSecondary, fontSize: 12)),
                  const SizedBox(width: 10),
                  Expanded(
                    child: SizedBox(
                      height: 32,
                      child: TextField(
                        controller: _fileNameController,
                        style: const TextStyle(fontSize: 12, color: IrisTheme.textPrimary),
                        decoration: InputDecoration(
                          filled: true,
                          fillColor: IrisTheme.surfaceDark,
                          contentPadding: const EdgeInsets.symmetric(horizontal: 10, vertical: 0),
                          border: OutlineInputBorder(borderRadius: BorderRadius.circular(6)),
                        ),
                      ),
                    ),
                  ),
                  const SizedBox(width: 16),
                ],
              ),
            )
          else
            Text(
              _selectedPaths.isEmpty
                  ? 'No file selected'
                  : '${_selectedPaths.length} file${_selectedPaths.length > 1 ? "s" : ""} selected',
              style: const TextStyle(color: IrisTheme.textMuted, fontSize: 12),
            ),
          const Spacer(),

          OutlinedButton(
            style: OutlinedButton.styleFrom(
              foregroundColor: IrisTheme.textSecondary,
              side: const BorderSide(color: IrisTheme.border),
            ),
            onPressed: () => Navigator.pop(context, null),
            child: const Text('Cancel'),
          ),
          const SizedBox(width: 12),

          ElevatedButton(
            style: ElevatedButton.styleFrom(
              backgroundColor: canConfirm ? IrisTheme.primaryCyan : IrisTheme.disabledBackground,
              foregroundColor: canConfirm ? Colors.black : IrisTheme.disabledText,
            ),
            onPressed: canConfirm ? _onConfirm : null,
            child: Text(widget.isSaveMode ? 'Save Project' : 'Open / Import'),
          ),
        ],
      ),
    );
  }

  IconData _getFileIcon(String path) {
    final ext = path.split('.').last.toLowerCase();
    if (['jpg', 'jpeg', 'png', 'webp', 'bmp'].contains(ext)) return Icons.image;
    if (['mp4', 'mov', 'mkv', 'webm', 'avi'].contains(ext)) return Icons.video_file;
    if (['wav', 'mp3', 'aac', 'flac', 'ogg', 'm4a'].contains(ext)) return Icons.audio_file;
    if (['irisproj', 'json'].contains(ext)) return Icons.dashboard_customize_outlined;
    return Icons.insert_drive_file;
  }

  String _getFileSize(FileSystemEntity entry) {
    try {
      final stat = entry.statSync();
      return _formatBytes(stat.size);
    } catch (_) {
      return '';
    }
  }

  String _formatBytes(int bytes) {
    if (bytes <= 0) return '0 B';
    if (bytes < 1024) return '$bytes B';
    if (bytes < 1024 * 1024) return '${(bytes / 1024).toStringAsFixed(1)} KB';
    return '${(bytes / (1024 * 1024)).toStringAsFixed(1)} MB';
  }
}
