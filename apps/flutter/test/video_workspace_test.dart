import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:iris_studio/bridge/iris_engine_controller.dart';
import 'package:iris_studio/models/project_model.dart';
import 'package:iris_studio/theme/iris_theme.dart';
import 'package:iris_studio/workspaces/video_workspace.dart';

void main() {
  testWidgets('Video Workspace - Inspector, Transitions, EQ, and Subtitles',
      (WidgetTester tester) async {
    tester.view.physicalSize = const Size(1400, 900);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.resetPhysicalSize);

    final controller = IrisEngineController();

    await tester.pumpWidget(
      MaterialApp(
        theme: IrisTheme.darkTheme,
        home: Scaffold(
          body: VideoWorkspace(controller: controller),
        ),
      ),
    );
    await tester.pumpAndSettle();

    // 1. Verify timeline and player core components
    expect(find.text('TRACKS'), findsOneWidget);
    expect(find.text('Split (S)'), findsOneWidget);
    expect(find.text('PROJECT MASTER & DYNAMICS'), findsOneWidget);

    // 2. Verify Stream Copy Optimizer Card
    expect(find.text('STREAM COPY'), findsOneWidget);
    expect(find.textContaining('Speedup'), findsOneWidget);

    // 3. Verify Master 3-Band Parametric EQ & Compressor
    expect(find.text('3-BAND PARAMETRIC EQ'), findsOneWidget);
    expect(find.text('Low Shelf (120 Hz)'), findsOneWidget);
    expect(find.text('Mid Peaking (1 kHz)'), findsOneWidget);
    expect(find.text('High Shelf (8 kHz)'), findsOneWidget);
    expect(find.text('AUDIO DYNAMICS COMPRESSOR'), findsOneWidget);
    expect(find.text('Threshold'), findsOneWidget);
    expect(find.text('Ratio'), findsOneWidget);

    // Adjust Low Shelf EQ
    controller.setAudioEq(low: 4.5);
    await tester.pumpAndSettle();
    expect(controller.project.eq.lowGainDb, 4.5);

    // 4. Verify Subtitles & Captions
    expect(find.text('SUBTITLES & CAPTIONS'), findsOneWidget);
    expect(find.text('Import SRT'), findsOneWidget);

    // Add a new caption at playhead
    controller.addSubtitle(text: 'Test Video Caption');
    await tester.pumpAndSettle();
    expect(find.text('Test Video Caption'), findsAtLeastNWidgets(1));

    // Phase 3: Verify 3D LUT Color Grading
    await tester.ensureVisible(find.text('3D LUT COLOR GRADING'));
    expect(find.text('3D LUT COLOR GRADING'), findsOneWidget);
    await tester.tap(find.text('Teal & Orange'));
    await tester.pumpAndSettle();
    expect(controller.lutPreset, LutPresetModel.tealAndOrange);
    expect(find.text('LUT Intensity'), findsOneWidget);

    // Phase 3: Verify Parametric RGB Curves
    await tester.ensureVisible(find.text('PARAMETRIC RGB CURVES'));
    expect(find.text('PARAMETRIC RGB CURVES'), findsOneWidget);

    // Phase 3: Verify Audio Ducking
    await tester.ensureVisible(find.text('AUDIO DUCKING (SIDECHAIN)'));
    expect(find.text('AUDIO DUCKING (SIDECHAIN)'), findsOneWidget);
    controller.toggleAudioDucking(true);
    await tester.pumpAndSettle();
    expect(controller.ducker.isEnabled, isTrue);
    expect(find.text('Voice Threshold'), findsOneWidget);

    // Phase 3: Verify Color Scopes bar
    expect(find.text('SCOPES:'), findsOneWidget);
    await tester.tap(find.text('Waveform'));
    await tester.pumpAndSettle();
    expect(controller.scopeMode, ColorScopeMode.waveform);
    await tester.tap(find.text('Vectorscope'));
    await tester.pumpAndSettle();
    expect(controller.scopeMode, ColorScopeMode.vectorscope);

    // Phase 4: Verify Hardware Acceleration Context Card
    await tester.ensureVisible(find.text('HARDWARE ACCEL'));
    expect(find.text('HARDWARE ACCEL'), findsOneWidget);
    expect(
      find.textContaining(RegExp(r'(D3D11VA|VA-API|VideoToolbox|MediaCodec|Acceleration|Driver)')),
      findsAtLeastNWidgets(1),
    );

    // Phase 4: Verify Motion Graphics Titles & Lower-Thirds Card
    await tester.ensureVisible(find.text('MOTION GRAPHICS TITLES'));
    expect(find.text('MOTION GRAPHICS TITLES'), findsOneWidget);
    expect(find.text('Clean'), findsOneWidget);
    expect(find.text('Cinematic'), findsOneWidget);
    await tester.tap(find.text('Cinematic'));
    await tester.pumpAndSettle();
    expect(find.text('CINEMATIC'), findsAtLeastNWidgets(1));

    // Phase 5: Verify Multi-Camera Switcher in Master Inspector
    await tester.ensureVisible(find.text('MULTI-CAM SWITCHER'));
    expect(find.text('MULTI-CAM SWITCHER'), findsOneWidget);
    expect(find.text('CAM: WIDE'), findsOneWidget);
    expect(find.text('CAM: CLOSE-UP'), findsOneWidget);
    expect(find.text('CAM: DRONE'), findsOneWidget);
    expect(find.text('CAM: STAGE'), findsOneWidget);

    // Switch angle to CLOSE-UP
    await tester.tap(find.text('CAM: CLOSE-UP'));
    await tester.pumpAndSettle();
    expect(controller.multiCam.activeAngleId, 'cam_2');
    expect(find.text('LIVE: CLOSE-UP'), findsOneWidget);

    // Phase 5: Verify Quad View 4-Up Monitor toggle in Scopes bar and Viewport
    expect(find.text('Quad View'), findsOneWidget);
    await tester.tap(find.text('Quad View'));
    await tester.pumpAndSettle();
    expect(controller.multiCam.isQuadViewActive, isTrue);

    // Verify 4-up quad monitor rendering
    expect(find.text('CAM 1'), findsOneWidget);
    expect(find.text('CAM 2'), findsOneWidget);
    expect(find.text('CAM 3'), findsOneWidget);
    expect(find.text('CAM 4'), findsOneWidget);
    expect(find.text('ON AIR'), findsOneWidget);
    expect(find.text('PREVIEW'), findsNWidgets(3));

    // Tap CAM 3 in quad view or switcher
    await tester.tap(find.text('CAM 3'));
    await tester.pumpAndSettle();
    expect(controller.multiCam.activeAngleId, 'cam_3');

    // Turn off Quad View
    controller.toggleMultiCamQuadView(false);
    await tester.pumpAndSettle();
    expect(controller.multiCam.isQuadViewActive, isFalse);

    // Phase 5: Verify HDR Tone Mapping Card in Master Inspector
    await tester.ensureVisible(find.text('HDR TONE MAPPING'));
    expect(find.text('HDR TONE MAPPING'), findsOneWidget);

    controller.updateHdrSettings(isEnabled: true);
    await tester.pumpAndSettle();
    expect(controller.hdrSettings.isEnabled, isTrue);
    expect(find.text('Transfer Function (EOTF / OETF):'), findsOneWidget);
    expect(find.text('PQ (ST 2084)'), findsOneWidget);
    expect(find.text('HLG'), findsOneWidget);
    expect(find.text('ACES Filmic'), findsOneWidget);
    expect(find.text('Reinhard Extended'), findsOneWidget);

    await tester.ensureVisible(find.text('HLG'));
    await tester.tap(find.text('HLG'));
    await tester.pumpAndSettle();
    expect(controller.hdrSettings.transferFunction, 'HLG');

    await tester.ensureVisible(find.text('Reinhard Extended'));
    await tester.tap(find.text('Reinhard Extended'));
    await tester.pumpAndSettle();
    expect(controller.hdrSettings.algorithm, 'Reinhard Extended');

    // 5. Select a clip on the timeline
    final clipText = find.text('Golden Hour Intro.mp4');
    expect(clipText, findsOneWidget);
    await tester.tap(clipText);
    await tester.pumpAndSettle();

    // Verify Clip Inspector opened
    expect(find.text('CLIP INSPECTOR'), findsOneWidget);
    expect(find.text('SPATIAL TRANSFORM'), findsOneWidget);
    expect(find.text('PLAYBACK SPEED & TIMING'), findsOneWidget);
    expect(find.text('Preserve Pitch (WSOLA)'), findsOneWidget);

    // Phase 5: Verify 3D Spatial Audio & Binaural Section in Clip Inspector
    await tester.ensureVisible(find.text('3D SPATIAL AUDIO & BINAURAL'));
    expect(find.text('3D SPATIAL AUDIO & BINAURAL'), findsOneWidget);

    controller.toggleClipSpatialAudio(true);
    await tester.pumpAndSettle();
    expect(controller.selectedClip?.spatialAudio.isEnabled, isTrue);
    expect(find.text('Azimuth (Horizontal)'), findsOneWidget);
    expect(find.text('Elevation (Vertical)'), findsOneWidget);
    expect(find.text('Distance'), findsOneWidget);
    expect(find.textContaining('Woodworth ITD'), findsOneWidget);

    controller.updateClipSpatialAudio(
      azimuthDeg: 45.0,
      elevationDeg: 15.0,
      distanceMeters: 2.5,
    );
    await tester.pumpAndSettle();
    expect(controller.selectedClip?.spatialAudio.azimuthDeg, 45.0);
    expect(controller.selectedClip?.spatialAudio.elevationDeg, 15.0);
    expect(controller.selectedClip?.spatialAudio.distanceMeters, 2.5);

    expect(find.text('KEYFRAME EASING CURVE'), findsOneWidget);
    expect(find.text('TRANSITIONS (EXTENDED PRESETS)'), findsOneWidget);

    // Phase 4: Verify Chroma Keying in Clip Inspector
    await tester.ensureVisible(find.text('CHROMA KEYING & SPILL'));
    expect(find.text('CHROMA KEYING & SPILL'), findsOneWidget);
    controller.toggleClipChromaKey(true);
    await tester.pumpAndSettle();
    expect(controller.selectedClip?.chromaKey.isEnabled, isTrue);
    expect(find.text('Green Screen'), findsOneWidget);
    expect(find.text('Blue Screen'), findsOneWidget);
    expect(find.text('Similarity Threshold'), findsOneWidget);
    expect(find.text('Spill Suppression'), findsOneWidget);

    // Phase 4: Verify Motion Tracking in Clip Inspector
    await tester.ensureVisible(find.text('MOTION TRACKING (NCC)'));
    expect(find.text('MOTION TRACKING (NCC)'), findsOneWidget);
    expect(find.text('Track Subject in Clip'), findsOneWidget);

    // Test Keyframe Easing selection
    await tester.ensureVisible(find.text('Ease In-Out'));
    await tester.tap(find.text('Ease In-Out'));
    await tester.pumpAndSettle();
    expect(controller.selectedEasing, 'Ease In-Out');

    // Test Transition selection
    await tester.ensureVisible(find.text('Iris Round'));
    await tester.tap(find.text('Iris Round'));
    await tester.pumpAndSettle();
    expect(
      controller.project.transitions.any((t) => t.kindName == 'Iris Round'),
      isTrue,
    );

    // Verify amber transition badge rendered on timeline
    expect(find.textContaining('Iris Round'), findsAtLeastNWidgets(1));

    // Phase 6: Verify Optical Flow & AI Slow-Motion Section in Clip Inspector
    await tester.ensureVisible(find.text('OPTICAL FLOW & AI SLOW-MO'));
    expect(find.text('OPTICAL FLOW & AI SLOW-MO'), findsOneWidget);

    controller.toggleClipOpticalFlow(true, multiplier: 2.0);
    await tester.pumpAndSettle();
    expect(controller.selectedClip?.opticalFlow.isEnabled, isTrue);
    expect(find.text('Slow-Motion Rate:'), findsOneWidget);
    expect(find.text('2x Smooth'), findsOneWidget);
    expect(find.text('4x Cinema Fluid'), findsOneWidget);
    expect(find.text('8x Super Slo-Mo'), findsOneWidget);

    await tester.ensureVisible(find.text('4x Cinema Fluid'));
    await tester.tap(find.text('4x Cinema Fluid'));
    await tester.pumpAndSettle();
    expect(controller.selectedClip?.opticalFlow.slowMotionMultiplier, 4.0);
    expect(find.text('Occlusion Guard'), findsOneWidget);

    // Phase 6: Verify Smart Rotoscoping & Edge Mask Section in Clip Inspector
    await tester.ensureVisible(find.text('SMART ROTOSCOPING & EDGE MASK'));
    expect(find.text('SMART ROTOSCOPING & EDGE MASK'), findsOneWidget);

    controller.toggleClipSmartMask(true);
    await tester.pumpAndSettle();
    expect(controller.selectedClip?.smartMask.isEnabled, isTrue);
    expect(find.text('Edge Contrast Sensitivity'), findsOneWidget);
    expect(find.text('Color Tolerance'), findsOneWidget);
    expect(find.text('Feather Radius'), findsOneWidget);
    expect(find.text('Invert Subject Mask'), findsOneWidget);

    controller.updateClipSmartMask(
      contrastThreshold: 0.35,
      colorTolerance: 0.40,
      featherRadius: 4.0,
      invert: true,
    );
    await tester.pumpAndSettle();
    expect(controller.selectedClip?.smartMask.contrastThreshold, 0.35);
    expect(controller.selectedClip?.smartMask.colorTolerance, 0.40);
    expect(controller.selectedClip?.smartMask.featherRadius, 4.0);
    expect(controller.selectedClip?.smartMask.invert, isTrue);

    // Close Clip Inspector
    controller.selectClip(null);
    await tester.pumpAndSettle();
    expect(find.text('PROJECT MASTER & DYNAMICS'), findsOneWidget);
  });
}

