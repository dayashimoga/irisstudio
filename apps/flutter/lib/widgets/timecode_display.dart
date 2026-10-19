import 'package:flutter/material.dart';
import '../models/project_model.dart';
import '../theme/iris_theme.dart';

class TimecodeDisplay extends StatelessWidget {
  final double seconds;
  final int fps;

  const TimecodeDisplay({
    super.key,
    required this.seconds,
    this.fps = 30,
  });

  @override
  Widget build(BuildContext context) {
    final rational = RationalTimeModel((seconds * 1000).round(), 1000);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
      decoration: BoxDecoration(
        color: IrisTheme.surfaceMid,
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: IrisTheme.border),
      ),
      child: Text(
        rational.toTimecode(),
        style: const TextStyle(
          color: IrisTheme.primaryCyan,
          fontFamily: 'monospace',
          fontSize: 13,
          fontWeight: FontWeight.w700,
          letterSpacing: 1.0,
        ),
      ),
    );
  }
}
