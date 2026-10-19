import 'package:flutter/material.dart';

class IrisTheme {
  // Deep charcoal creative workstation palette
  static const Color background = Color(0xFF0D0F12);
  static const Color surfaceDark = Color(0xFF14171D);
  static const Color surfaceMid = Color(0xFF1C2028);
  static const Color surfaceLight = Color(0xFF262C38);
  static const Color border = Color(0xFF2F3747);
  static const Color borderHighlight = Color(0xFF455068);

  // Accents
  static const Color primaryCyan = Color(0xFF00E5FF);
  static const Color accentPurple = Color(0xFF7C4DFF);
  static const Color accentGreen = Color(0xFF00E676);
  static const Color accentAmber = Color(0xFFFF9100);
  static const Color accentRed = Color(0xFFFF5252);

  // Text
  static const Color textPrimary = Color(0xFFF0F3F8);
  static const Color textSecondary = Color(0xFF9AA5B8);
  static const Color textMuted = Color(0xFF647187);

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
