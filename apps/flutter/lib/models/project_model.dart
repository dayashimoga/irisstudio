class RationalTimeModel {
  final int numerator;
  final int denominator;

  const RationalTimeModel(this.numerator, this.denominator);

  double get toSeconds => numerator / (denominator > 0 ? denominator : 1);

  String toTimecode() {
    final totalSecs = toSeconds.clamp(0.0, 999999.0);
    final hours = (totalSecs / 3600).floor();
    final rem1 = totalSecs - (hours * 3600);
    final minutes = (rem1 / 60).floor();
    final rem2 = rem1 - (minutes * 60);
    final seconds = rem2.floor();
    final millis = ((rem2 - seconds) * 1000).round();

    return '${hours.toString().padLeft(2, '0')}:'
        '${minutes.toString().padLeft(2, '0')}:'
        '${seconds.toString().padLeft(2, '0')}.'
        '${millis.toString().padLeft(3, '0')}';
  }
}

enum TrackKindModel { video, audio, overlay }

class ClipModel {
  final String id;
  final String assetId;
  final String name;
  double timelineStart; // in seconds
  double duration; // in seconds
  double speed;
  double opacity;
  double scale;
  double rotation;
  double volume;

  ClipModel({
    required this.id,
    required this.assetId,
    required this.name,
    required this.timelineStart,
    required this.duration,
    this.speed = 1.0,
    this.opacity = 1.0,
    this.scale = 1.0,
    this.rotation = 0.0,
    this.volume = 1.0,
  });

  ClipModel copyWith({
    String? id,
    String? assetId,
    String? name,
    double? timelineStart,
    double? duration,
    double? speed,
    double? opacity,
    double? scale,
    double? rotation,
    double? volume,
  }) {
    return ClipModel(
      id: id ?? this.id,
      assetId: assetId ?? this.assetId,
      name: name ?? this.name,
      timelineStart: timelineStart ?? this.timelineStart,
      duration: duration ?? this.duration,
      speed: speed ?? this.speed,
      opacity: opacity ?? this.opacity,
      scale: scale ?? this.scale,
      rotation: rotation ?? this.rotation,
      volume: volume ?? this.volume,
    );
  }
}

class TrackModel {
  final String id;
  final String name;
  final TrackKindModel kind;
  bool isMuted;
  bool isSolo;
  bool isLocked;
  double volume;
  final List<ClipModel> clips;

  TrackModel({
    required this.id,
    required this.name,
    required this.kind,
    this.isMuted = false,
    this.isSolo = false,
    this.isLocked = false,
    this.volume = 1.0,
    required this.clips,
  });
}

class ProjectModel {
  final String id;
  String name;
  final int fps;
  final int width;
  final int height;
  final List<TrackModel> tracks;

  ProjectModel({
    required this.id,
    required this.name,
    this.fps = 30,
    this.width = 1920,
    this.height = 1080,
    required this.tracks,
  });

  double get totalDuration {
    double maxDur = 0.0;
    for (final t in tracks) {
      for (final c in t.clips) {
        final end = c.timelineStart + c.duration;
        if (end > maxDur) {
          maxDur = end;
        }
      }
    }
    return maxDur > 0.0 ? maxDur : 10.0;
  }
}
