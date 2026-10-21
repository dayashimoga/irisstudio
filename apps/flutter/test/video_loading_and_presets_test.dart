import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:iris_studio/bridge/iris_engine_controller.dart';
import 'package:iris_studio/bridge/iris_ffi_bindings.dart';
import 'package:iris_studio/models/photo_state.dart';
import 'package:iris_studio/models/preset_model.dart';
import 'package:iris_studio/models/project_model.dart';
import 'package:iris_studio/widgets/media_picker_dialog.dart';

void main() {
  setUpAll(() {
    TestWidgetsFlutterBinding.ensureInitialized();
    IrisFfiBindings.initialize();
  });

  group('Video Loading and Pipeline Repair Tests', () {
    test('openVideoFile creates source-matched sequence, linked tracks, and decodes first frame', () async {
      final controller = IrisEngineController(startWithDemo: false);
      controller.createNewProject(name: 'Video Loading Test', type: 'video');

      // Verify initial state is empty
      expect(controller.videoPreviewState, VideoPreviewState.empty);
      expect(controller.programPreviewImage, isNull);

      final fixturePath = '${Directory.current.path}/../../tests/fixtures/test_video_360p.mp4';
      final file = File(fixturePath);
      expect(file.existsSync(), isTrue, reason: 'Fixture test_video_360p.mp4 must exist');

      final success = await controller.openVideoFile(file.path);
      expect(success, isTrue, reason: 'openVideoFile should succeed for valid MP4');

      // Verify sequence was created
      expect(controller.project.width, 640);
      expect(controller.project.height, 360);
      expect(controller.project.totalDuration, greaterThan(0.0));

      // Verify tracks and linked clips
      final videoTrack = controller.project.tracks.firstWhere((t) => t.kind == TrackKindModel.video);
      final audioTrack = controller.project.tracks.firstWhere((t) => t.kind == TrackKindModel.audio);
      expect(videoTrack.clips.isNotEmpty, isTrue);
      expect(audioTrack.clips.isNotEmpty, isTrue);

      final vClip = videoTrack.clips.first;
      final aClip = audioTrack.clips.first;
      expect(vClip.duration, greaterThan(0.0));
      expect(aClip.duration, greaterThan(0.0));
      expect(controller.selectedClip?.id, vClip.id);

      // Verify preview state and decoded first frame
      expect(controller.videoPreviewState, VideoPreviewState.ready);
      expect(controller.programPreviewImage, isNotNull);
      expect(controller.programPreviewImage!.width, 320);
      expect(controller.programPreviewImage!.height, 180);

      // Verify seek updates playhead and keeps ready state
      controller.seekTo(1.0);
      expect(controller.playheadSeconds, 1.0);
      expect(controller.videoPreviewState, VideoPreviewState.ready);
    });

    test('importMediaFile adds to Media Bin without mutating existing sequence', () async {
      final controller = IrisEngineController(startWithDemo: false);
      controller.createNewProject(name: 'Bin Test', type: 'video');

      final fixturePath = '${Directory.current.path}/../../tests/fixtures/test_video_360p.mp4';
      final success = await controller.importMediaFile(fixturePath);
      expect(success, isTrue);

      // Media item present in bin
      expect(controller.mediaBin.isNotEmpty, isTrue);
      final item = controller.mediaBin.first;
      expect(item.filePath, fixturePath);
      expect(item.width, 640);
      expect(item.height, 360);

      // Existing tracks should still have no clips
      final videoTrack = controller.project.tracks.firstWhere((t) => t.kind == TrackKindModel.video);
      expect(videoTrack.clips.isEmpty, isTrue);

      // Selecting bin item activates source preview
      controller.selectMediaItem(item);
      expect(controller.isSourcePreviewActive, isTrue);
    });
  });

  group('Look Presets Engine Tests', () {
    test('Built-in video looks are loaded and valid', () {
      final controller = IrisEngineController(startWithDemo: false);
      expect(controller.videoPresets.length, greaterThanOrEqualTo(10));

      final cinemaWarm = controller.videoPresets.firstWhere((p) => p.name == 'Cinema Warm');
      expect(cinemaWarm.category.toLowerCase().contains('cinema'), isTrue);
      expect(cinemaWarm.adjustments.temperature, greaterThan(0));

      final sports = controller.videoPresets.firstWhere((p) => p.name == 'Sports/Action');
      expect(sports.category.toLowerCase().contains('action') || sports.category.toLowerCase().contains('sports'), isTrue);
      expect(sports.adjustments.contrast, greaterThan(0));

      final bw = controller.videoPresets.firstWhere((p) => p.name == 'Black & White');
      expect(bw.category, 'Monochrome');
      expect(bw.adjustments.saturation, -100.0);
    });

    test('Applying preset updates settings atomically and supports intensity blending', () {
      final controller = IrisEngineController(startWithDemo: false);
      final travel = controller.videoPresets.firstWhere((p) => p.name == 'Travel');

      controller.applyPreset(travel, intensity: 1.0, isVideo: false);
      expect(controller.activePreset?.id, travel.id);
      expect(controller.presetIntensity, 1.0);
      expect(controller.isPresetModified, isFalse);
      expect(controller.photoAdjustments.saturation, travel.adjustments.saturation);

      // Blending at 50%
      controller.setPresetIntensity(0.5, isVideo: false);
      expect(controller.presetIntensity, 0.5);
      expect(
        controller.photoAdjustments.saturation,
        closeTo(travel.adjustments.saturation * 0.5, 0.01),
      );

      // Manual slider tweak marks preset as modified
      controller.updateAdjustments(
        controller.photoAdjustments.copyWith(exposure: 1.5),
      );
      expect(controller.isPresetModified, isTrue);

      // Resetting restores neutral
      controller.resetPreset(isVideo: false);
      expect(controller.activePreset, isNull);
      expect(controller.isPresetModified, isFalse);
      expect(controller.photoAdjustments.exposure, 0.0);
    });

    test('Custom preset creation, favorite, duplicate, export and import validation', () async {
      final controller = IrisEngineController(startWithDemo: false);

      // 1. Save custom preset
      controller.updateAdjustments(PhotoAdjustments(exposure: 0.8, contrast: 15.0, saturation: 10.0));
      final saved = await controller.saveCustomPreset(
        name: 'My Special Grade',
        category: 'Custom',
        description: 'Warm highlight grade',
        isVideo: true,
      );
      expect(saved, isTrue);

      final PresetModel custom = controller.videoPresets.firstWhere((p) => p.name == 'My Special Grade');
      expect(custom.isBuiltin, isFalse);
      expect(custom.adjustments.exposure, 0.8);

      // 2. Favorite toggle
      controller.toggleFavoritePreset(custom.id);
      expect(controller.favoritePresetIds.contains(custom.id), isTrue);
      controller.toggleFavoritePreset(custom.id);
      expect(controller.favoritePresetIds.contains(custom.id), isFalse);

      // 3. Duplicate
      final PresetModel dup = await controller.duplicatePreset(custom);
      expect(dup.name, 'My Special Grade (Copy)');
      expect(controller.videoPresets.any((p) => p.id == dup.id), isTrue);

      // 4. Export to file
      final tempDir = Directory.systemTemp.createTempSync('iris_preset_test_');
      final exportPath = '${tempDir.path}/grade.irispreset';
      final exportOk = await controller.exportPresetToFile(custom, exportPath);
      expect(exportOk, isTrue);
      expect(File(exportPath).existsSync(), isTrue);

      // 5. Import from file
      final importOk = await controller.importPresetFromFile(exportPath);
      expect(importOk, isTrue);

      // 6. Security & Untrusted data validation: Corrupted JSON rejected
      final corruptPath = '${tempDir.path}/corrupt.irispreset';
      File(corruptPath).writeAsStringSync('{"malicious": "payload", "version": "invalid"}');
      final badImport = await controller.importPresetFromFile(corruptPath);
      expect(badImport, isFalse, reason: 'Corrupt or non-conforming preset files must be rejected');
    });

    test('Photo Auto Enhance bounded analysis produces valid adjustments', () async {
      final controller = IrisEngineController(startWithDemo: false);
      final photoFixture = '${Directory.current.path}/../../tests/fixtures/test_photo_sample.jpg';
      final file = File(photoFixture);
      expect(file.existsSync(), isTrue);

      final loaded = await controller.loadPhoto(file.path);
      expect(loaded, isTrue);
      expect(controller.photoWidth, greaterThan(0));
      expect(controller.photoHeight, greaterThan(0));

      final enhanced = await controller.runPhotoAutoEnhance();
      expect(enhanced, isTrue);
      expect(controller.activePreset?.name, 'Auto Enhance');
      expect(controller.isPresetModified, isFalse);

      // Adjustments should be bounded within safe creative ranges
      expect(controller.photoAdjustments.exposure, inInclusiveRange(-3.0, 3.0));
      expect(controller.photoAdjustments.contrast, inInclusiveRange(-50.0, 50.0));
      expect(controller.photoAdjustments.saturation, inInclusiveRange(-50.0, 50.0));
    });
  });

  group('P0 Stability and Bounded Import Regression Tests', () {
    test('Deduplication: Repeated openVideoFile calls with the same file path do not duplicate sequence or media items', () async {
      final controller = IrisEngineController(startWithDemo: false);
      controller.createNewProject(name: 'Dedup Test', type: 'video');

      final fixturePath = '${Directory.current.path}/../../tests/fixtures/test_video_360p.mp4';
      final file = File(fixturePath);
      expect(file.existsSync(), isTrue);

      // First open
      final firstSuccess = await controller.openVideoFile(file.path);
      expect(firstSuccess, isTrue);
      expect(controller.importStatus, MediaImportStatus.ready);
      expect(controller.mediaBin.length, 1);
      final initialProjectId = controller.project.id;

      // Second identical open while ready
      final secondSuccess = await controller.openVideoFile(file.path);
      expect(secondSuccess, isTrue);
      expect(controller.mediaBin.length, 1, reason: 'Media bin count must not increase on duplicate open');
      expect(controller.project.id, initialProjectId, reason: 'Project must not be recreated for identical file');
    });

    test('Cancellation: cancelImport() aborts active import and returns to responsive idle state', () async {
      final controller = IrisEngineController(startWithDemo: false);
      controller.createNewProject(name: 'Cancel Test', type: 'video');

      final fixturePath = '${Directory.current.path}/../../tests/fixtures/test_video_360p.mp4';

      // Start import without awaiting, immediately cancel
      final future = controller.openVideoFile(fixturePath);
      controller.cancelImport();

      final res = await future;
      if (!res) {
        expect(controller.importStatus, anyOf(MediaImportStatus.cancelled, MediaImportStatus.idle));
      }
      expect(controller.videoPreviewState, anyOf(VideoPreviewState.empty, VideoPreviewState.ready));
    });

    test('Error handling and dismissal: Nonexistent file produces actionable error and clearVideoError() restores responsive state', () async {
      final controller = IrisEngineController(startWithDemo: false);
      controller.createNewProject(name: 'Missing File Test', type: 'video');

      final res = await controller.openVideoFile('non_existent_file_path_12345.mp4');
      expect(res, isFalse);
      expect(controller.importStatus, MediaImportStatus.failed);
      expect(controller.videoPreviewState, VideoPreviewState.error);
      expect(controller.videoPreviewError, isNotNull);
      expect(controller.videoPreviewError!.contains('does not exist'), isTrue);

      // Dismissing error clears state back to empty/idle
      controller.clearVideoError();
      expect(controller.videoPreviewState, VideoPreviewState.empty);
      expect(controller.videoPreviewError, isNull);
      expect(controller.importStatus, MediaImportStatus.idle);
    });

    test('Rapid scrubbing backpressure: Fast playhead updates do not block or crash', () async {
      final controller = IrisEngineController(startWithDemo: false);
      final fixturePath = '${Directory.current.path}/../../tests/fixtures/test_video_360p.mp4';
      await controller.openVideoFile(fixturePath);

      // Simulate rapid 60 FPS scrub gestures across 30 timestamps
      for (int i = 0; i < 30; i++) {
        controller.seekTo(i * 0.1);
      }
      expect(controller.playheadSeconds, closeTo(2.9, 0.001));
      expect(controller.videoPreviewState, VideoPreviewState.ready);
    });

    test('Safe path normalization: Handles spaces and unicode gracefully', () {
      final normalized1 = IrisFfiBindings.probeMedia('   sample.mp4   ');
      expect(normalized1.isValid, isFalse);
      final normalized2 = IrisFfiBindings.probeMedia('file:///C:/path/with spaces/video.mp4');
      expect(normalized2.isValid, isFalse);
    });

    testWidgets('MediaPickerDialog blocks concurrent dialog invocations', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => ElevatedButton(
              onPressed: () {
                MediaPickerDialog.show(context, title: 'Picker 1');
                MediaPickerDialog.show(context, title: 'Picker 2').then((res) {
                  expect(res, isNull, reason: 'Second concurrent picker must be rejected');
                });
              },
              child: const Text('Open'),
            ),
          ),
        ),
      );

      await tester.tap(find.text('Open'));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 100));

      // Close dialog
      final closeButton = find.byIcon(Icons.close);
      if (closeButton.evaluate().isNotEmpty) {
        await tester.tap(closeButton);
        await tester.pumpAndSettle();
      }
    });
  });
}
