import 'package:flutter/material.dart';
import '../bridge/iris_engine_controller.dart';
import '../models/photo_state.dart';
import '../theme/iris_theme.dart';

class PhotoWorkspace extends StatefulWidget {
  final IrisEngineController controller;

  const PhotoWorkspace({super.key, required this.controller});

  @override
  State<PhotoWorkspace> createState() => _PhotoWorkspaceState();
}

class _PhotoWorkspaceState extends State<PhotoWorkspace> {
  double _zoomLevel = 1.0;
  Offset _panOffset = Offset.zero;
  String _activeTab = 'adjust'; // 'adjust', 'crop', 'presets'

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.controller,
      builder: (context, _) {
        return Row(
          children: [
            // Left Toolbar (Tool selector: Adjust, Crop, Presets)
            _buildLeftToolRail(),

            // Center Interactive Canvas & Viewport
            Expanded(
              child: Column(
                children: [
                  // Canvas Header Toolbar (Zoom, Fit, Before/After toggle, Reset)
                  _buildCanvasHeader(),

                  // Main Canvas Viewport
                  Expanded(
                    child: ClipRect(
                      child: GestureDetector(
                        onPanUpdate: (details) {
                          setState(() {
                            _panOffset += details.delta;
                          });
                        },
                        child: Stack(
                          alignment: Alignment.center,
                          children: [
                            // Checkerboard / Background
                            Container(color: IrisTheme.background),

                            // Rendered Image Canvas with Transforms
                            FittedBox(
                              fit: BoxFit.contain,
                              child: Transform.translate(
                                offset: _panOffset,
                                child: Transform.scale(
                                  scale: _zoomLevel,
                                  child: _buildInteractivePhoto(),
                                ),
                              ),
                            ),

                            // Split Line Overlay (when Before/After enabled)
                            if (widget.controller.isBeforeAfterSplit)
                              _buildSplitSliderOverlay(),
                          ],
                        ),
                      ),
                    ),
                  ),
                ],
              ),
            ),

            // Right Inspector Panel (Adjustments, Crop controls, Histogram)
            _buildRightInspector(),
          ],
        );
      },
    );
  }

  Widget _buildLeftToolRail() {
    return Container(
      width: 52,
      decoration: const BoxDecoration(
        color: IrisTheme.surfaceDark,
        border: Border(right: BorderSide(color: IrisTheme.border)),
      ),
      child: Column(
        children: [
          const SizedBox(height: 12),
          _buildRailButton(
            id: 'adjust',
            icon: Icons.tune,
            tooltip: 'Light & Color Adjustments',
          ),
          _buildRailButton(
            id: 'crop',
            icon: Icons.crop_rotate,
            tooltip: 'Crop, Straighten & Rotate',
          ),
          _buildRailButton(
            id: 'presets',
            icon: Icons.filter_vintage_outlined,
            tooltip: 'Creative Presets & LUTs',
          ),
          const Spacer(),
          IconButton(
            icon: const Icon(Icons.info_outline, size: 18),
            tooltip: 'EXIF Metadata & ICC Profile (sRGB Linear)',
            onPressed: () {},
          ),
          const SizedBox(height: 12),
        ],
      ),
    );
  }

  Widget _buildRailButton({
    required String id,
    required IconData icon,
    required String tooltip,
  }) {
    final isSelected = _activeTab == id;
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: IconButton(
        icon: Icon(
          icon,
          size: 20,
          color: isSelected ? IrisTheme.primaryCyan : IrisTheme.textSecondary,
        ),
        tooltip: tooltip,
        splashRadius: 20,
        style: IconButton.styleFrom(
          backgroundColor: isSelected ? IrisTheme.surfaceLight : Colors.transparent,
        ),
        onPressed: () => setState(() => _activeTab = id),
      ),
    );
  }

  Widget _buildCanvasHeader() {
    return Container(
      height: 38,
      decoration: const BoxDecoration(
        color: IrisTheme.surfaceDark,
        border: Border(bottom: BorderSide(color: IrisTheme.border)),
      ),
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Row(
        children: [
          // Zoom level controls
          IconButton(
            icon: const Icon(Icons.remove, size: 14),
            tooltip: 'Zoom Out',
            onPressed: () {
              setState(() => _zoomLevel = (_zoomLevel - 0.25).clamp(0.2, 5.0));
            },
            splashRadius: 14,
          ),
          Text(
            '${(_zoomLevel * 100).round()}%',
            style: const TextStyle(
              fontSize: 11,
              fontFamily: 'monospace',
              color: IrisTheme.textSecondary,
            ),
          ),
          IconButton(
            icon: const Icon(Icons.add, size: 14),
            tooltip: 'Zoom In',
            onPressed: () {
              setState(() => _zoomLevel = (_zoomLevel + 0.25).clamp(0.2, 5.0));
            },
            splashRadius: 14,
          ),
          TextButton(
            onPressed: () {
              setState(() {
                _zoomLevel = 1.0;
                _panOffset = Offset.zero;
              });
            },
            child: const Text('Fit Canvas', style: TextStyle(fontSize: 11)),
          ),

          const SizedBox(width: 12),
          const VerticalDivider(width: 1, indent: 8, endIndent: 8),
          const SizedBox(width: 12),

          // Before / After Split View Button
          ElevatedButton.icon(
            icon: Icon(
              widget.controller.isBeforeAfterSplit ? Icons.compare : Icons.compare_arrows,
              size: 14,
            ),
            label: Text(
              widget.controller.isBeforeAfterSplit ? 'Split Active' : 'Before / After',
              style: const TextStyle(fontSize: 11),
            ),
            style: ElevatedButton.styleFrom(
              backgroundColor: widget.controller.isBeforeAfterSplit
                  ? IrisTheme.primaryCyan
                  : IrisTheme.surfaceLight,
              foregroundColor:
                  widget.controller.isBeforeAfterSplit ? Colors.black : IrisTheme.textPrimary,
              elevation: 0,
              visualDensity: VisualDensity.compact,
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 0),
            ),
            onPressed: widget.controller.toggleBeforeAfterSplit,
          ),

          const Spacer(),

          // Reset adjustments button
          TextButton.icon(
            icon: const Icon(Icons.restore, size: 14),
            label: const Text('Reset Edits', style: TextStyle(fontSize: 11)),
            style: TextButton.styleFrom(foregroundColor: IrisTheme.textMuted),
            onPressed: () => widget.controller.updateAdjustments(PhotoAdjustments()),
          ),
        ],
      ),
    );
  }

  Widget _buildInteractivePhoto() {
    final adj = widget.controller.photoAdjustments;

    // Simulate high-fidelity parametric canvas rendering
    return Transform.rotate(
      angle: (widget.controller.photoStraighten + widget.controller.photoRotation) *
          (3.14159265 / 180.0),
      child: Transform.scale(
        scaleX: widget.controller.photoFlipH ? -1.0 : 1.0,
        scaleY: widget.controller.photoFlipV ? -1.0 : 1.0,
        child: Container(
          width: 580,
          height: 380,
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(4),
            boxShadow: [
              BoxShadow(
                color: Colors.black.withValues(alpha: 0.5),
                blurRadius: 18,
                spreadRadius: 2,
              ),
            ],
          ),
          child: Stack(
            fit: StackFit.expand,
            children: [
              // Synthetic landscape photo preview
              ClipRRect(
                borderRadius: BorderRadius.circular(4),
                child: CustomPaint(
                  painter: _SyntheticPhotoPainter(
                    adjustments: adj,
                    splitPosition: widget.controller.isBeforeAfterSplit
                        ? widget.controller.splitPosition
                        : 1.0,
                  ),
                ),
              ),

              // Vignette overlay
              if (adj.vignette != 0.0)
                Container(
                  decoration: BoxDecoration(
                    borderRadius: BorderRadius.circular(4),
                    gradient: RadialGradient(
                      radius: 0.9,
                      colors: [
                        Colors.transparent,
                        Colors.black.withValues(alpha: (adj.vignette / 100.0).clamp(0.0, 0.85)),
                      ],
                    ),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildSplitSliderOverlay() {
    return Positioned.fill(
      child: LayoutBuilder(
        builder: (context, constraints) {
          final splitX = constraints.maxWidth * widget.controller.splitPosition;
          return Stack(
            children: [
              // Divider vertical line
              Positioned(
                left: splitX - 1,
                top: 0,
                bottom: 0,
                child: Container(width: 2, color: IrisTheme.primaryCyan),
              ),
              // Draggable handle
              Positioned(
                left: splitX - 14,
                top: constraints.maxHeight / 2 - 14,
                child: GestureDetector(
                  onHorizontalDragUpdate: (details) {
                    final newPos = (details.globalPosition.dx - 52) / constraints.maxWidth;
                    widget.controller.setSplitPosition(newPos);
                  },
                  child: Container(
                    width: 28,
                    height: 28,
                    decoration: BoxDecoration(
                      color: IrisTheme.primaryCyan,
                      shape: BoxShape.circle,
                      boxShadow: [
                        BoxShadow(
                          color: Colors.black.withValues(alpha: 0.4),
                          blurRadius: 6,
                        ),
                      ],
                    ),
                    child: const Icon(Icons.code, size: 14, color: Colors.black),
                  ),
                ),
              ),
              // Labels
              Positioned(
                left: splitX - 60,
                top: 20,
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                  decoration: BoxDecoration(
                    color: Colors.black.withValues(alpha: 0.6),
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: const Text('BEFORE', style: TextStyle(color: Colors.white, fontSize: 9)),
                ),
              ),
              Positioned(
                left: splitX + 16,
                top: 20,
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                  decoration: BoxDecoration(
                    color: Colors.black.withValues(alpha: 0.6),
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: const Text('AFTER', style: TextStyle(color: IrisTheme.primaryCyan, fontSize: 9)),
                ),
              ),
            ],
          );
        },
      ),
    );
  }

  Widget _buildRightInspector() {
    return Container(
      width: 310,
      decoration: const BoxDecoration(
        color: IrisTheme.surfaceDark,
        border: Border(left: BorderSide(color: IrisTheme.border)),
      ),
      child: Column(
        children: [
          // Inspector Header with Histogram
          _buildHistogramCard(),

          // Sub-panels
          Expanded(
            child: SingleChildScrollView(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
              child: _activeTab == 'crop'
                  ? _buildCropGeometryPanel()
                  : _activeTab == 'presets'
                      ? _buildPresetsPanel()
                      : _buildLightColorAdjustmentsPanel(),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildHistogramCard() {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: const BoxDecoration(
        color: IrisTheme.surfaceMid,
        border: Border(bottom: BorderSide(color: IrisTheme.border)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: const [
              Text(
                'HISTOGRAM',
                style: TextStyle(
                  color: IrisTheme.textMuted,
                  fontSize: 10,
                  fontWeight: FontWeight.w700,
                  letterSpacing: 0.8,
                ),
              ),
              Text(
                'sRGB Linear',
                style: TextStyle(color: IrisTheme.textMuted, fontSize: 9),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Container(
            height: 60,
            decoration: BoxDecoration(
              color: IrisTheme.surfaceDark,
              borderRadius: BorderRadius.circular(4),
              border: Border.all(color: IrisTheme.border),
            ),
            child: CustomPaint(
              painter: _HistogramPainter(),
              size: const Size(double.infinity, 60),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildLightColorAdjustmentsPanel() {
    final adj = widget.controller.photoAdjustments;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildSectionHeader('EXPOSURE & CONTRAST'),
        _buildParamSlider(
          label: 'Exposure',
          value: adj.exposure,
          min: -5.0,
          max: 5.0,
          unit: ' EV',
          onChanged: (val) {
            widget.controller.updateAdjustments(adj.copyWith(exposure: val));
          },
        ),
        _buildParamSlider(
          label: 'Contrast',
          value: adj.contrast,
          min: -100.0,
          max: 100.0,
          unit: '',
          onChanged: (val) {
            widget.controller.updateAdjustments(adj.copyWith(contrast: val));
          },
        ),
        _buildParamSlider(
          label: 'Highlights',
          value: adj.highlights,
          min: -100.0,
          max: 100.0,
          unit: '',
          onChanged: (val) {
            widget.controller.updateAdjustments(adj.copyWith(highlights: val));
          },
        ),
        _buildParamSlider(
          label: 'Shadows',
          value: adj.shadows,
          min: -100.0,
          max: 100.0,
          unit: '',
          onChanged: (val) {
            widget.controller.updateAdjustments(adj.copyWith(shadows: val));
          },
        ),

        const SizedBox(height: 18),
        _buildSectionHeader('COLOR & WHITE BALANCE'),
        _buildParamSlider(
          label: 'Temperature',
          value: adj.temperature,
          min: -100.0,
          max: 100.0,
          unit: '',
          onChanged: (val) {
            widget.controller.updateAdjustments(adj.copyWith(temperature: val));
          },
        ),
        _buildParamSlider(
          label: 'Tint',
          value: adj.tint,
          min: -100.0,
          max: 100.0,
          unit: '',
          onChanged: (val) {
            widget.controller.updateAdjustments(adj.copyWith(tint: val));
          },
        ),
        _buildParamSlider(
          label: 'Vibrance',
          value: adj.vibrance,
          min: -100.0,
          max: 100.0,
          unit: '',
          onChanged: (val) {
            widget.controller.updateAdjustments(adj.copyWith(vibrance: val));
          },
        ),
        _buildParamSlider(
          label: 'Saturation',
          value: adj.saturation,
          min: -100.0,
          max: 100.0,
          unit: '',
          onChanged: (val) {
            widget.controller.updateAdjustments(adj.copyWith(saturation: val));
          },
        ),

        const SizedBox(height: 18),
        _buildSectionHeader('EFFECTS & DETAIL'),
        _buildParamSlider(
          label: 'Sharpening',
          value: adj.sharpening,
          min: 0.0,
          max: 100.0,
          unit: '',
          onChanged: (val) {
            widget.controller.updateAdjustments(adj.copyWith(sharpening: val));
          },
        ),
        _buildParamSlider(
          label: 'Gaussian Blur',
          value: adj.blur,
          min: 0.0,
          max: 20.0,
          unit: ' px',
          onChanged: (val) {
            widget.controller.updateAdjustments(adj.copyWith(blur: val));
          },
        ),
        _buildParamSlider(
          label: 'Vignette',
          value: adj.vignette,
          min: -100.0,
          max: 100.0,
          unit: '',
          onChanged: (val) {
            widget.controller.updateAdjustments(adj.copyWith(vignette: val));
          },
        ),
      ],
    );
  }

  Widget _buildCropGeometryPanel() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildSectionHeader('ASPECT RATIO PRESETS'),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: AspectRatioPreset.values.map((preset) {
            final isSelected = widget.controller.aspectRatio == preset;
            return ChoiceChip(
              label: Text(preset.label, style: const TextStyle(fontSize: 11)),
              selected: isSelected,
              selectedColor: IrisTheme.primaryCyan,
              backgroundColor: IrisTheme.surfaceMid,
              labelStyle: TextStyle(
                color: isSelected ? Colors.black : IrisTheme.textPrimary,
                fontWeight: isSelected ? FontWeight.w600 : FontWeight.w400,
              ),
              onSelected: (_) => widget.controller.setAspectRatio(preset),
            );
          }).toList(),
        ),

        const SizedBox(height: 24),
        _buildSectionHeader('ORIENTATION & FLIP'),
        Row(
          children: [
            Expanded(
              child: OutlinedButton.icon(
                icon: const Icon(Icons.rotate_right, size: 16),
                label: const Text('Rotate 90°', style: TextStyle(fontSize: 11)),
                onPressed: widget.controller.rotatePhoto,
              ),
            ),
            const SizedBox(width: 8),
            IconButton(
              icon: const Icon(Icons.flip, size: 18),
              tooltip: 'Flip Horizontal',
              onPressed: widget.controller.toggleFlipHorizontal,
            ),
            IconButton(
              icon: const Icon(Icons.flip_camera_android, size: 18),
              tooltip: 'Flip Vertical',
              onPressed: widget.controller.toggleFlipVertical,
            ),
          ],
        ),

        const SizedBox(height: 24),
        _buildSectionHeader('STRAIGHTEN & PERSPECTIVE'),
        _buildParamSlider(
          label: 'Angle',
          value: widget.controller.photoStraighten,
          min: -45.0,
          max: 45.0,
          unit: '°',
          onChanged: widget.controller.setPhotoStraighten,
        ),
      ],
    );
  }

  Widget _buildPresetsPanel() {
    final presets = [
      {'name': 'Default Clean', 'desc': 'Neutral linear baseline'},
      {'name': 'Vivid Landscape', 'desc': 'Punchy skies & enhanced foliage'},
      {'name': 'Cinematic Teal/Orange', 'desc': 'Moody cinema grade'},
      {'name': 'B&W Contrast', 'desc': 'Monochrome dramatic tone'},
      {'name': 'Warm Vintage', 'desc': 'Golden nostalgic tone'},
    ];

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildSectionHeader('CREATIVE STYLES'),
        ListView.separated(
          shrinkWrap: true,
          physics: const NeverScrollableScrollPhysics(),
          itemCount: presets.length,
          separatorBuilder: (_, _) => const SizedBox(height: 8),
          itemBuilder: (ctx, idx) {
            final p = presets[idx];
            return InkWell(
              onTap: () => widget.controller.applyPresetFilter(p['name']!),
              borderRadius: BorderRadius.circular(6),
              child: Container(
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: IrisTheme.surfaceMid,
                  borderRadius: BorderRadius.circular(6),
                  border: Border.all(color: IrisTheme.border),
                ),
                child: Row(
                  children: [
                    const Icon(Icons.auto_awesome, color: IrisTheme.primaryCyan, size: 18),
                    const SizedBox(width: 12),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            p['name']!,
                            style: const TextStyle(
                              color: IrisTheme.textPrimary,
                              fontSize: 12,
                              fontWeight: FontWeight.w600,
                            ),
                          ),
                          Text(
                            p['desc']!,
                            style: const TextStyle(color: IrisTheme.textMuted, fontSize: 10),
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
            );
          },
        ),
      ],
    );
  }

  Widget _buildSectionHeader(String title) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Text(
        title,
        style: const TextStyle(
          color: IrisTheme.textMuted,
          fontSize: 10,
          fontWeight: FontWeight.w700,
          letterSpacing: 0.8,
        ),
      ),
    );
  }

  Widget _buildParamSlider({
    required String label,
    required double value,
    required double min,
    required double max,
    required String unit,
    required ValueChanged<double> onChanged,
  }) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Column(
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(label, style: const TextStyle(color: IrisTheme.textSecondary, fontSize: 12)),
              Text(
                '${value >= 0 && unit != ' EV' && unit != '°' ? '+' : ''}${value.toStringAsFixed(1)}$unit',
                style: const TextStyle(
                  color: IrisTheme.primaryCyan,
                  fontSize: 11,
                  fontFamily: 'monospace',
                ),
              ),
            ],
          ),
          Slider(
            value: value,
            min: min,
            max: max,
            onChanged: onChanged,
          ),
        ],
      ),
    );
  }
}

class _SyntheticPhotoPainter extends CustomPainter {
  final PhotoAdjustments adjustments;
  final double splitPosition;

  _SyntheticPhotoPainter({
    required this.adjustments,
    required this.splitPosition,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint();

    // Sky gradient
    final skyGradient = LinearGradient(
      begin: Alignment.topCenter,
      end: Alignment.bottomCenter,
      colors: [
        Color(0xFF1E528E),
        Color(0xFFE28B58),
      ],
    );
    paint.shader = skyGradient.createShader(Rect.fromLTWH(0, 0, size.width, size.height * 0.6));
    canvas.drawRect(Rect.fromLTWH(0, 0, size.width, size.height * 0.6), paint);

    // Mountain silhouettes
    paint.shader = null;
    paint.color = const Color(0xFF1B2332);
    final path = Path()
      ..moveTo(0, size.height * 0.65)
      ..lineTo(size.width * 0.25, size.height * 0.38)
      ..lineTo(size.width * 0.55, size.height * 0.52)
      ..lineTo(size.width * 0.85, size.height * 0.32)
      ..lineTo(size.width, size.height * 0.62)
      ..lineTo(size.width, size.height)
      ..lineTo(0, size.height)
      ..close();
    canvas.drawPath(path, paint);

    // Lake reflections
    final lakeGradient = LinearGradient(
      begin: Alignment.topCenter,
      end: Alignment.bottomCenter,
      colors: [
        Color(0xFF152A4A),
        Color(0xFF0D1B33),
      ],
    );
    paint.shader = lakeGradient.createShader(
        Rect.fromLTWH(0, size.height * 0.65, size.width, size.height * 0.35));
    canvas.drawRect(
        Rect.fromLTWH(0, size.height * 0.65, size.width, size.height * 0.35), paint);
  }

  @override
  bool shouldRepaint(covariant _SyntheticPhotoPainter oldDelegate) => true;
}

class _HistogramPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..strokeWidth = 1.0
      ..style = PaintingStyle.stroke;

    // Draw stylized red, green, blue spline curves
    paint.color = Colors.redAccent.withValues(alpha: 0.7);
    _drawCurve(canvas, size, paint, 0.2, 0.7, 0.4);

    paint.color = Colors.greenAccent.withValues(alpha: 0.7);
    _drawCurve(canvas, size, paint, 0.3, 0.8, 0.35);

    paint.color = Colors.lightBlueAccent.withValues(alpha: 0.8);
    _drawCurve(canvas, size, paint, 0.15, 0.6, 0.65);
  }

  void _drawCurve(Canvas canvas, Size size, Paint paint, double p1, double p2, double p3) {
    final path = Path();
    path.moveTo(0, size.height);
    path.cubicTo(
      size.width * 0.25,
      size.height * (1.0 - p1),
      size.width * 0.5,
      size.height * (1.0 - p2),
      size.width * 0.75,
      size.height * (1.0 - p3),
    );
    path.lineTo(size.width, size.height);
    canvas.drawPath(path, paint);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}
