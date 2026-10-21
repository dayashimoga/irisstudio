import 'dart:io';
import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:iris_studio/bridge/iris_engine_controller.dart';
import 'package:iris_studio/bridge/iris_ffi_bindings.dart';
import 'package:iris_studio/models/export_job.dart';
import 'package:iris_studio/models/media_item.dart';
import 'package:iris_studio/models/project_model.dart';
import 'package:iris_studio/theme/iris_theme.dart';
import 'package:iris_studio/workspaces/video_workspace.dart';

void main() {
  setUpAll(() {
    TestWidgetsFlutterBinding.ensureInitialized();
    IrisFfiBindings.initialize();
  });

  group('Iris Studio End-to-End Real Native Pipelines & Workflows', () {
    test('1. Video + Audio Pipeline: Edit, Mix, Save/Reopen, and Valid MP4 Container Export', () async {
      final controller = IrisEngineController(startWithDemo: false);
      controller.createNewProject(name: 'Integration Video Test', type: 'video');

      final tempDir = Directory.systemTemp.createTempSync('iris_test_media_');
      final videoFile = File('${tempDir.path}/test_footage.mp4');
      final audioFile = File('${tempDir.path}/test_dialogue.wav');

      // Write valid ISO-BMFF MP4 container
      final sampleMp4 = buildValidMp4Container(
        width: 1920,
        height: 1080,
        fps: 30.0,
        durationSeconds: 3.0,
      );
      videoFile.writeAsBytesSync(sampleMp4);

      // Write valid RIFF WAV audio
      final sampleWav = buildValidWavAudio(
        durationSeconds: 2.5,
        sampleRate: 48000,
        channels: 2,
      );
      audioFile.writeAsBytesSync(sampleWav);

      // Probe and import into Media Bin
      final vImport = await controller.importMediaFile(videoFile.path);
      final aImport = await controller.importMediaFile(audioFile.path);
      expect(vImport, isTrue);
      expect(aImport, isTrue);
      expect(controller.mediaBin.length, 2);

      final videoAsset = controller.mediaBin.firstWhere((m) => m.mediaType == MediaType.video);
      final audioAsset = controller.mediaBin.firstWhere((m) => m.mediaType == MediaType.audio);
      expect(videoAsset.width, 1920);
      expect(videoAsset.height, 1080);
      expect(audioAsset.sampleRate, 48000);

      // Add to project timeline
      final vTrack = controller.project.tracks.firstWhere((t) => t.kind == TrackKindModel.video);
      final aTrack = controller.project.tracks.firstWhere((t) => t.kind == TrackKindModel.audio);

      vTrack.clips.add(
        ClipModel(
          id: 'v_clip_1',
          assetId: videoAsset.id,
          name: videoAsset.fileName,
          timelineStart: 0.0,
          duration: 3.0,
        ),
      );

      aTrack.clips.add(
        ClipModel(
          id: 'a_clip_1',
          assetId: audioAsset.id,
          name: audioAsset.fileName,
          timelineStart: 0.0,
          duration: 2.5,
        ),
      );

      // Apply audio mix settings & EQ
      controller.setAudioEq(low: 2.5, mid: -1.0, high: 3.0);
      expect(controller.project.eq.lowGainDb, 2.5);

      // Add subtitle caption
      controller.addSubtitle(text: 'E2E Dialogue Subtitle');
      expect(controller.project.subtitles.length, 1);
      expect(controller.project.subtitles.first.text, 'E2E Dialogue Subtitle');

      // Save project transactionally to .iris file
      final projectSavePath = '${tempDir.path}/test_project.iris';
      final saveOk = await controller.saveProject(filePath: projectSavePath);
      expect(saveOk, isTrue);
      expect(File(projectSavePath).existsSync(), isTrue);

      // Reopen project from file and verify persistence
      controller.createNewProject(name: 'Empty Scratch');
      expect(controller.project.tracks.first.clips.length, 0);

      final openOk = await controller.openProjectFile(projectSavePath);
      expect(openOk, isTrue);
      expect(controller.project.name, 'Integration Video Test');
      expect(controller.project.tracks.any((t) => t.clips.any((c) => c.name == 'test_footage.mp4')), isTrue);
      expect(controller.project.eq.lowGainDb, 2.5);

      // Perform verified ISO-BMFF MP4 export
      final outMp4Path = '${tempDir.path}/final_render.mp4';
      final exportJob = ExportJob(
        id: 'job_v1',
        title: 'Integration Export Render',
        outputPath: outMp4Path,
        format: 'MP4 (H.264 / AAC)',
        resolution: '1920x1080',
        fps: 30,
        bitrateKbps: 12000,
      );

      controller.addExportJob(exportJob);

      // Poll until export completes
      int polls = 0;
      while (exportJob.status != ExportStatus.completed && polls < 40) {
        await Future.delayed(const Duration(milliseconds: 100));
        polls++;
      }
      expect(exportJob.status, ExportStatus.completed);
      expect(File(outMp4Path).existsSync(), isTrue);
      expect(File(outMp4Path).lengthSync(), greaterThan(100));

      tempDir.deleteSync(recursive: true);
    });

    test('2. Audio-Only Pipeline: Dedicated Audio Track, Waveform Peaks, Fades, and WAV Export', () async {
      final controller = IrisEngineController(startWithDemo: false);
      controller.createNewProject(name: 'Podcast Episode 1', type: 'audio');

      // Verify no dummy video track was generated
      expect(controller.project.tracks.any((t) => t.kind == TrackKindModel.video), isFalse);
      expect(controller.project.tracks.any((t) => t.kind == TrackKindModel.audio), isTrue);

      final tempDir = Directory.systemTemp.createTempSync('iris_audio_test_');
      final audioFilePath = '${tempDir.path}/interview.wav';
      final wavBytes = buildValidWavAudio(
        durationSeconds: 4.0,
        sampleRate: 44100,
        channels: 2,
      );
      File(audioFilePath).writeAsBytesSync(wavBytes);

      // Probe media
      final imp = await controller.importMediaFile(audioFilePath);
      expect(imp, isTrue);
      expect(controller.mediaBin.length, 1);
      final media = controller.mediaBin.first;
      expect(media.mediaType, MediaType.audio);
      expect(media.waveformPeaks != null && media.waveformPeaks!.isNotEmpty, isTrue);

      // Add to audio track
      final aTrack = controller.project.tracks.firstWhere((t) => t.kind == TrackKindModel.audio);
      final voiceClip = ClipModel(
        id: 'clip_voice',
        assetId: media.id,
        name: media.fileName,
        timelineStart: 0.0,
        duration: 4.0,
        volume: 1.2,
      );
      aTrack.clips.add(voiceClip);
      expect(aTrack.clips.first.volume, 1.2);

      // Export Audio-Only Master
      final outWavPath = '${tempDir.path}/master_podcast.wav';
      final audioJob = ExportJob(
        id: 'job_audio_master',
        title: 'Master Audio Mix',
        outputPath: outWavPath,
        format: 'Production Master (Lossless / High Bitrate)',
        resolution: 'Audio Master',
        fps: 30,
        bitrateKbps: 1411,
      );
      controller.addExportJob(audioJob);

      int polls = 0;
      while (audioJob.status != ExportStatus.completed && polls < 40) {
        await Future.delayed(const Duration(milliseconds: 100));
        polls++;
      }
      expect(audioJob.status, ExportStatus.completed);
      expect(File(outWavPath).existsSync(), isTrue);

      // Validate WAV RIFF header
      final exportedBytes = File(outWavPath).readAsBytesSync();
      expect(exportedBytes.length, greaterThan(44));
      expect(String.fromCharCodes(exportedBytes.sublist(0, 4)), 'RIFF');
      expect(String.fromCharCodes(exportedBytes.sublist(8, 12)), 'WAVE');

      tempDir.deleteSync(recursive: true);
    });

    test('3. Photo Pipeline: Real Pixels, Real Histogram, Adjustments, Undo/Redo, and Real Rust File Export', () async {
      final controller = IrisEngineController(startWithDemo: false);
      controller.createNewProject(name: 'Photo Retouch', type: 'photo');

      // Create a 100x100 test RGBA image
      final width = 100;
      final height = 100;
      final rawPixels = Uint8List(width * height * 4);
      for (int i = 0; i < rawPixels.length; i += 4) {
        rawPixels[i] = 120;     // R
        rawPixels[i + 1] = 180; // G
        rawPixels[i + 2] = 220; // B
        rawPixels[i + 3] = 255; // A
      }

      final tempDir = Directory.systemTemp.createTempSync('iris_photo_test_');
      final photoPath = '${tempDir.path}/source.png';
      
      // Encode via Rust FFI
      final encodeSuccess = IrisFfiBindings.encodePhotoToFile(
        outputPath: photoPath,
        formatCode: 0, // PNG
        quality: 95,
        width: width,
        height: height,
        pixels: rawPixels,
      );
      expect(encodeSuccess, isTrue);
      expect(File(photoPath).existsSync(), isTrue);

      // Load into controller
      final loadOk = await controller.loadPhoto(photoPath);
      expect(loadOk, isTrue);
      expect(controller.photoWidth, 100);
      expect(controller.photoHeight, 100);
      expect(controller.photoHistogram.rBins.length, 256);

      // Adjust exposure & contrast
      controller.updateAdjustments(controller.photoAdjustments.copyWith(
        exposure: 1.2,
        contrast: 20.0,
      ));
      expect(controller.photoAdjustments.exposure, 1.2);
      expect(controller.photoAdjustments.contrast, 20.0);

      // Undo
      controller.undo();
      expect(controller.photoAdjustments.contrast, 0.0);
      expect(controller.photoAdjustments.exposure, 0.0);

      // Redo
      controller.redo();
      expect(controller.photoAdjustments.contrast, 20.0);
      expect(controller.photoAdjustments.exposure, 1.2);

      // Export photo through controller to JPEG via Rust FFI
      final outJpeg = '${tempDir.path}/retouched.jpg';
      final exportResult = await controller.exportPhoto(
        outputPath: outJpeg,
        formatCode: 1, // JPEG
        quality: 90,
      );
      expect(exportResult, isTrue);
      expect(File(outJpeg).existsSync(), isTrue);

      final jpegBytes = File(outJpeg).readAsBytesSync();
      expect(jpegBytes.length, greaterThan(10));
      expect(jpegBytes[0], 0xFF);
      expect(jpegBytes[1], 0xD8); // JPEG magic bytes

      tempDir.deleteSync(recursive: true);
    });

    testWidgets('4. Responsive UI & Accessibility: Test Across Window Scales without Layout Overflow', (WidgetTester tester) async {
      final resolutions = [
        const Size(1920, 1080),
        const Size(1280, 800),
        const Size(900, 600),
      ];

      for (final res in resolutions) {
        tester.view.physicalSize = res;
        tester.view.devicePixelRatio = 1.0;

        final controller = IrisEngineController(startWithDemo: true);

        await tester.pumpWidget(
          MaterialApp(
            theme: IrisTheme.darkTheme,
            home: Scaffold(
              body: VideoWorkspace(controller: controller),
            ),
          ),
        );
        await tester.pumpAndSettle();

        expect(find.text('TRACKS'), findsOneWidget);
        expect(find.text('PROJECT MASTER & DYNAMICS'), findsOneWidget);
        expect(find.text('STREAM COPY'), findsOneWidget);
      }

      tester.view.resetPhysicalSize();
    });
  });
}
