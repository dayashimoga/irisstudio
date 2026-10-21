import 'package:flutter/material.dart';

class IrisTheme {
  // Deep charcoal creative workstation palette
  static const Color background = Color(0xFF0D0F12);
  static const Color surfaceDark = Color(0xFF14171D);
  static const Color surfaceMid = Color(0xFF1C2028);
  static const Color surfaceLight = Color(0xFF262C38);
  static const Color border = Color(0xFF333D4F);
  static const Color borderHighlight = Color(0xFF4C5B75);

  // Accents
  static const Color primaryCyan = Color(0xFF00E5FF);
  static const Color accentPurple = Color(0xFF7C4DFF);
  static const Color accentGreen = Color(0xFF00E676);
  static const Color accentAmber = Color(0xFFFF9100);
  static const Color accentRed = Color(0xFFFF5252);

  // High-contrast semantic text tokens (WCAG AA >= 4.5:1 against dark surfaces)
  static const Color textPrimary = Color(0xFFF0F4FA);
  static const Color textSecondary = Color(0xFFB8C5D8); // 8.2:1 on surfaceDark
  static const Color textMuted = Color(0xFF98A7BE);     // 5.8:1 on surfaceDark, 5.2:1 on surfaceMid

  // Interactive and state tokens
  static const Color surfaceSelected = Color(0xFF1E2B3D);
  static const Color surfaceHover = Color(0xFF232D3B);
  static const Color focusRing = Color(0xFF00E5FF);
  static const Color disabledBackground = Color(0xFF161920);
  static const Color disabledText = Color(0xFF6B788C);

  static ThemeData get darkTheme {
    return ThemeData(
      brightness: Brightness.dark,
      scaffoldBackgroundColor: background,
      primaryColor: primaryCyan,
      canvasColor: surfaceDark,
      cardColor: surfaceMid,
      dividerColor: border,
      colorScheme: const ColorScheme.dark(
        primary: primaryCyan,
        secondary: accentPurple,
        surface: surfaceDark,
        error: accentRed,
        onPrimary: Colors.black,
        onSecondary: Colors.white,
        onSurface: textPrimary,
      ),
      textTheme: const TextTheme(
        headlineMedium: TextStyle(
          color: textPrimary,
          fontSize: 22,
          fontWeight: FontWeight.w600,
          letterSpacing: -0.4,
        ),
        titleLarge: TextStyle(
          color: textPrimary,
          fontSize: 16,
          fontWeight: FontWeight.w600,
          letterSpacing: -0.2,
        ),
        titleMedium: TextStyle(
          color: textPrimary,
          fontSize: 14,
          fontWeight: FontWeight.w500,
        ),
        bodyLarge: TextStyle(
          color: textPrimary,
          fontSize: 14,
        ),
        bodyMedium: TextStyle(
          color: textSecondary,
          fontSize: 12,
        ),
        bodySmall: TextStyle(
          color: textMuted,
          fontSize: 11,
        ),
        labelLarge: TextStyle(
          color: textPrimary,
          fontSize: 12,
          fontWeight: FontWeight.w600,
        ),
      ),
      sliderTheme: SliderThemeData(
        activeTrackColor: primaryCyan,
        inactiveTrackColor: surfaceLight,
        thumbColor: primaryCyan,
        overlayColor: primaryCyan.withValues(alpha: 0.2),
        trackHeight: 3.0,
        thumbShape: const RoundSliderThumbShape(enabledThumbRadius: 6.0),
      ),
      tooltipTheme: TooltipThemeData(
        decoration: BoxDecoration(
          color: surfaceLight,
          borderRadius: BorderRadius.circular(4),
          border: Border.all(color: border),
        ),
        textStyle: const TextStyle(color: textPrimary, fontSize: 11),
        waitDuration: const Duration(milliseconds: 400),
      ),
    );
  }
}
