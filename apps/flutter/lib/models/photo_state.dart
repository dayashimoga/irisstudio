enum RetouchToolMode {
  none,
  redEye,
  cloneStamp,
  healBrush,
  brush,
  eraser,
}

enum LayerBlendMode {
  normal('Normal'),
  multiply('Multiply'),
  screen('Screen'),
  overlay('Overlay'),
  darken('Darken'),
  lighten('Lighten'),
  colorDodge('Color Dodge'),
  colorBurn('Color Burn'),
  softLight('Soft Light'),
  hardLight('Hard Light'),
  difference('Difference'),
  exclusion('Exclusion');

  final String label;
  const LayerBlendMode(this.label);
}

class PhotoLayerModel {
  final String id;
  String name;
  double opacity; // 0.0 to 1.0
  LayerBlendMode blendMode;
  bool isVisible;
  bool isLocked;

  PhotoLayerModel({
    required this.id,
    required this.name,
    this.opacity = 1.0,
    this.blendMode = LayerBlendMode.normal,
    this.isVisible = true,
    this.isLocked = false,
  });
}

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
  double denoise; // 0.0 to 100.0
  double levelsInBlack; // 0.0 to 255.0
  double levelsGamma; // 0.1 to 10.0
  double levelsInWhite; // 0.0 to 255.0
  double lensK1; // -100.0 to +100.0

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
    this.denoise = 0.0,
    this.levelsInBlack = 0.0,
    this.levelsGamma = 1.0,
    this.levelsInWhite = 255.0,
    this.lensK1 = 0.0,
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
    double? denoise,
    double? levelsInBlack,
    double? levelsGamma,
    double? levelsInWhite,
    double? lensK1,
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
      denoise: denoise ?? this.denoise,
      levelsInBlack: levelsInBlack ?? this.levelsInBlack,
      levelsGamma: levelsGamma ?? this.levelsGamma,
      levelsInWhite: levelsInWhite ?? this.levelsInWhite,
      lensK1: lensK1 ?? this.lensK1,
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
    denoise = 0.0;
    levelsInBlack = 0.0;
    levelsGamma = 1.0;
    levelsInWhite = 255.0;
    lensK1 = 0.0;
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
