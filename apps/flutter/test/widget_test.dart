import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:iris_studio/main.dart';

void main() {
  testWidgets('Iris Studio workspace shell and navigation smoke test', (WidgetTester tester) async {
    // Set workstation viewport resolution
    tester.view.physicalSize = const Size(1280, 800);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.resetPhysicalSize);

    // Build IrisStudioApp and trigger initial frame
    await tester.pumpWidget(const IrisStudioApp());
    await tester.pump();

    // Verify studio branding is present
    expect(find.text('IRIS STUDIO'), findsOneWidget);

    // Verify workspace tabs are rendered
    expect(find.byKey(const ValueKey('tab_home')), findsOneWidget);
    expect(find.byKey(const ValueKey('tab_photo')), findsOneWidget);
    expect(find.byKey(const ValueKey('tab_video')), findsOneWidget);
    expect(find.byKey(const ValueKey('tab_export')), findsOneWidget);

    // Switch to Photo Workspace
    await tester.tap(find.byKey(const ValueKey('tab_photo')));
    await tester.pumpAndSettle();
    expect(find.text('Fit Canvas'), findsOneWidget);

    // Switch to Video Workspace
    await tester.tap(find.byKey(const ValueKey('tab_video')));
    await tester.pumpAndSettle();
    expect(find.text('TRACKS'), findsOneWidget);
    expect(find.text('Split (S)'), findsOneWidget);

    // Switch to Export Workspace
    await tester.tap(find.byKey(const ValueKey('tab_export')));
    await tester.pumpAndSettle();
    expect(find.text('Production Delivery Presets'), findsOneWidget);
  });
}
