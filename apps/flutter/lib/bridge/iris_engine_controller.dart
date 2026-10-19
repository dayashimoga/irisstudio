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

    return ProjectModel(
      id: 'prj_demo_01',
      name: 'Iris Studio Showcase Project',
      fps: 30,
      width: 1920,
      height: 1080,
      tracks: [overlayTrack, vTrack, aTrack],
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
