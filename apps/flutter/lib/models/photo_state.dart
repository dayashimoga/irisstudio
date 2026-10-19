class PhotoAdjustments {
  double exposure; // -5.0 to +5.0 EV
  double contrast; // -100.0 to +100.0
  double temperature; // -100.0 to +100.0
  double tint; // -100.0 to +100.0
  double highlights; // -100.0 to +100.0
  double shadows; // -100.0 to +100.0
  double saturation; // -100.0 to +100.0
  double vibrance; // -100.0 to +100.0
  double sharpening; // 0.0 to 100.0
  double blur; // 0.0 to 20.0
  double vignette; // -100.0 to +100.0

  PhotoAdjustments({
    this.exposure = 0.0,
    this.contrast = 0.0,
    this.temperature = 0.0,
    this.tint = 0.0,
    this.highlights = 0.0,
    this.shadows = 0.0,
    this.saturation = 0.0,
    this.vibrance = 0.0,
    this.sharpening = 0.0,
    this.blur = 0.0,
    this.vignette = 0.0,
  });

  PhotoAdjustments copyWith({
    double? exposure,
    double? contrast,
    double? temperature,
    double? tint,
    double? highlights,
    double? shadows,
    double? saturation,
    double? vibrance,
    double? sharpening,
    double? blur,
    double? vignette,
  }) {
    return PhotoAdjustments(
      exposure: exposure ?? this.exposure,
      contrast: contrast ?? this.contrast,
      temperature: temperature ?? this.temperature,
      tint: tint ?? this.tint,
      highlights: highlights ?? this.highlights,
      shadows: shadows ?? this.shadows,
      saturation: saturation ?? this.saturation,
      vibrance: vibrance ?? this.vibrance,
      sharpening: sharpening ?? this.sharpening,
      blur: blur ?? this.blur,
      vignette: vignette ?? this.vignette,
    );
  }

  void reset() {
    exposure = 0.0;
    contrast = 0.0;
    temperature = 0.0;
    tint = 0.0;
    highlights = 0.0;
    shadows = 0.0;
    saturation = 0.0;
    vibrance = 0.0;
    sharpening = 0.0;
    blur = 0.0;
    vignette = 0.0;
  }

  Map<String, dynamic> toJson() => {
        'exposure': exposure,
        'contrast': contrast,
        'temperature': temperature,
        'tint': tint,
        'highlights': highlights,
        'shadows': shadows,
        'saturation': saturation,
        'vibrance': vibrance,
      };
}

enum AspectRatioPreset {
  free('Free'),
  square('1:1 Square'),
  portrait('4:5 Portrait'),
  landscape('16:9 Landscape'),
  vertical('9:16 Reel/Story'),
  photo('3:2 Classic');

  final String label;
  const AspectRatioPreset(this.label);

  double? get ratio {
    switch (this) {
      case AspectRatioPreset.free:
        return null;
      case AspectRatioPreset.square:
        return 1.0;
      case AspectRatioPreset.portrait:
        return 4.0 / 5.0;
      case AspectRatioPreset.landscape:
        return 16.0 / 9.0;
      case AspectRatioPreset.vertical:
        return 9.0 / 16.0;
      case AspectRatioPreset.photo:
        return 3.0 / 2.0;
    }
  }
}
