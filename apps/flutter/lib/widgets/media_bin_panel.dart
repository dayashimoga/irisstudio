import 'dart:io';
import 'package:flutter/material.dart';
import '../bridge/iris_engine_controller.dart';
import '../models/media_item.dart';
import '../theme/iris_theme.dart';
import 'media_picker_dialog.dart';

class MediaBinPanel extends StatelessWidget {
  final IrisEngineController controller;

  const MediaBinPanel({super.key, required this.controller});

  Future<void> _importMedia(BuildContext context) async {
    final paths = await MediaPickerDialog.show(
      context,
      title: 'Import Media into Project',
      initialFilter: FileFilterMode.allMedia,
      allowMultiple: true,
    );

    if (paths != null && paths.isNotEmpty) {
      for (final p in paths) {
        await controller.importMediaFile(p);
      }
    }
  }

  Future<void> _relinkMissingMedia(BuildContext context, MediaItem item) async {
    final paths = await MediaPickerDialog.show(
      context,
      title: 'Relink Missing Media: ${item.fileName}',
      initialFilter: item.mediaType == MediaType.photo
          ? FileFilterMode.photos
          : (item.mediaType == MediaType.audio ? FileFilterMode.audio : FileFilterMode.videos),
      allowMultiple: false,
    );

    if (paths != null && paths.isNotEmpty) {
      controller.relinkMediaItem(item.id, paths.first);
    }
  }

  @override
  Widget build(BuildContext context) {
    final items = controller.mediaBin;

    return Container(
      decoration: const BoxDecoration(
        color: IrisTheme.surfaceDark,
        border: Border(right: BorderSide(color: IrisTheme.border)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Header Bar
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
            decoration: const BoxDecoration(
              color: IrisTheme.surfaceMid,
              border: Border(bottom: BorderSide(color: IrisTheme.border)),
            ),
            child: Row(
              children: [
                const Icon(Icons.video_library_outlined, size: 16, color: IrisTheme.primaryCyan),
                const SizedBox(width: 6),
                const Expanded(
                  child: Text(
                    'MEDIA POOL',
                    style: TextStyle(
                      fontSize: 11,
                      fontWeight: FontWeight.w700,
                      color: IrisTheme.textPrimary,
                      letterSpacing: 0.8,
                    ),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                Text(
                  '${items.length}',
                  style: const TextStyle(fontSize: 11, color: IrisTheme.textMuted),
                ),
                const SizedBox(width: 6),
                IconButton(
                  icon: const Icon(Icons.add, size: 16, color: IrisTheme.primaryCyan),
                  tooltip: 'Import Media',
                  visualDensity: VisualDensity.compact,
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints(),
                  onPressed: () => _importMedia(context),
                ),
              ],
            ),
          ),

          // Items List / Grid
          Expanded(
            child: items.isEmpty
                ? _buildEmptyState(context)
                : ListView.builder(
                    padding: const EdgeInsets.all(8),
                    itemCount: items.length,
                    itemBuilder: (ctx, idx) => _buildMediaTile(context, items[idx]),
                  ),
          ),
        ],
      ),
    );
  }

  Widget _buildEmptyState(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: IrisTheme.surfaceMid,
                shape: BoxShape.circle,
                border: Border.all(color: IrisTheme.border),
              ),
              child: const Icon(Icons.perm_media_outlined, size: 32, color: IrisTheme.textMuted),
            ),
            const SizedBox(height: 14),
            const Text(
              'No Media Imported',
              style: TextStyle(
                color: IrisTheme.textPrimary,
                fontSize: 13,
                fontWeight: FontWeight.w600,
              ),
            ),
            const SizedBox(height: 6),
            const Text(
              'Import real video, photo, or audio files from your filesystem to begin editing.',
              style: TextStyle(color: IrisTheme.textMuted, fontSize: 11),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 16),
            OutlinedButton.icon(
              icon: const Icon(Icons.file_open_outlined, size: 14),
              label: const Text('Import Media Files'),
              style: OutlinedButton.styleFrom(
                foregroundColor: IrisTheme.primaryCyan,
                side: const BorderSide(color: IrisTheme.primaryCyan),
              ),
              onPressed: () => _importMedia(context),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildMediaTile(BuildContext context, MediaItem item) {
    final isSelected = controller.selectedMediaItem?.id == item.id;
    final isMissing = item.isMissing || !File(item.filePath).existsSync();

    return InkWell(
      onTap: () => controller.selectMediaItem(item),
      onDoubleTap: () => controller.addMediaItemToTimeline(item),
      child: Container(
        margin: const EdgeInsets.only(bottom: 6),
        padding: const EdgeInsets.all(8),
        decoration: BoxDecoration(
          color: isSelected ? IrisTheme.surfaceSelected : IrisTheme.surfaceMid,
          borderRadius: BorderRadius.circular(6),
          border: Border.all(
            color: isMissing
                ? IrisTheme.accentAmber
                : (isSelected ? IrisTheme.primaryCyan : IrisTheme.border),
            width: isSelected ? 1.5 : 1.0,
          ),
        ),
        child: Row(
          children: [
            // Thumbnail or Media Icon
            Container(
              width: 48,
              height: 48,
              decoration: BoxDecoration(
                color: IrisTheme.surfaceDark,
                borderRadius: BorderRadius.circular(4),
              ),
              child: item.thumbnailBytes != null
                  ? ClipRRect(
                      borderRadius: BorderRadius.circular(4),
                      child: Image.memory(
                        item.thumbnailBytes!,
                        fit: BoxFit.cover,
                      ),
                    )
                  : Center(
                      child: Icon(
                        _getTypeIcon(item.mediaType),
                        color: _getTypeColor(item.mediaType),
                        size: 24,
                      ),
                    ),
            ),
            const SizedBox(width: 10),

            // Metadata column
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    item.fileName,
                    style: TextStyle(
                      color: isMissing ? IrisTheme.accentAmber : IrisTheme.textPrimary,
                      fontSize: 12,
                      fontWeight: FontWeight.w600,
                    ),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                  const SizedBox(height: 2),

                  Wrap(
                    spacing: 4,
                    runSpacing: 2,
                    crossAxisAlignment: WrapCrossAlignment.center,
                    children: [
                      Container(
                        padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
                        decoration: BoxDecoration(
                          color: IrisTheme.surfaceLight,
                          borderRadius: BorderRadius.circular(2),
                        ),
                        child: Text(
                          item.format.toUpperCase(),
                          style: const TextStyle(
                            color: IrisTheme.primaryCyan,
                            fontSize: 9,
                            fontWeight: FontWeight.bold,
                          ),
                        ),
                      ),
                      Text(
                        item.humanDuration,
                        style: const TextStyle(color: IrisTheme.textMuted, fontSize: 10),
                      ),
                      Text(
                        item.humanFileSize,
                        style: const TextStyle(color: IrisTheme.textMuted, fontSize: 10),
                      ),
                    ],
                  ),

                  if (isMissing) ...[
                    const SizedBox(height: 4),
                    Wrap(
                      spacing: 4,
                      runSpacing: 2,
                      crossAxisAlignment: WrapCrossAlignment.center,
                      children: [
                        const Icon(Icons.warning_amber_rounded, size: 12, color: IrisTheme.accentAmber),
                        const Text(
                          'File Missing',
                          style: TextStyle(color: IrisTheme.accentAmber, fontSize: 10, fontWeight: FontWeight.w600),
                        ),
                        InkWell(
                          onTap: () => _relinkMissingMedia(context, item),
                          child: const Text(
                            'Relink...',
                            style: TextStyle(color: IrisTheme.primaryCyan, fontSize: 10, decoration: TextDecoration.underline),
                          ),
                        ),
                      ],
                    ),
                  ],
                ],
              ),
            ),

            // Add to timeline button
            IconButton(
              icon: const Icon(Icons.playlist_add, size: 18),
              tooltip: 'Add to Timeline',
              splashRadius: 16,
              onPressed: () => controller.addMediaItemToTimeline(item),
            ),
          ],
        ),
      ),
    );
  }

  IconData _getTypeIcon(MediaType type) {
    switch (type) {
      case MediaType.photo:
        return Icons.image;
      case MediaType.video:
        return Icons.movie_outlined;
      case MediaType.audio:
        return Icons.audiotrack;
      case MediaType.unknown:
        return Icons.insert_drive_file;
    }
  }

  Color _getTypeColor(MediaType type) {
    switch (type) {
      case MediaType.photo:
        return IrisTheme.accentPurple;
      case MediaType.video:
        return IrisTheme.primaryCyan;
      case MediaType.audio:
        return IrisTheme.accentGreen;
      case MediaType.unknown:
        return IrisTheme.textMuted;
    }
  }
}
