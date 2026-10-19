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
    expect(find.textContaining('D3D11VA'), findsOneWidget);

    // Phase 4: Verify Motion Graphics Titles & Lower-Thirds Card
    await tester.ensureVisible(find.text('MOTION GRAPHICS TITLES'));
    expect(find.text('MOTION GRAPHICS TITLES'), findsOneWidget);
    expect(find.text('Clean'), findsOneWidget);
    expect(find.text('Cinematic'), findsOneWidget);
    await tester.tap(find.text('Cinematic'));
    await tester.pumpAndSettle();
    expect(find.text('CINEMATIC'), findsAtLeastNWidgets(1));

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

    // Close Clip Inspector
    controller.selectClip(null);
    await tester.pumpAndSettle();
    expect(find.text('PROJECT MASTER & DYNAMICS'), findsOneWidget);
  });
}
