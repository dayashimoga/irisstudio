enum ExportStatus { queued, processing, completed, failed, cancelled }

class ExportJob {
  final String id;
  final String title;
  final String outputPath;
  final String format;
  final String resolution;
  final int fps;
  final int bitrateKbps;
  ExportStatus status;
  double progressPercent;
  double currentFps;
  int fileSizeBytes;
  String? errorMessage;

  ExportJob({
    required this.id,
    required this.title,
    required this.outputPath,
    required this.format,
    required this.resolution,
    required this.fps,
    required this.bitrateKbps,
    this.status = ExportStatus.queued,
    this.progressPercent = 0.0,
    this.currentFps = 0.0,
    this.fileSizeBytes = 0,
    this.errorMessage,
  });
}
