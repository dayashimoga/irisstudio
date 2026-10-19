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

class ChromaKeyModel {
  bool isEnabled;
  double keyColorR;
  double keyColorG;
  double keyColorB;
  double similarity;
  double smoothness;
  double spillSuppression;

  ChromaKeyModel({
    this.isEnabled = false,
    this.keyColorR = 0.0,
    this.keyColorG = 1.0,
    this.keyColorB = 0.0,
    this.similarity = 0.35,
    this.smoothness = 0.15,
    this.spillSuppression = 0.8,
  });

  ChromaKeyModel copyWith({
    bool? isEnabled,
    double? keyColorR,
    double? keyColorG,
    double? keyColorB,
    double? similarity,
    double? smoothness,
    double? spillSuppression,
  }) {
    return ChromaKeyModel(
      isEnabled: isEnabled ?? this.isEnabled,
      keyColorR: keyColorR ?? this.keyColorR,
      keyColorG: keyColorG ?? this.keyColorG,
      keyColorB: keyColorB ?? this.keyColorB,
      similarity: similarity ?? this.similarity,
      smoothness: smoothness ?? this.smoothness,
      spillSuppression: spillSuppression ?? this.spillSuppression,
    );
  }
}

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
  bool preservePitch;
  ChromaKeyModel chromaKey;

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
    this.preservePitch = true,
    ChromaKeyModel? chromaKey,
  }) : chromaKey = chromaKey ?? ChromaKeyModel();

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
    bool? preservePitch,
    ChromaKeyModel? chromaKey,
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
      preservePitch: preservePitch ?? this.preservePitch,
      chromaKey: chromaKey ?? this.chromaKey.copyWith(),
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

class TransitionModel {
  final String id;
  final String kindName;
  final double startTime;
  final double duration;
  final String clipAId;
  final String clipBId;

  const TransitionModel({
    required this.id,
    required this.kindName,
    required this.startTime,
    required this.duration,
    required this.clipAId,
    required this.clipBId,
  });
}

class SubtitleItemModel {
  final String id;
  final double startTime;
  final double duration;
  final String text;

  const SubtitleItemModel({
    required this.id,
    required this.startTime,
    required this.duration,
    required this.text,
  });

  double get endTime => startTime + duration;
}

class AudioEqModel {
  double lowGainDb;
  double midGainDb;
  double highGainDb;

  AudioEqModel({
    this.lowGainDb = 0.0,
    this.midGainDb = 0.0,
    this.highGainDb = 0.0,
  });
}

class AudioCompressorModel {
  double thresholdDbfs;
  double ratio;
  double makeupGainDb;

  AudioCompressorModel({
    this.thresholdDbfs = -18.0,
    this.ratio = 4.0,
    this.makeupGainDb = 3.0,
  });
}

enum LutPresetModel {
  none,
  tealAndOrange,
  warmVintage,
  bleachBypass,
  fadedFilm,
  monochromeContrast,
  customCube,
}

class RgbCurvePointModel {
  final double x;
  final double y;
  const RgbCurvePointModel(this.x, this.y);
}

class RgbCurvesModel {
  List<RgbCurvePointModel> master;
  List<RgbCurvePointModel> red;
  List<RgbCurvePointModel> green;
  List<RgbCurvePointModel> blue;

  RgbCurvesModel({
    List<RgbCurvePointModel>? master,
    List<RgbCurvePointModel>? red,
    List<RgbCurvePointModel>? green,
    List<RgbCurvePointModel>? blue,
  })  : master = master ?? [const RgbCurvePointModel(0.0, 0.0), const RgbCurvePointModel(1.0, 1.0)],
        red = red ?? [const RgbCurvePointModel(0.0, 0.0), const RgbCurvePointModel(1.0, 1.0)],
        green = green ?? [const RgbCurvePointModel(0.0, 0.0), const RgbCurvePointModel(1.0, 1.0)],
        blue = blue ?? [const RgbCurvePointModel(0.0, 0.0), const RgbCurvePointModel(1.0, 1.0)];
}

class AudioDuckerModel {
  bool isEnabled;
  double thresholdDb;
  double duckAttenuationDb;
  double attackMs;
  double holdMs;
  double releaseMs;
  String? dialogueTrackId;

  AudioDuckerModel({
    this.isEnabled = false,
    this.thresholdDb = -24.0,
    this.duckAttenuationDb = -12.0,
    this.attackMs = 20.0,
    this.holdMs = 300.0,
    this.releaseMs = 450.0,
    this.dialogueTrackId,
  });
}

enum ColorScopeMode {
  none,
  waveform,
  vectorscope,
  rgbParade,
}

enum MotionTitleTemplate {
  minimalClean,
  lowerThirdNews,
  cinematicCenter,
  calloutBox,
}

class MotionTitleModel {
  final String id;
  String title;
  String subtitle;
  MotionTitleTemplate template;
  double startTime;
  double duration;

  MotionTitleModel({
    required this.id,
    required this.title,
    required this.subtitle,
    this.template = MotionTitleTemplate.minimalClean,
    required this.startTime,
    this.duration = 4.0,
  });

  double get endTime => startTime + duration;
}

class HardwareAccelerationInfo {
  final String backendName;
  final bool isAvailable;
  final bool supportsZeroCopy;
  final int maxWidth;
  final int maxHeight;
  final int maxStreams;

  const HardwareAccelerationInfo({
    required this.backendName,
    this.isAvailable = true,
    this.supportsZeroCopy = true,
    this.maxWidth = 8192,
    this.maxHeight = 4320,
    this.maxStreams = 8,
  });

  static const HardwareAccelerationInfo defaultDetected = HardwareAccelerationInfo(
    backendName: 'DirectX 11 Video Acceleration (D3D11VA)',
    isAvailable: true,
    supportsZeroCopy: true,
    maxWidth: 8192,
    maxHeight: 4320,
    maxStreams: 8,
  );
}

class ProjectModel {
  final String id;
  String name;
  final int fps;
  final int width;
  final int height;
  final List<TrackModel> tracks;
  final List<TransitionModel> transitions;
  final List<SubtitleItemModel> subtitles;
  final List<MotionTitleModel> motionTitles;
  HardwareAccelerationInfo hardwareInfo;
  AudioEqModel eq;
  AudioCompressorModel compressor;
  LutPresetModel lutPreset;
  double lutIntensity;
  String? customCubeLutPath;
  RgbCurvesModel curves;
  AudioDuckerModel ducker;
  ColorScopeMode scopeMode;

  ProjectModel({
    required this.id,
    required this.name,
    this.fps = 30,
    this.width = 1920,
    this.height = 1080,
    required this.tracks,
    List<TransitionModel>? transitions,
    List<SubtitleItemModel>? subtitles,
    List<MotionTitleModel>? motionTitles,
    HardwareAccelerationInfo? hardwareInfo,
    AudioEqModel? eq,
    AudioCompressorModel? compressor,
    this.lutPreset = LutPresetModel.none,
    this.lutIntensity = 1.0,
    this.customCubeLutPath,
    RgbCurvesModel? curves,
    AudioDuckerModel? ducker,
    this.scopeMode = ColorScopeMode.none,
  })  : transitions = transitions ?? [],
        subtitles = subtitles ?? [],
        motionTitles = motionTitles ?? [],
        hardwareInfo = hardwareInfo ?? HardwareAccelerationInfo.defaultDetected,
        eq = eq ?? AudioEqModel(),
        compressor = compressor ?? AudioCompressorModel(),
        curves = curves ?? RgbCurvesModel(),
        ducker = ducker ?? AudioDuckerModel();

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
