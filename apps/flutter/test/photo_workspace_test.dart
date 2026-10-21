import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:iris_studio/main.dart';

void main() {
  testWidgets('Phase 1 Photo MVP tools, layers, retouching, and batch processing test',
      (WidgetTester tester) async {
    tester.view.physicalSize = const Size(1400, 900);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.resetPhysicalSize);

    await tester.pumpWidget(const IrisStudioApp());
    await tester.pump();

    // 1. Navigate to Photo workspace
    await tester.tap(find.byKey(const ValueKey('tab_photo')));
    await tester.pumpAndSettle();

    expect(find.text('Fit Canvas'), findsOneWidget);
    expect(find.text('Before / After'), findsOneWidget);
    expect(find.text('HISTOGRAM'), findsOneWidget);

    // 2. Toggle Before / After Split View
    await tester.tap(find.text('Before / After'));
    await tester.pumpAndSettle();
    expect(find.text('BEFORE'), findsOneWidget);
    expect(find.text('AFTER'), findsOneWidget);

    // 3. Switch to Crop & Geometry tab
    await tester.tap(find.byTooltip('Crop, Straighten, Rotate & Lens'));
    await tester.pumpAndSettle();
    expect(find.text('ASPECT RATIO PRESETS'), findsOneWidget);
    expect(find.text('1:1 Square'), findsOneWidget);
    expect(find.text('LENS DISTORTION CORRECTION'), findsOneWidget);

    // Test aspect ratio selection
    await tester.tap(find.text('1:1 Square'));
    await tester.pumpAndSettle();

    // 4. Switch to Retouch tab
    await tester.tap(find.byTooltip('Retouch: Red-Eye, Clone, Heal & Brush'));
    await tester.pumpAndSettle();
    expect(find.text('RETOUCHING TOOLS'), findsOneWidget);
    expect(find.text('Red-Eye'), findsOneWidget);
    expect(find.text('Clone Stamp'), findsOneWidget);
    expect(find.text('Heal Brush'), findsOneWidget);

    // Select Clone Stamp tool
    await tester.tap(find.text('Clone Stamp'));
    await tester.pumpAndSettle();
    expect(find.text('Tap on canvas to set Source point.'), findsOneWidget);

    // 5. Switch to Photo Layers tab
    await tester.tap(find.byTooltip('Photo Layers & Blend Modes'));
    await tester.pumpAndSettle();
    expect(find.text('PHOTO LAYERS'), findsOneWidget);
    expect(find.text('Background (Base)'), findsOneWidget);

    // Add a new layer
    await tester.tap(find.byTooltip('Add Layer'));
    await tester.pumpAndSettle();
    expect(find.text('Layer 2'), findsOneWidget);

    // 6. Switch to Creative Presets & LUTs tab
    await tester.tap(find.byTooltip('Creative Presets & LUTs'));
    await tester.pumpAndSettle();
    expect(find.text('AI & CONTENT-AWARE AUTO ENHANCE'), findsOneWidget);
    expect(find.text('Auto Enhance Photo'), findsOneWidget);
    expect(find.text('Landscape'), findsAtLeastNWidgets(1));

    // 7. Switch to Batch Photo Export tab
    await tester.tap(find.byTooltip('Batch Photo Processing & Naming Rules'));
    await tester.pumpAndSettle();
    expect(find.text('BATCH PHOTO EXPORT'), findsOneWidget);
    expect(find.text('Start Batch Export (5 Photos)'), findsOneWidget);

    // Trigger batch export
    await tester.tap(find.text('Start Batch Export (5 Photos)'));
    await tester.pump();
    expect(find.byType(LinearProgressIndicator), findsOneWidget);

    // Fast forward batch simulation
    for (int i = 0; i < 6; i++) {
      await tester.pump(const Duration(milliseconds: 250));
    }
    await tester.pumpAndSettle();
    expect(find.text('Batch complete: 5 images exported successfully'), findsOneWidget);
  });
}
