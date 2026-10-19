import 'package:flutter/material.dart';
import '../theme/iris_theme.dart';

class AudioMeters extends StatelessWidget {
  final double leftDbfs;
  final double rightDbfs;
  final double peakLeft;
  final double peakRight;
  final bool isVertical;

  const AudioMeters({
    super.key,
    required this.leftDbfs,
    required this.rightDbfs,
    required this.peakLeft,
    required this.peakRight,
    this.isVertical = true,
  });

  @override
  Widget build(BuildContext context) {
    if (isVertical) {
      return Container(
        width: 44,
        padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 6),
        decoration: BoxDecoration(
          color: IrisTheme.surfaceDark,
          borderRadius: BorderRadius.circular(6),
          border: Border.all(color: IrisTheme.border),
        ),
        child: Column(
          children: [
            const Text(
              'dBFS',
              style: TextStyle(
                color: IrisTheme.textMuted,
                fontSize: 9,
                fontWeight: FontWeight.w600,
              ),
            ),
            const SizedBox(height: 6),
            Expanded(
              child: Row(
                mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                children: [
                  _buildVerticalBar('L', peakLeft, leftDbfs),
                  _buildVerticalBar('R', peakRight, rightDbfs),
                ],
              ),
            ),
            const SizedBox(height: 6),
            Text(
              leftDbfs.toStringAsFixed(1),
              style: const TextStyle(
                color: IrisTheme.textSecondary,
                fontSize: 8,
                fontFamily: 'monospace',
              ),
            ),
          ],
        ),
      );
    } else {
      return Row(
        children: [
          Expanded(child: _buildHorizontalBar('L', peakLeft)),
          const SizedBox(width: 8),
          Expanded(child: _buildHorizontalBar('R', peakRight)),
        ],
      );
    }
  }

  Widget _buildVerticalBar(String label, double peak, double dbfs) {
    return Column(
      children: [
        Expanded(
          child: Container(
            width: 8,
            decoration: BoxDecoration(
              color: IrisTheme.surfaceLight,
              borderRadius: BorderRadius.circular(2),
            ),
            alignment: Alignment.bottomCenter,
            child: FractionallySizedBox(
              heightFactor: peak.clamp(0.0, 1.0),
              child: Container(
                decoration: BoxDecoration(
                  gradient: const LinearGradient(
                    begin: Alignment.bottomCenter,
                    end: Alignment.topCenter,
                    colors: [
                      IrisTheme.accentGreen,
                      IrisTheme.accentAmber,
                      IrisTheme.accentRed,
                    ],
                    stops: [0.65, 0.88, 1.0],
                  ),
                  borderRadius: BorderRadius.circular(2),
                ),
              ),
            ),
          ),
        ),
        const SizedBox(height: 4),
        Text(
          label,
          style: const TextStyle(color: IrisTheme.textMuted, fontSize: 8),
        ),
      ],
    );
  }

  Widget _buildHorizontalBar(String label, double peak) {
    return Row(
      children: [
        Text(
          label,
          style: const TextStyle(color: IrisTheme.textMuted, fontSize: 9),
        ),
        const SizedBox(width: 4),
        Expanded(
          child: Container(
            height: 6,
            decoration: BoxDecoration(
              color: IrisTheme.surfaceLight,
              borderRadius: BorderRadius.circular(2),
            ),
            alignment: Alignment.centerLeft,
            child: FractionallySizedBox(
              widthFactor: peak.clamp(0.0, 1.0),
              child: Container(
                decoration: BoxDecoration(
                  gradient: const LinearGradient(
                    colors: [IrisTheme.accentGreen, IrisTheme.accentAmber],
                  ),
                  borderRadius: BorderRadius.circular(2),
                ),
              ),
            ),
          ),
        ),
      ],
    );
  }
}
