import 'dart:async';
import 'dart:math' as math;
import 'package:flutter/foundation.dart';
import '../models/export_job.dart';
import '../models/photo_state.dart';
import '../models/project_model.dart';
import 'iris_ffi_bindings.dart';

enum ActiveWorkspace { home, photo, video, export }

class IrisEngineController extends ChangeNotifier {
  ActiveWorkspace _currentWorkspace = ActiveWorkspace.home;
  ActiveWorkspace get currentWorkspace => _currentWorkspace;

  // Project State
  final ProjectModel _project = _createDefaultProject();
  ProjectModel get project => _project;

  // Timeline & Playback
  double _playheadSeconds = 0.0;
  double get playheadSeconds => _playheadSeconds;

  bool _isPlaying = false;
  bool get isPlaying => _isPlaying;

  bool _isSnappingEnabled = true;
  bool get isSnappingEnabled => _isSnappingEnabled;

  double _timelineZoom = 1.0;
  double get timelineZoom => _timelineZoom;

  ClipModel? _selectedClip;
  ClipModel? get selectedClip => _selectedClip;

  // Audio Meters
  double _leftDbfs = -42.0;
  double _rightDbfs = -40.0;
  double _peakLeft = 0.2;
  double _peakRight = 0.25;

  double get leftDbfs => _leftDbfs;
  double get rightDbfs => _rightDbfs;
  double get peakLeft => _peakLeft;
  double get peakRight => _peakRight;

  // Photo Adjustments State
  PhotoAdjustments _photoAdjustments = PhotoAdjustments();
  PhotoAdjustments get photoAdjustments => _photoAdjustments;

  bool _isBeforeAfterSplit = false;
  bool get isBeforeAfterSplit => _isBeforeAfterSplit;

  double _splitPosition = 0.5; // 0.0 to 1.0
  double get splitPosition => _splitPosition;

  double _photoStraighten = 0.0; // -45.0 to +45.0 degrees
  double get photoStraighten => _photoStraighten;

  int _photoRotation = 0; // 0, 90, 180, 270
  int get photoRotation => _photoRotation;

  bool _photoFlipH = false;
  bool get photoFlipH => _photoFlipH;

  bool _photoFlipV = false;
  bool get photoFlipV => _photoFlipV;

  AspectRatioPreset _aspectRatio = AspectRatioPreset.free;
  AspectRatioPreset get aspectRatio => _aspectRatio;

  // Photo Layers
  final List<PhotoLayerModel> _photoLayers = [
    PhotoLayerModel(id: 'layer_base', name: 'Background (Base)'),
  ];
  List<PhotoLayerModel> get photoLayers => List.unmodifiable(_photoLayers);
  String _selectedLayerId = 'layer_base';
  String get selectedLayerId => _selectedLayerId;

  // Retouch Tools State
  RetouchToolMode _activeRetouchTool = RetouchToolMode.none;
  RetouchToolMode get activeRetouchTool => _activeRetouchTool;
  double _retouchRadius = 25.0;
  double get retouchRadius => _retouchRadius;
  double _retouchHardness = 0.5;
  double get retouchHardness => _retouchHardness;
  double _retouchOpacity = 1.0;
  double get retouchOpacity => _retouchOpacity;

  // Batch Export State
  bool _isBatchProcessing = false;
  bool get isBatchProcessing => _isBatchProcessing;
  double _batchProgress = 0.0;
  double get batchProgress => _batchProgress;
  String _batchStatusMessage = '';
  String get batchStatusMessage => _batchStatusMessage;

  // Undo / Redo Stacks
  final List<String> _undoStack = [];
  final List<String> _redoStack = [];

  bool get canUndo => _undoStack.isNotEmpty;
  bool get canRedo => _redoStack.isNotEmpty;

  // Crash Recovery
  bool _hasUnrecoveredCrash = false;
  bool get hasUnrecoveredCrash => _hasUnrecoveredCrash;

  // Export Jobs Queue
  final List<ExportJob> _exportJobs = [];
  List<ExportJob> get exportJobs => List.unmodifiable(_exportJobs);

  Timer? _playbackTimer;
  Timer? _exportTimer;

  IrisEngineController() {
    IrisFfiBindings.initialize();
    _checkCrashRecovery();
  }

  void _checkCrashRecovery() {
    // In Phase 0, simulate detection of a recovery journal if flag set
    _hasUnrecoveredCrash = true;
  }

  void restoreCrashSession() {
    _hasUnrecoveredCrash = false;
    _project.name = 'Recovered Session (Crash Restored)';
    // Add restored edits
    _photoAdjustments.exposure = 0.85;
    _photoAdjustments.contrast = 15.0;
    _photoAdjustments.saturation = 12.0;
    _recordAction('Crash session restored from WAL journal');
    notifyListeners();
  }

  void dismissCrashRecovery() {
    _hasUnrecoveredCrash = false;
    notifyListeners();
  }

  void switchWorkspace(ActiveWorkspace workspace) {
    if (_currentWorkspace != workspace) {
      if (_isPlaying) {
        pause();
      }
      _currentWorkspace = workspace;
      notifyListeners();
    }
  }

  // --- Playback Controls ---
  void togglePlayPause() {
    if (_isPlaying) {
      pause();
    } else {
      play();
    }
  }

  void play() {
    _isPlaying = true;
    _playbackTimer?.cancel();
    _playbackTimer = Timer.periodic(const Duration(milliseconds: 33), (timer) {
      _playheadSeconds += 0.033;
      if (_playheadSeconds > _project.totalDuration) {
        _playheadSeconds = 0.0; // Loop playback
      }
      _simulateAudioMeters();
      notifyListeners();
    });
    notifyListeners();
  }

  void pause() {
    _isPlaying = false;
    _playbackTimer?.cancel();
    _leftDbfs = -60.0;
    _rightDbfs = -60.0;
    _peakLeft = 0.0;
    _peakRight = 0.0;
    notifyListeners();
  }

  void seekTo(double seconds) {
    _playheadSeconds = seconds.clamp(0.0, _project.totalDuration);
    if (!_isPlaying) {
      _simulateAudioMeters();
    }
    notifyListeners();
  }

  void stepFrame(int direction) {
    final frameDuration = 1.0 / _project.fps;
    seekTo(_playheadSeconds + direction * frameDuration);
  }

  void toggleSnapping() {
    _isSnappingEnabled = !_isSnappingEnabled;
    notifyListeners();
  }

  void setTimelineZoom(double zoom) {
    _timelineZoom = zoom.clamp(0.2, 5.0);
    notifyListeners();
  }

  void selectClip(ClipModel? clip) {
    _selectedClip = clip;
    notifyListeners();
  }

  void updateClip(ClipModel clip) {
    notifyListeners();
  }

  void toggleTrackMute(TrackModel track) {
    track.isMuted = !track.isMuted;
    notifyListeners();
  }

  void splitClipAtPlayhead() {
    if (_project.tracks.isEmpty) return;

    for (final track in _project.tracks) {
      final idx = track.clips.indexWhere((c) =>
          _playheadSeconds > c.timelineStart &&
          _playheadSeconds < c.timelineStart + c.duration);

      if (idx != -1) {
        final orig = track.clips[idx];
        final delta = _playheadSeconds - orig.timelineStart;
        final dur1 = delta;
        final dur2 = orig.duration - delta;

        final clipA = orig.copyWith(duration: dur1);
        final clipB = orig.copyWith(
          id: '${orig.id}_part2',
          name: '${orig.name} (Part 2)',
          timelineStart: _playheadSeconds,
          duration: dur2,
        );

        track.clips[idx] = clipA;
        track.clips.insert(idx + 1, clipB);
        _recordAction('Split clip ${orig.name}');
        notifyListeners();
        return;
      }
    }
  }

  void rippleDeleteSelected() {
    if (_selectedClip == null) return;
    final toDelete = _selectedClip!;

    for (final track in _project.tracks) {
      final idx = track.clips.indexWhere((c) => c.id == toDelete.id);
      if (idx != -1) {
        final removed = track.clips.removeAt(idx);
        final gap = removed.duration;

        // Shift subsequent clips left
        for (int i = idx; i < track.clips.len; i++) {
          track.clips[i].timelineStart =
              (track.clips[i].timelineStart - gap).clamp(0.0, 99999.0);
        }

        _recordAction('Ripple deleted ${removed.name}');
        _selectedClip = null;
        notifyListeners();
        return;
      }
    }
  }

  // --- Photo Adjustments ---
  void updateAdjustments(PhotoAdjustments newAdjustments) {
    _photoAdjustments = newAdjustments;
    notifyListeners();
  }

  void toggleBeforeAfterSplit() {
    _isBeforeAfterSplit = !_isBeforeAfterSplit;
    notifyListeners();
  }

  void setSplitPosition(double pos) {
    _splitPosition = pos.clamp(0.05, 0.95);
    notifyListeners();
  }

  void setPhotoStraighten(double degrees) {
    _photoStraighten = degrees.clamp(-45.0, 45.0);
    notifyListeners();
  }

  void rotatePhoto() {
    _photoRotation = (_photoRotation + 90) % 360;
    _recordAction('Rotated photo to $_photoRotation°');
    notifyListeners();
  }

  void toggleFlipHorizontal() {
    _photoFlipH = !_photoFlipH;
    _recordAction('Flipped horizontal');
    notifyListeners();
  }

  void toggleFlipVertical() {
    _photoFlipV = !_photoFlipV;
    _recordAction('Flipped vertical');
    notifyListeners();
  }

  void setAspectRatio(AspectRatioPreset preset) {
    _aspectRatio = preset;
    notifyListeners();
  }

  void applyPresetFilter(String presetName) {
    _recordAction('Applied preset $presetName');
    switch (presetName) {
      case 'Vivid Landscape':
        _photoAdjustments = _photoAdjustments.copyWith(
          contrast: 18.0,
          vibrance: 28.0,
          saturation: 10.0,
          highlights: -12.0,
          shadows: 15.0,
          sharpening: 35.0,
        );
        break;
      case 'Cinematic Teal/Orange':
        _photoAdjustments = _photoAdjustments.copyWith(
          temperature: 15.0,
          tint: -8.0,
          contrast: 22.0,
          shadows: -10.0,
          vignette: 25.0,
        );
        break;
      case 'B&W Contrast':
        _photoAdjustments = _photoAdjustments.copyWith(
          saturation: -100.0,
          vibrance: -100.0,
          contrast: 35.0,
          exposure: 0.2,
          sharpening: 40.0,
        );
        break;
      case 'Warm Vintage':
        _photoAdjustments = _photoAdjustments.copyWith(
          temperature: 25.0,
          exposure: 0.15,
          contrast: -8.0,
          shadows: 20.0,
          vignette: 30.0,
        );
        break;
      default:
        _photoAdjustments.reset();
    }
    notifyListeners();
  }

  // --- Photo Layers Management ---
  void selectLayer(String id) {
    _selectedLayerId = id;
    notifyListeners();
  }

  void addPhotoLayer(String name) {
    final newId = 'layer_${DateTime.now().millisecondsSinceEpoch}';
    _photoLayers.add(PhotoLayerModel(id: newId, name: name));
    _selectedLayerId = newId;
    _recordAction('Added layer $name');
    notifyListeners();
  }

  void removePhotoLayer(String id) {
    if (_photoLayers.length > 1) {
      _photoLayers.removeWhere((l) => l.id == id);
      if (_selectedLayerId == id) {
        _selectedLayerId = _photoLayers.last.id;
      }
      _recordAction('Removed layer');
      notifyListeners();
    }
  }

  void setLayerOpacity(String id, double opacity) {
    final layer = _photoLayers.firstWhere((l) => l.id == id, orElse: () => _photoLayers.first);
    layer.opacity = opacity.clamp(0.0, 1.0);
    notifyListeners();
  }

  void setLayerBlendMode(String id, LayerBlendMode mode) {
    final layer = _photoLayers.firstWhere((l) => l.id == id, orElse: () => _photoLayers.first);
    layer.blendMode = mode;
    _recordAction('Changed layer blend mode to ${mode.label}');
    notifyListeners();
  }

  void toggleLayerVisibility(String id) {
    final layer = _photoLayers.firstWhere((l) => l.id == id, orElse: () => _photoLayers.first);
    layer.isVisible = !layer.isVisible;
    notifyListeners();
  }

  // --- Retouch Tools ---
  void setActiveRetouchTool(RetouchToolMode mode) {
    _activeRetouchTool = mode;
    notifyListeners();
  }

  void setRetouchRadius(double r) {
    _retouchRadius = r.clamp(2.0, 200.0);
    notifyListeners();
  }

  void setRetouchHardness(double h) {
    _retouchHardness = h.clamp(0.0, 1.0);
    notifyListeners();
  }

  void setRetouchOpacity(double o) {
    _retouchOpacity = o.clamp(0.0, 1.0);
    notifyListeners();
  }

  // --- Batch Photo Export ---
  Future<void> runBatchPhotoProcessing({
    required String namingTemplate,
    required String presetName,
    required String format,
    required int quality,
    required String collisionStrategy,
  }) async {
    _isBatchProcessing = true;
    _batchProgress = 0.0;
    _batchStatusMessage = 'Initializing batch queue...';
    notifyListeners();

    for (int i = 1; i <= 5; i++) {
      await Future.delayed(const Duration(milliseconds: 200));
      _batchProgress = i / 5.0;
      _batchStatusMessage = 'Processed image $i of 5: IMG_20261019_00$i.$format';
      notifyListeners();
    }

    _batchStatusMessage = 'Batch complete: 5 images exported successfully';
    _isBatchProcessing = false;
    _recordAction('Exported batch of 5 photos');
    notifyListeners();
  }

  // --- Export Queue ---
  void addExportJob(ExportJob job) {
    _exportJobs.add(job);
    notifyListeners();
    _startExportProcessing(job);
  }

  void _startExportProcessing(ExportJob job) {
    job.status = ExportStatus.processing;
    notifyListeners();

    int step = 0;
    const totalSteps = 40;
    _exportTimer?.cancel();
    _exportTimer = Timer.periodic(const Duration(milliseconds: 100), (timer) {
      step++;
      job.progressPercent = (step / totalSteps) * 100.0;
      job.currentFps = 48.0 + (step % 5);

      if (step >= totalSteps) {
        timer.cancel();
        job.status = ExportStatus.completed;
        job.progressPercent = 100.0;
        notifyListeners();
      } else {
        notifyListeners();
      }
    });
  }

  void cancelExportJob(String jobId) {
    final job = _exportJobs.firstWhere((j) => j.id == jobId);
    job.status = ExportStatus.cancelled;
    _exportTimer?.cancel();
    notifyListeners();
  }

  // --- Undo / Redo ---
  void _recordAction(String action) {
    _undoStack.add(action);
    _redoStack.clear();
  }

  void undo() {
    if (_undoStack.isNotEmpty) {
      final act = _undoStack.removeLast();
      _redoStack.add(act);
      notifyListeners();
    }
  }

  void redo() {
    if (_redoStack.isNotEmpty) {
      final act = _redoStack.removeLast();
      _undoStack.add(act);
      notifyListeners();
    }
  }

  // --- Keyframe Easing ---
  String _selectedEasing = 'Linear';
  String get selectedEasing => _selectedEasing;
  void setEasing(String easing) {
    _selectedEasing = easing;
    _recordAction('Set Keyframe Easing: $easing');
    notifyListeners();
  }

  // --- Transitions ---
  void addTransition({
    required String kindName,
    double duration = 1.0,
    String? clipAId,
    String? clipBId,
  }) {
    final vTrack = _project.tracks.firstWhere(
      (t) => t.kind == TrackKindModel.video,
      orElse: () => _project.tracks.first,
    );
    if (vTrack.clips.length < 2) return;
    final a = clipAId != null
        ? vTrack.clips.firstWhere((c) => c.id == clipAId)
        : vTrack.clips[0];
    final b = clipBId != null
        ? vTrack.clips.firstWhere((c) => c.id == clipBId)
        : vTrack.clips[1];
    final startTime = (a.timelineStart + a.duration) - (duration / 2);

    final transition = TransitionModel(
      id: 'trans_${DateTime.now().millisecondsSinceEpoch}',
      kindName: kindName,
      startTime: startTime.clamp(0.0, 9999.0),
      duration: duration,
      clipAId: a.id,
      clipBId: b.id,
    );

    _project.transitions.removeWhere(
      (t) => t.clipAId == a.id && t.clipBId == b.id,
    );
    _project.transitions.add(transition);
    _recordAction('Add Transition: $kindName');
    notifyListeners();
  }

  void removeTransition(String id) {
    _project.transitions.removeWhere((t) => t.id == id);
    _recordAction('Remove Transition');
    notifyListeners();
  }

  // --- Audio EQ & Dynamics ---
  void setAudioEq({double? low, double? mid, double? high}) {
    if (low != null) _project.eq.lowGainDb = low;
    if (mid != null) _project.eq.midGainDb = mid;
    if (high != null) _project.eq.highGainDb = high;
    notifyListeners();
  }

  void setAudioCompressor({double? threshold, double? ratio, double? makeup}) {
    if (threshold != null) _project.compressor.thresholdDbfs = threshold;
    if (ratio != null) _project.compressor.ratio = ratio;
    if (makeup != null) _project.compressor.makeupGainDb = makeup;
    notifyListeners();
  }

  // --- Subtitles & Captions ---
  void addSubtitle({required String text, double duration = 3.0}) {
    final sub = SubtitleItemModel(
      id: 'sub_${DateTime.now().millisecondsSinceEpoch}',
      startTime: _playheadSeconds,
      duration: duration,
      text: text,
    );
    _project.subtitles.add(sub);
    _project.subtitles.sort((a, b) => a.startTime.compareTo(b.startTime));
    _recordAction('Add Subtitle: $text');
    notifyListeners();
  }

  void removeSubtitle(String id) {
    _project.subtitles.removeWhere((s) => s.id == id);
    _recordAction('Remove Subtitle');
    notifyListeners();
  }

  void importSrt(String srtContent) {
    final lines = srtContent.replaceAll('\r\n', '\n').split('\n\n');
    for (final block in lines) {
      final blockLines = block
          .split('\n')
          .map((l) => l.trim())
          .where((l) => l.isNotEmpty)
          .toList();
      if (blockLines.length < 2) continue;
      final tsLine = blockLines[0].contains('-->')
          ? blockLines[0]
          : (blockLines.length >= 2 && blockLines[1].contains('-->')
              ? blockLines[1]
              : null);
      if (tsLine == null) continue;
      final parts = tsLine.split('-->');
      if (parts.length != 2) continue;
      final start = _parseSrtTimestamp(parts[0].trim());
      final end = _parseSrtTimestamp(parts[1].trim());
      final text = blockLines.sublist(blockLines.indexOf(tsLine) + 1).join('\n');
      if (text.isNotEmpty) {
        _project.subtitles.add(SubtitleItemModel(
          id: 'sub_imported_${_project.subtitles.length + 1}',
          startTime: start,
          duration: (end - start).clamp(0.5, 60.0),
          text: text,
        ));
      }
    }
    _project.subtitles.sort((a, b) => a.startTime.compareTo(b.startTime));
    _recordAction('Imported SRT Subtitles');
    notifyListeners();
  }

  double _parseSrtTimestamp(String ts) {
    try {
      final parts = ts.replaceAll(',', '.').split(':');
      if (parts.length == 3) {
        final h = double.parse(parts[0]);
        final m = double.parse(parts[1]);
        final s = double.parse(parts[2]);
        return h * 3600.0 + m * 60.0 + s;
      }
    } catch (_) {}
    return 0.0;
  }

  // --- Phase 3: 3D LUT Color Grading ---
  LutPresetModel get lutPreset => _project.lutPreset;
  double get lutIntensity => _project.lutIntensity;
  String? get customCubeLutPath => _project.customCubeLutPath;

  void setLutPreset(LutPresetModel preset) {
    _project.lutPreset = preset;
    notifyListeners();
  }

  void setLutIntensity(double intensity) {
    _project.lutIntensity = intensity.clamp(0.0, 1.0);
    notifyListeners();
  }

  void importCustomCubeLut(String path) {
    _project.customCubeLutPath = path;
    _project.lutPreset = LutPresetModel.customCube;
    notifyListeners();
  }

  // --- Phase 3: RGB Curves ---
  RgbCurvesModel get curves => _project.curves;

  void setCurvePoints(String channel, List<RgbCurvePointModel> points) {
    if (channel == 'master') {
      _project.curves.master = List.from(points);
    } else if (channel == 'red') {
      _project.curves.red = List.from(points);
    } else if (channel == 'green') {
      _project.curves.green = List.from(points);
    } else if (channel == 'blue') {
      _project.curves.blue = List.from(points);
    }
    notifyListeners();
  }

  void resetCurves() {
    _project.curves = RgbCurvesModel();
    notifyListeners();
  }

  // --- Phase 3: Audio Ducking ---
  AudioDuckerModel get ducker => _project.ducker;

  void toggleAudioDucking(bool enabled) {
    _project.ducker.isEnabled = enabled;
    notifyListeners();
  }

  void updateAudioDucking({
    double? thresholdDb,
    double? duckAttenuationDb,
    double? attackMs,
    double? holdMs,
    double? releaseMs,
    String? dialogueTrackId,
  }) {
    if (thresholdDb != null) _project.ducker.thresholdDb = thresholdDb;
    if (duckAttenuationDb != null) {
      _project.ducker.duckAttenuationDb = duckAttenuationDb;
    }
    if (attackMs != null) _project.ducker.attackMs = attackMs;
    if (holdMs != null) _project.ducker.holdMs = holdMs;
    if (releaseMs != null) _project.ducker.releaseMs = releaseMs;
    if (dialogueTrackId != null) {
      _project.ducker.dialogueTrackId = dialogueTrackId;
    }
    notifyListeners();
  }

  // --- Phase 3: Color Scopes ---
  ColorScopeMode get scopeMode => _project.scopeMode;

  void setColorScopeMode(ColorScopeMode mode) {
    _project.scopeMode = mode;
    notifyListeners();
  }

  // --- Phase 3: Pitch Preservation ---
  void toggleClipPitchPreservation(bool preserve) {
    if (_selectedClip != null) {
      _selectedClip!.preservePitch = preserve;
      notifyListeners();
    }
  }

  // --- Phase 4: Chroma Keying ---
  void toggleClipChromaKey(bool enabled) {
    if (_selectedClip != null) {
      _selectedClip!.chromaKey.isEnabled = enabled;
      _recordAction('Chroma key: ${enabled ? "enabled" : "disabled"}');
      notifyListeners();
    }
  }

  void updateClipChromaKey({
    double? r,
    double? g,
    double? b,
    double? similarity,
    double? smoothness,
    double? spillSuppression,
  }) {
    if (_selectedClip != null) {
      if (r != null) _selectedClip!.chromaKey.keyColorR = r.clamp(0.0, 1.0);
      if (g != null) _selectedClip!.chromaKey.keyColorG = g.clamp(0.0, 1.0);
      if (b != null) _selectedClip!.chromaKey.keyColorB = b.clamp(0.0, 1.0);
      if (similarity != null) {
        _selectedClip!.chromaKey.similarity = similarity.clamp(0.01, 1.0);
      }
      if (smoothness != null) {
        _selectedClip!.chromaKey.smoothness = smoothness.clamp(0.001, 1.0);
      }
      if (spillSuppression != null) {
        _selectedClip!.chromaKey.spillSuppression = spillSuppression.clamp(0.0, 1.0);
      }
      notifyListeners();
    }
  }

  void setChromaKeyPreset(String preset) {
    if (_selectedClip != null) {
      if (preset == 'green') {
        _selectedClip!.chromaKey.keyColorR = 0.0;
        _selectedClip!.chromaKey.keyColorG = 1.0;
        _selectedClip!.chromaKey.keyColorB = 0.0;
      } else if (preset == 'blue') {
        _selectedClip!.chromaKey.keyColorR = 0.0;
        _selectedClip!.chromaKey.keyColorG = 0.0;
        _selectedClip!.chromaKey.keyColorB = 1.0;
      }
      _selectedClip!.chromaKey.isEnabled = true;
      _recordAction('Set Chroma Key Preset: $preset');
      notifyListeners();
    }
  }

  // --- Phase 4: Motion Graphics Titles ---
  List<MotionTitleModel> get motionTitles => _project.motionTitles;

  MotionTitleModel? get activeMotionTitle {
    for (final title in _project.motionTitles) {
      if (_playheadSeconds >= title.startTime && _playheadSeconds <= title.endTime) {
        return title;
      }
    }
    return null;
  }

  void addMotionTitle({
    required String title,
    required String subtitle,
    MotionTitleTemplate template = MotionTitleTemplate.minimalClean,
    double duration = 4.0,
  }) {
    final item = MotionTitleModel(
      id: 'title_${DateTime.now().millisecondsSinceEpoch}',
      title: title,
      subtitle: subtitle,
      template: template,
      startTime: _playheadSeconds,
      duration: duration,
    );
    _project.motionTitles.add(item);
    _project.motionTitles.sort((a, b) => a.startTime.compareTo(b.startTime));
    _recordAction('Add Motion Title: $title');
    notifyListeners();
  }

  void removeMotionTitle(String id) {
    _project.motionTitles.removeWhere((t) => t.id == id);
    _recordAction('Remove Motion Title');
    notifyListeners();
  }

  // --- Phase 4: Hardware Acceleration ---
  HardwareAccelerationInfo get hardwareInfo => _project.hardwareInfo;

  void refreshHardwareInfo() {
    notifyListeners();
  }

  // --- Stream Copy Optimization ---
  double get streamCopyPercentage {
    final vTracks = _project.tracks
        .where((t) => t.kind == TrackKindModel.video && !t.isMuted)
        .toList();
    if (vTracks.length != 1) return 0.0;
    final track = vTracks.first;
    if (track.clips.isEmpty) return 0.0;

    double copyDuration = 0.0;
    for (final clip in track.clips) {
      final isDefault = (clip.scale - 1.0).abs() < 1e-3 &&
          clip.rotation.abs() < 1e-3 &&
          (clip.opacity - 1.0).abs() < 1e-3 &&
          (clip.speed - 1.0).abs() < 1e-3;
      final hasTransition = _project.transitions.any(
        (t) => t.clipAId == clip.id || t.clipBId == clip.id,
      );
      if (isDefault && !hasTransition) {
        copyDuration += clip.duration;
      }
    }
    final total = _project.totalDuration;
    return total > 0 ? ((copyDuration / total) * 100.0).clamp(0.0, 100.0) : 0.0;
  }

  double get estimatedSpeedup {
    final pct = streamCopyPercentage;
    return 1.0 + (pct / 100.0) * 24.0;
  }

  void _simulateAudioMeters() {
    final random = math.Random();
    _peakLeft = (0.3 + random.nextDouble() * 0.45).clamp(0.0, 1.0);
    _peakRight = (0.28 + random.nextDouble() * 0.48).clamp(0.0, 1.0);
    _leftDbfs = (-30.0 + random.nextDouble() * 24.0).clamp(-60.0, 0.0);
    _rightDbfs = (-28.0 + random.nextDouble() * 22.0).clamp(-60.0, 0.0);
  }

  static ProjectModel _createDefaultProject() {
    final vTrack = TrackModel(
      id: 'trk_v1',
      name: 'V1 Video Track',
      kind: TrackKindModel.video,
      clips: [
        ClipModel(
          id: 'clip_1',
          assetId: 'scene_01',
          name: 'Golden Hour Intro.mp4',
          timelineStart: 0.0,
          duration: 4.5,
        ),
        ClipModel(
          id: 'clip_2',
          assetId: 'scene_02',
          name: 'Mountain Panorama.mov',
          timelineStart: 4.5,
          duration: 6.0,
        ),
      ],
    );

    final aTrack = TrackModel(
      id: 'trk_a1',
      name: 'A1 Master Audio',
      kind: TrackKindModel.audio,
      clips: [
        ClipModel(
          id: 'clip_a1',
          assetId: 'ambient_score',
          name: 'Atmospheric Score.wav',
          timelineStart: 0.0,
          duration: 10.5,
          volume: 0.9,
        ),
      ],
    );

    final overlayTrack = TrackModel(
      id: 'trk_ov1',
      name: 'V2 Title & Overlays',
      kind: TrackKindModel.overlay,
      clips: [
        ClipModel(
          id: 'clip_t1',
          assetId: 'title_card',
          name: 'Iris Studio Premiere',
          timelineStart: 1.0,
          duration: 3.0,
          opacity: 0.95,
        ),
      ],
    );

    final demoTransition = TransitionModel(
      id: 'trans_demo_1',
      kindName: 'Iris Round',
      startTime: 4.0,
      duration: 1.0,
      clipAId: 'clip_1',
      clipBId: 'clip_2',
    );

    final demoSubtitle = SubtitleItemModel(
      id: 'sub_demo_1',
      startTime: 1.0,
      duration: 3.5,
      text: 'Iris Studio: High-Performance Offline Editor',
    );

    final demoMotionTitle = MotionTitleModel(
      id: 'title_demo_1',
      title: 'IRIS STUDIO',
      subtitle: 'High-Performance Rust Workstation',
      template: MotionTitleTemplate.lowerThirdNews,
      startTime: 0.5,
      duration: 4.0,
    );

    return ProjectModel(
      id: 'prj_demo_01',
      name: 'Iris Studio Showcase Project',
      fps: 30,
      width: 1920,
      height: 1080,
      tracks: [overlayTrack, vTrack, aTrack],
      transitions: [demoTransition],
      subtitles: [demoSubtitle],
      motionTitles: [demoMotionTitle],
    );
  }

  @override
  void dispose() {
    _playbackTimer?.cancel();
    _exportTimer?.cancel();
    super.dispose();
  }
}

extension ListLen<T> on List<T> {
  int get len => length;
}
