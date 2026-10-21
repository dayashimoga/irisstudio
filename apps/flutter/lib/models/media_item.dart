import 'dart:io';
import 'package:flutter/foundation.dart';
import '../bridge/iris_ffi_bindings.dart';

enum MediaType { photo, video, audio, unknown }

class MediaItem {
  final String id;
  String filePath;
  final String fileName;
  final MediaType mediaType;
  final String format;
  final int width;
  final int height;
  final double fps;
  final double durationSeconds;
  final int channels;
  final int sampleRate;
  final String codecName;
  final int fileSizeBytes;
  bool isMissing;
  Uint8List? thumbnailBytes;
  List<double>? waveformPeaks;

  MediaItem({
    required this.id,
    required this.filePath,
    required this.fileName,
    required this.mediaType,
    required this.format,
    required this.width,
    required this.height,
    required this.fps,
    required this.durationSeconds,
    required this.channels,
    required this.sampleRate,
    required this.codecName,
    required this.fileSizeBytes,
    this.isMissing = false,
    this.thumbnailBytes,
    this.waveformPeaks,
  });

  static MediaItem empty() => MediaItem(
        id: '',
        filePath: '',
        fileName: '',
        mediaType: MediaType.unknown,
        format: '',
        width: 0,
        height: 0,
        fps: 0,
        durationSeconds: 0,
        channels: 0,
        sampleRate: 0,
        codecName: '',
        fileSizeBytes: 0,
      );

  static MediaItem fromProbe({
    required String id,
    required String filePath,
    required IrisMediaInfo probe,
    Uint8List? thumbnailBytes,
  }) {
    MediaType type = MediaType.unknown;
    if (probe.mediaType == 'photo') {
      type = MediaType.photo;
    } else if (probe.mediaType == 'video') {
      type = MediaType.video;
    } else if (probe.mediaType == 'audio') {
      type = MediaType.audio;
    }

    return MediaItem(
      id: id,
      filePath: filePath,
      fileName: probe.filename.isNotEmpty
          ? probe.filename
          : filePath.split(Platform.pathSeparator).last,
      mediaType: type,
      format: probe.format,
      width: probe.width,
      height: probe.height,
      fps: probe.fps,
      durationSeconds: probe.durationSeconds,
      channels: probe.channels,
      sampleRate: probe.sampleRate,
      codecName: probe.codecName,
      fileSizeBytes: probe.fileSizeBytes,
      isMissing: !File(filePath).existsSync(),
      thumbnailBytes: thumbnailBytes,
    );
  }

  Map<String, dynamic> toJson() => {
        'id': id,
        'filePath': filePath,
        'fileName': fileName,
        'mediaType': mediaType.name,
        'format': format,
        'width': width,
        'height': height,
        'fps': fps,
        'durationSeconds': durationSeconds,
        'channels': channels,
        'sampleRate': sampleRate,
        'codecName': codecName,
        'fileSizeBytes': fileSizeBytes,
        'isMissing': isMissing,
      };

  factory MediaItem.fromJson(Map<String, dynamic> json) {
    final typeName = json['mediaType'] as String? ?? 'unknown';
    final mediaType = MediaType.values.firstWhere(
      (m) => m.name == typeName,
      orElse: () => MediaType.unknown,
    );

    final path = json['filePath'] as String? ?? '';

    return MediaItem(
      id: json['id'] as String? ?? 'item_${DateTime.now().millisecondsSinceEpoch}',
      filePath: path,
      fileName: json['fileName'] as String? ?? '',
      mediaType: mediaType,
      format: json['format'] as String? ?? '',
      width: json['width'] as int? ?? 0,
      height: json['height'] as int? ?? 0,
      fps: (json['fps'] as num?)?.toDouble() ?? 30.0,
      durationSeconds: (json['durationSeconds'] as num?)?.toDouble() ?? 0.0,
      channels: json['channels'] as int? ?? 2,
      sampleRate: json['sampleRate'] as int? ?? 48000,
      codecName: json['codecName'] as String? ?? '',
      fileSizeBytes: json['fileSizeBytes'] as int? ?? 0,
      isMissing: path.isNotEmpty ? !File(path).existsSync() : false,
    );
  }

  String get humanDuration {
    if (mediaType == MediaType.photo) return 'Photo Asset';
    final totalSec = durationSeconds.round();
    final mins = (totalSec ~/ 60).toString().padLeft(2, '0');
    final secs = (totalSec % 60).toString().padLeft(2, '0');
    return '$mins:$secs';
  }

  String get humanFileSize {
    if (fileSizeBytes <= 0) return '0 B';
    if (fileSizeBytes < 1024) return '$fileSizeBytes B';
    if (fileSizeBytes < 1024 * 1024) {
      return '${(fileSizeBytes / 1024).toStringAsFixed(1)} KB';
    }
    return '${(fileSizeBytes / (1024 * 1024)).toStringAsFixed(1)} MB';
  }
}
