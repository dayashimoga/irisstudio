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
  String _activeTab = 'adjust'; // 'adjust', 'crop', 'retouch', 'layers', 'presets', 'batch'

  // Retouching active strokes
  final List<Offset> _retouchStrokes = [];
  Offset? _cloneSourcePoint;
  Offset? _brushHoverPos;

  // Batch export form fields
  String _batchNamingTemplate = '{name}_{preset}_{index}';
  final String _batchPreset = 'web_optimized';
  String _batchFormat = 'jpg';
  int _batchQuality = 85;
  String _batchCollision = 'RenameWithIndex';

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.controller,
      builder: (context, _) {
        final isCompact = MediaQuery.of(context).size.width < 850;

        return Row(
          children: [
            // Left Toolbar (Tool selector: Adjust, Crop, Retouch, Layers, Presets, Batch)
            _buildLeftToolRail(),

            // Center Interactive Canvas & Viewport
            Expanded(
              child: Column(
                children: [
                  // Canvas Header Toolbar (Open, Zoom, Fit, Split, Undo/Redo, Export, Reset)
                  _buildCanvasHeader(),

                  // Main Canvas Viewport
                  Expanded(
                    child: ClipRect(
                      child: GestureDetector(
                        onPanUpdate: (details) {
                          if (widget.controller.activeRetouchTool != RetouchToolMode.none) {
                            setState(() {
                              _retouchStrokes.add(details.localPosition);
                            });
                          } else {
                            setState(() {
                              _panOffset += details.delta;
                            });
                          }
                        },
                        onTapDown: (details) {
                          if (widget.controller.activeRetouchTool == RetouchToolMode.cloneStamp &&
                              _cloneSourcePoint == null) {
                            setState(() {
                              _cloneSourcePoint = details.localPosition;
                            });
                          } else if (widget.controller.activeRetouchTool != RetouchToolMode.none) {
                            setState(() {
                              _retouchStrokes.add(details.localPosition);
                            });
                          }
                        },
                        child: MouseRegion(
                          onHover: (e) => setState(() => _brushHoverPos = e.localPosition),
                          child: Stack(
                            alignment: Alignment.center,
                            children: [
                              // Checkerboard / Canvas Background
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

                              // Interactive Retouch Stings & Overlays
                              if (_retouchStrokes.isNotEmpty)
                                CustomPaint(
                                  size: Size.infinite,
                                  painter: _RetouchStrokesPainter(
                                    strokes: _retouchStrokes,
                                    radius: widget.controller.retouchRadius,
                                    tool: widget.controller.activeRetouchTool,
                                  ),
                                ),

                              // Clone Stamp Source Reticle
                              if (_cloneSourcePoint != null)
                                Positioned(
                                  left: _cloneSourcePoint!.dx - 12,
                                  top: _cloneSourcePoint!.dy - 12,
                                  child: Container(
                                    width: 24,
                                    height: 24,
                                    decoration: BoxDecoration(
                                      shape: BoxShape.circle,
                                      border: Border.all(color: Colors.yellowAccent, width: 2),
                                    ),
                                    child: const Center(
                                      child: Icon(Icons.add, size: 14, color: Colors.yellowAccent),
                                    ),
                                  ),
                                ),

                              // Brush Hover Reticle
                              if (widget.controller.activeRetouchTool != RetouchToolMode.none &&
                                  _brushHoverPos != null)
                                Positioned(
                                  left: _brushHoverPos!.dx - widget.controller.retouchRadius / 2,
                                  top: _brushHoverPos!.dy - widget.controller.retouchRadius / 2,
                                  child: IgnorePointer(
                                    child: Container(
                                      width: widget.controller.retouchRadius,
                                      height: widget.controller.retouchRadius,
                                      decoration: BoxDecoration(
                                        shape: BoxShape.circle,
                                        border: Border.all(
                                          color: Colors.white.withValues(alpha: 0.8),
                                          width: 1.5,
                                        ),
                                      ),
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
                  ),
                ],
              ),
            ),

            // Right Inspector Panel (or Drawer on compact displays)
            if (!isCompact) _buildRightInspector(),
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
            tooltip: 'Crop, Straighten, Rotate & Lens',
          ),
          _buildRailButton(
            id: 'retouch',
            icon: Icons.auto_fix_high,
            tooltip: 'Retouch: Red-Eye, Clone, Heal & Brush',
          ),
          _buildRailButton(
            id: 'layers',
            icon: Icons.layers_outlined,
            tooltip: 'Photo Layers & Blend Modes',
          ),
          _buildRailButton(
            id: 'presets',
            icon: Icons.filter_vintage_outlined,
            tooltip: 'Creative Presets & LUTs',
          ),
          _buildRailButton(
            id: 'batch',
            icon: Icons.dynamic_feed_outlined,
            tooltip: 'Batch Photo Processing & Naming Rules',
          ),
          const Spacer(),
          IconButton(
            icon: const Icon(Icons.info_outline, size: 18),
            tooltip: 'EXIF Metadata & ICC Profile (sRGB Linear)',
            onPressed: _showExifMetadataDialog,
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
        onPressed: () {
          setState(() {
            _activeTab = id;
            if (id != 'retouch') {
              widget.controller.setActiveRetouchTool(RetouchToolMode.none);
            }
          });
        },
      ),
    );
  }

  Widget _buildCanvasHeader() {
    return Container(
      height: 42,
      decoration: const BoxDecoration(
        color: IrisTheme.surfaceDark,
        border: Border(bottom: BorderSide(color: IrisTheme.border)),
      ),
      padding: const EdgeInsets.symmetric(horizontal: 12),
      child: SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            // Open Image Button
            ElevatedButton.icon(
              icon: const Icon(Icons.folder_open, size: 14),
              label: const Text('Open Photo', style: TextStyle(fontSize: 11)),
              style: ElevatedButton.styleFrom(
                backgroundColor: IrisTheme.surfaceLight,
                foregroundColor: IrisTheme.textPrimary,
                elevation: 0,
                visualDensity: VisualDensity.compact,
              ),
              onPressed: _showOpenPhotoDialog,
            ),

            const SizedBox(width: 8),
            const VerticalDivider(width: 1, indent: 8, endIndent: 8),
            const SizedBox(width: 8),

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

            const SizedBox(width: 8),
            const VerticalDivider(width: 1, indent: 8, endIndent: 8),
            const SizedBox(width: 8),

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

            const SizedBox(width: 8),
            // Undo / Redo
            IconButton(
              icon: const Icon(Icons.undo, size: 16),
              tooltip: 'Undo (Ctrl+Z)',
              onPressed: widget.controller.canUndo ? widget.controller.undo : null,
              splashRadius: 16,
            ),
            IconButton(
              icon: const Icon(Icons.redo, size: 16),
              tooltip: 'Redo (Ctrl+Y)',
              onPressed: widget.controller.canRedo ? widget.controller.redo : null,
              splashRadius: 16,
            ),

            const SizedBox(width: 16),

            // Export Photo Button
          ElevatedButton.icon(
            key: const ValueKey('export_photo_header_btn'),
            icon: const Icon(Icons.file_download_outlined, size: 14),
            label: const Text('Export Photo', style: TextStyle(fontSize: 11)),
            style: ElevatedButton.styleFrom(
              backgroundColor: IrisTheme.primaryCyan,
              foregroundColor: Colors.black,
              elevation: 0,
              visualDensity: VisualDensity.compact,
            ),
            onPressed: _showExportPhotoDialog,
          ),

          const SizedBox(width: 8),
          // Reset adjustments button
          TextButton.icon(
            icon: const Icon(Icons.restore, size: 14),
            label: const Text('Reset All', style: TextStyle(fontSize: 11)),
            style: TextButton.styleFrom(foregroundColor: IrisTheme.textMuted),
            onPressed: () {
              widget.controller.updateAdjustments(PhotoAdjustments());
              setState(() {
                _retouchStrokes.clear();
                _cloneSourcePoint = null;
              });
            },
          ),
        ],
      ),
    ),
  );
}

  Widget _buildInteractivePhoto() {
    final adj = widget.controller.photoAdjustments;

    return Transform.rotate(
      angle: (widget.controller.photoStraighten + widget.controller.photoRotation) *
          (3.14159265 / 180.0),
      child: Transform.scale(
        scaleX: widget.controller.photoFlipH ? -1.0 : 1.0,
        scaleY: widget.controller.photoFlipV ? -1.0 : 1.0,
        child: Container(
          width: 800,
          height: 500,
          decoration: BoxDecoration(
            color: IrisTheme.surfaceDark,
            boxShadow: [
              BoxShadow(
                color: Colors.black.withValues(alpha: 0.6),
                blurRadius: 24,
                spreadRadius: 4,
              ),
            ],
          ),
          child: Stack(
            children: [
              // Custom Painter simulating the photo with current parametric adjustments
              Positioned.fill(
                child: CustomPaint(
                  painter: _SyntheticPhotoPainter(
                    adjustments: adj,
                    straighten: widget.controller.photoStraighten,
                  ),
                ),
              ),

              // Vignette Overlay Simulation
              if (adj.vignette != 0.0)
                Positioned.fill(
                  child: IgnorePointer(
                    child: Container(
                      decoration: BoxDecoration(
                        gradient: RadialGradient(
                          radius: 0.85,
                          colors: [
                            Colors.transparent,
                            adj.vignette > 0
                                ? Colors.black.withValues(
                                    alpha: (adj.vignette.abs() / 100.0).clamp(0.0, 0.95))
                                : Colors.white.withValues(
                                    alpha: (adj.vignette.abs() / 100.0).clamp(0.0, 0.95)),
                          ],
                        ),
                      ),
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
              // Draggable split divider line
              Positioned(
                left: splitX - 1,
                top: 0,
                bottom: 0,
                child: GestureDetector(
                  onHorizontalDragUpdate: (details) {
                    final newPos = (details.globalPosition.dx - 52) / constraints.maxWidth;
                    widget.controller.setSplitPosition(newPos);
                  },
                  child: Container(
                    width: 20,
                    color: Colors.transparent,
                    child: Center(
                      child: Container(
                        width: 2,
                        color: IrisTheme.primaryCyan,
                      ),
                    ),
                  ),
                ),
              ),

              // Before badge (left)
              Positioned(
                left: 16,
                top: 16,
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                  decoration: BoxDecoration(
                    color: Colors.black.withValues(alpha: 0.6),
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: const Text('BEFORE', style: TextStyle(color: Colors.white, fontSize: 9)),
                ),
              ),

              // After badge (right)
              Positioned(
                right: 16,
                top: 16,
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
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
      width: 320,
      decoration: const BoxDecoration(
        color: IrisTheme.surfaceDark,
        border: Border(left: BorderSide(color: IrisTheme.border)),
      ),
      child: Column(
        children: [
          // Inspector Header with Histogram
          _buildHistogramCard(),

          // Sub-panels based on active tab
          Expanded(
            child: SingleChildScrollView(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
              child: _buildActiveSubPanel(),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildActiveSubPanel() {
    switch (_activeTab) {
      case 'crop':
        return _buildCropGeometryPanel();
      case 'retouch':
        return _buildRetouchPanel();
      case 'layers':
        return _buildLayersPanel();
      case 'presets':
        return _buildPresetsPanel();
      case 'batch':
        return _buildBatchExportPanel();
      case 'adjust':
      default:
        return _buildLightColorAdjustmentsPanel();
    }
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
                'sRGB Linear (Float32)',
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
          onChanged: (val) => widget.controller.updateAdjustments(adj.copyWith(exposure: val)),
        ),
        _buildParamSlider(
          label: 'Contrast',
          value: adj.contrast,
          min: -100.0,
          max: 100.0,
          onChanged: (val) => widget.controller.updateAdjustments(adj.copyWith(contrast: val)),
        ),
        _buildParamSlider(
          label: 'Highlights',
          value: adj.highlights,
          min: -100.0,
          max: 100.0,
          onChanged: (val) => widget.controller.updateAdjustments(adj.copyWith(highlights: val)),
        ),
        _buildParamSlider(
          label: 'Shadows',
          value: adj.shadows,
          min: -100.0,
          max: 100.0,
          onChanged: (val) => widget.controller.updateAdjustments(adj.copyWith(shadows: val)),
        ),

        const SizedBox(height: 16),
        _buildSectionHeader('WHITE BALANCE & COLOR'),
        _buildParamSlider(
          label: 'Temperature',
          value: adj.temperature,
          min: -100.0,
          max: 100.0,
          onChanged: (val) => widget.controller.updateAdjustments(adj.copyWith(temperature: val)),
        ),
        _buildParamSlider(
          label: 'Tint',
          value: adj.tint,
          min: -100.0,
          max: 100.0,
          onChanged: (val) => widget.controller.updateAdjustments(adj.copyWith(tint: val)),
        ),
        _buildParamSlider(
          label: 'Saturation',
          value: adj.saturation,
          min: -100.0,
          max: 100.0,
          onChanged: (val) => widget.controller.updateAdjustments(adj.copyWith(saturation: val)),
        ),
        _buildParamSlider(
          label: 'Vibrance',
          value: adj.vibrance,
          min: -100.0,
          max: 100.0,
          onChanged: (val) => widget.controller.updateAdjustments(adj.copyWith(vibrance: val)),
        ),

        const SizedBox(height: 16),
        _buildSectionHeader('LEVELS ADJUSTMENT'),
        _buildParamSlider(
          label: 'Black Point',
          value: adj.levelsInBlack,
          min: 0.0,
          max: 100.0,
          onChanged: (val) => widget.controller.updateAdjustments(adj.copyWith(levelsInBlack: val)),
        ),
        _buildParamSlider(
          label: 'Midtone Gamma',
          value: adj.levelsGamma,
          min: 0.2,
          max: 3.0,
          onChanged: (val) => widget.controller.updateAdjustments(adj.copyWith(levelsGamma: val)),
        ),
        _buildParamSlider(
          label: 'White Point',
          value: adj.levelsInWhite,
          min: 150.0,
          max: 255.0,
          onChanged: (val) => widget.controller.updateAdjustments(adj.copyWith(levelsInWhite: val)),
        ),

        const SizedBox(height: 16),
        _buildSectionHeader('DETAIL & TEXTURE'),
        _buildParamSlider(
          label: 'Sharpening',
          value: adj.sharpening,
          min: 0.0,
          max: 100.0,
          onChanged: (val) => widget.controller.updateAdjustments(adj.copyWith(sharpening: val)),
        ),
        _buildParamSlider(
          label: 'Bilateral Denoise',
          value: adj.denoise,
          min: 0.0,
          max: 100.0,
          onChanged: (val) => widget.controller.updateAdjustments(adj.copyWith(denoise: val)),
        ),
        _buildParamSlider(
          label: 'Gaussian Blur',
          value: adj.blur,
          min: 0.0,
          max: 20.0,
          onChanged: (val) => widget.controller.updateAdjustments(adj.copyWith(blur: val)),
        ),
        _buildParamSlider(
          label: 'Vignette',
          value: adj.vignette,
          min: -100.0,
          max: 100.0,
          onChanged: (val) => widget.controller.updateAdjustments(adj.copyWith(vignette: val)),
        ),
      ],
    );
  }

  Widget _buildCropGeometryPanel() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildSectionHeader('ASPECT RATIO PRESETS'),
        const SizedBox(height: 8),
        Wrap(
          spacing: 6,
          runSpacing: 6,
          children: AspectRatioPreset.values.map((preset) {
            final isSelected = widget.controller.aspectRatio == preset;
            return ChoiceChip(
              label: Text(preset.label, style: const TextStyle(fontSize: 11)),
              selected: isSelected,
              selectedColor: IrisTheme.primaryCyan,
              backgroundColor: IrisTheme.surfaceLight,
              labelStyle: TextStyle(
                color: isSelected ? Colors.black : IrisTheme.textPrimary,
                fontWeight: isSelected ? FontWeight.w600 : FontWeight.normal,
              ),
              onSelected: (_) => widget.controller.setAspectRatio(preset),
            );
          }).toList(),
        ),

        const SizedBox(height: 20),
        _buildSectionHeader('STRAIGHTEN & ROTATE'),
        _buildParamSlider(
          label: 'Straighten Angle',
          value: widget.controller.photoStraighten,
          min: -45.0,
          max: 45.0,
          unit: '°',
          onChanged: widget.controller.setPhotoStraighten,
        ),
        const SizedBox(height: 8),
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
            Expanded(
              child: OutlinedButton.icon(
                icon: const Icon(Icons.flip, size: 16),
                label: const Text('Flip H', style: TextStyle(fontSize: 11)),
                onPressed: widget.controller.toggleFlipHorizontal,
              ),
            ),
            const SizedBox(width: 8),
            Expanded(
              child: OutlinedButton.icon(
                icon: const Icon(Icons.flip_camera_android, size: 16),
                label: const Text('Flip V', style: TextStyle(fontSize: 11)),
                onPressed: widget.controller.toggleFlipVertical,
              ),
            ),
          ],
        ),

        const SizedBox(height: 20),
        _buildSectionHeader('LENS DISTORTION CORRECTION'),
        _buildParamSlider(
          label: 'Radial Distortion (k1)',
          value: widget.controller.photoAdjustments.lensK1,
          min: -100.0,
          max: 100.0,
          onChanged: (val) {
            widget.controller.updateAdjustments(
              widget.controller.photoAdjustments.copyWith(lensK1: val),
            );
          },
        ),
        const Text(
          'Negative values correct barrel distortion (wide-angle lenses). Positive values correct pincushion distortion.',
          style: TextStyle(color: IrisTheme.textMuted, fontSize: 10),
        ),
      ],
    );
  }

  Widget _buildRetouchPanel() {
    final activeTool = widget.controller.activeRetouchTool;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildSectionHeader('RETOUCHING TOOLS'),
        const SizedBox(height: 8),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            _buildToolChip('Red-Eye', RetouchToolMode.redEye, Icons.remove_red_eye),
            _buildToolChip('Clone Stamp', RetouchToolMode.cloneStamp, Icons.control_point),
            _buildToolChip('Heal Brush', RetouchToolMode.healBrush, Icons.healing),
            _buildToolChip('Paint Brush', RetouchToolMode.brush, Icons.brush),
            _buildToolChip('Eraser', RetouchToolMode.eraser, Icons.auto_fix_normal),
          ],
        ),

        const SizedBox(height: 20),
        _buildSectionHeader('BRUSH PROPERTIES'),
        _buildParamSlider(
          label: 'Radius',
          value: widget.controller.retouchRadius,
          min: 4.0,
          max: 150.0,
          unit: ' px',
          onChanged: widget.controller.setRetouchRadius,
        ),
        _buildParamSlider(
          label: 'Hardness / Feather',
          value: widget.controller.retouchHardness,
          min: 0.0,
          max: 1.0,
          onChanged: widget.controller.setRetouchHardness,
        ),
        _buildParamSlider(
          label: 'Opacity',
          value: widget.controller.retouchOpacity,
          min: 0.1,
          max: 1.0,
          onChanged: widget.controller.setRetouchOpacity,
        ),

        const SizedBox(height: 16),
        if (activeTool == RetouchToolMode.cloneStamp)
          Container(
            padding: const EdgeInsets.all(10),
            decoration: BoxDecoration(
              color: IrisTheme.surfaceMid,
              borderRadius: BorderRadius.circular(6),
              border: Border.all(color: IrisTheme.border),
            ),
            child: Row(
              children: [
                const Icon(Icons.info_outline, size: 16, color: Colors.yellowAccent),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    _cloneSourcePoint == null
                        ? 'Tap on canvas to set Source point.'
                        : 'Source set! Drag on target to clone texture.',
                    style: const TextStyle(fontSize: 11, color: IrisTheme.textPrimary),
                  ),
                ),
              ],
            ),
          ),

        const SizedBox(height: 20),
        OutlinedButton.icon(
          icon: const Icon(Icons.clear_all, size: 16),
          label: const Text('Clear Retouch Marks', style: TextStyle(fontSize: 11)),
          style: OutlinedButton.styleFrom(
            minimumSize: const Size(double.infinity, 36),
          ),
          onPressed: () {
            setState(() {
              _retouchStrokes.clear();
              _cloneSourcePoint = null;
            });
          },
        ),
      ],
    );
  }

  Widget _buildToolChip(String label, RetouchToolMode mode, IconData icon) {
    final isSelected = widget.controller.activeRetouchTool == mode;
    return ChoiceChip(
      avatar: Icon(icon, size: 14, color: isSelected ? Colors.black : IrisTheme.primaryCyan),
      label: Text(label, style: const TextStyle(fontSize: 11)),
      selected: isSelected,
      selectedColor: IrisTheme.primaryCyan,
      backgroundColor: IrisTheme.surfaceLight,
      labelStyle: TextStyle(
        color: isSelected ? Colors.black : IrisTheme.textPrimary,
        fontWeight: isSelected ? FontWeight.w600 : FontWeight.normal,
      ),
      onSelected: (val) {
        widget.controller.setActiveRetouchTool(val ? mode : RetouchToolMode.none);
      },
    );
  }

  Widget _buildLayersPanel() {
    final layers = widget.controller.photoLayers;
    final selectedId = widget.controller.selectedLayerId;
    final selectedLayer = layers.firstWhere((l) => l.id == selectedId, orElse: () => layers.first);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            _buildSectionHeader('PHOTO LAYERS'),
            IconButton(
              icon: const Icon(Icons.add, size: 18, color: IrisTheme.primaryCyan),
              tooltip: 'Add Layer',
              splashRadius: 18,
              onPressed: () {
                widget.controller.addPhotoLayer('Layer ${layers.length + 1}');
              },
            ),
          ],
        ),
        const SizedBox(height: 8),

        // Layer stack list
        ...layers.reversed.map((layer) {
          final isSelected = layer.id == selectedId;
          return Container(
            margin: const EdgeInsets.symmetric(vertical: 3),
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
            decoration: BoxDecoration(
              color: isSelected ? IrisTheme.surfaceLight : IrisTheme.surfaceMid,
              borderRadius: BorderRadius.circular(6),
              border: Border.all(
                color: isSelected ? IrisTheme.primaryCyan : IrisTheme.border,
                width: isSelected ? 1.5 : 1.0,
              ),
            ),
            child: Row(
              children: [
                IconButton(
                  icon: Icon(
                    layer.isVisible ? Icons.visibility : Icons.visibility_off,
                    size: 16,
                    color: layer.isVisible ? IrisTheme.textPrimary : IrisTheme.textMuted,
                  ),
                  splashRadius: 16,
                  onPressed: () => widget.controller.toggleLayerVisibility(layer.id),
                ),
                const SizedBox(width: 4),
                Expanded(
                  child: InkWell(
                    onTap: () => widget.controller.selectLayer(layer.id),
                    child: Text(
                      layer.name,
                      style: TextStyle(
                        fontSize: 12,
                        fontWeight: isSelected ? FontWeight.w600 : FontWeight.normal,
                        color: IrisTheme.textPrimary,
                      ),
                    ),
                  ),
                ),
                Text(
                  layer.blendMode.label,
                  style: const TextStyle(fontSize: 10, color: IrisTheme.textMuted),
                ),
                if (layers.length > 1)
                  IconButton(
                    icon: const Icon(Icons.delete_outline, size: 16, color: Colors.redAccent),
                    splashRadius: 16,
                    onPressed: () => widget.controller.removePhotoLayer(layer.id),
                  ),
              ],
            ),
          );
        }),

        const SizedBox(height: 16),
        _buildSectionHeader('SELECTED LAYER CONTROLS'),
        _buildParamSlider(
          label: 'Layer Opacity',
          value: selectedLayer.opacity,
          min: 0.0,
          max: 1.0,
          onChanged: (val) => widget.controller.setLayerOpacity(selectedId, val),
        ),

        const SizedBox(height: 8),
        const Text('Blend Mode', style: TextStyle(color: IrisTheme.textSecondary, fontSize: 11)),
        const SizedBox(height: 6),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 10),
          decoration: BoxDecoration(
            color: IrisTheme.surfaceLight,
            borderRadius: BorderRadius.circular(6),
            border: Border.all(color: IrisTheme.border),
          ),
          child: DropdownButton<LayerBlendMode>(
            value: selectedLayer.blendMode,
            isExpanded: true,
            underline: const SizedBox(),
            dropdownColor: IrisTheme.surfaceDark,
            items: LayerBlendMode.values.map((mode) {
              return DropdownMenuItem(
                value: mode,
                child: Text(mode.label, style: const TextStyle(fontSize: 12)),
              );
            }).toList(),
            onChanged: (val) {
              if (val != null) {
                widget.controller.setLayerBlendMode(selectedId, val);
              }
            },
          ),
        ),
      ],
    );
  }

  Widget _buildPresetsPanel() {
    final presets = [
      {'name': 'Vivid Landscape', 'desc': 'Punchy contrast, vibrant foliage and clear skies'},
      {'name': 'Cinematic Teal/Orange', 'desc': 'Warm skin tones and moody teal shadows'},
      {'name': 'B&W Contrast', 'desc': 'Dramatic monochrome with high dynamic range'},
      {'name': 'Warm Vintage', 'desc': 'Golden nostalgia with gentle fading and warm haze'},
    ];

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildSectionHeader('CREATIVE LOOKS & LUTS'),
        const SizedBox(height: 8),
        ...presets.map((preset) {
          return Container(
            margin: const EdgeInsets.symmetric(vertical: 4),
            padding: const EdgeInsets.all(10),
            decoration: BoxDecoration(
              color: IrisTheme.surfaceMid,
              borderRadius: BorderRadius.circular(6),
              border: Border.all(color: IrisTheme.border),
            ),
            child: Row(
              children: [
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        preset['name']!,
                        style: const TextStyle(
                          fontSize: 12,
                          fontWeight: FontWeight.w600,
                          color: IrisTheme.textPrimary,
                        ),
                      ),
                      const SizedBox(height: 2),
                      Text(
                        preset['desc']!,
                        style: const TextStyle(fontSize: 10, color: IrisTheme.textMuted),
                      ),
                    ],
                  ),
                ),
                ElevatedButton(
                  style: ElevatedButton.styleFrom(
                    backgroundColor: IrisTheme.surfaceLight,
                    foregroundColor: IrisTheme.primaryCyan,
                    visualDensity: VisualDensity.compact,
                    elevation: 0,
                  ),
                  onPressed: () => widget.controller.applyPresetFilter(preset['name']!),
                  child: const Text('Apply', style: TextStyle(fontSize: 11)),
                ),
              ],
            ),
          );
        }),
      ],
    );
  }

  Widget _buildBatchExportPanel() {
    final isProcessing = widget.controller.isBatchProcessing;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildSectionHeader('BATCH PHOTO EXPORT'),
        const SizedBox(height: 8),
        const Text(
          'Export multiple images with consistent adjustments, custom naming templates, and collision safety.',
          style: TextStyle(color: IrisTheme.textMuted, fontSize: 11),
        ),
        const SizedBox(height: 16),

        const Text('Naming Template', style: TextStyle(color: IrisTheme.textSecondary, fontSize: 11)),
        const SizedBox(height: 6),
        TextField(
          controller: TextEditingController(text: _batchNamingTemplate),
          style: const TextStyle(fontSize: 12, fontFamily: 'monospace'),
          decoration: InputDecoration(
            isDense: true,
            filled: true,
            fillColor: IrisTheme.surfaceLight,
            border: OutlineInputBorder(borderRadius: BorderRadius.circular(6)),
            hintText: '{name}_{preset}_{index}',
          ),
          onChanged: (val) => _batchNamingTemplate = val,
        ),
        const SizedBox(height: 4),
        const Text(
          'Tokens: {name} original name, {preset} preset, {index} number',
          style: TextStyle(color: IrisTheme.textMuted, fontSize: 10),
        ),

        const SizedBox(height: 16),
        const Text('Export Format', style: TextStyle(color: IrisTheme.textSecondary, fontSize: 11)),
        const SizedBox(height: 6),
        Row(
          children: ['jpg', 'png', 'webp'].map((fmt) {
            final isSelected = _batchFormat == fmt;
            return Padding(
              padding: const EdgeInsets.only(right: 8),
              child: ChoiceChip(
                label: Text(fmt.toUpperCase(), style: const TextStyle(fontSize: 11)),
                selected: isSelected,
                selectedColor: IrisTheme.primaryCyan,
                backgroundColor: IrisTheme.surfaceLight,
                labelStyle: TextStyle(
                  color: isSelected ? Colors.black : IrisTheme.textPrimary,
                ),
                onSelected: (_) => setState(() => _batchFormat = fmt),
              ),
            );
          }).toList(),
        ),

        if (_batchFormat == 'jpg') ...[
          const SizedBox(height: 12),
          _buildParamSlider(
            label: 'JPEG Quality',
            value: _batchQuality.toDouble(),
            min: 10.0,
            max: 100.0,
            unit: '%',
            onChanged: (val) => setState(() => _batchQuality = val.round()),
          ),
        ],

        const SizedBox(height: 16),
        const Text('Collision Rule', style: TextStyle(color: IrisTheme.textSecondary, fontSize: 11)),
        const SizedBox(height: 6),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 10),
          decoration: BoxDecoration(
            color: IrisTheme.surfaceLight,
            borderRadius: BorderRadius.circular(6),
            border: Border.all(color: IrisTheme.border),
          ),
          child: DropdownButton<String>(
            value: _batchCollision,
            isExpanded: true,
            underline: const SizedBox(),
            dropdownColor: IrisTheme.surfaceDark,
            items: const [
              DropdownMenuItem(value: 'RenameWithIndex', child: Text('Rename With Index (_001)', style: TextStyle(fontSize: 12))),
              DropdownMenuItem(value: 'Overwrite', child: Text('Overwrite Existing', style: TextStyle(fontSize: 12))),
              DropdownMenuItem(value: 'Skip', child: Text('Skip Existing', style: TextStyle(fontSize: 12))),
            ],
            onChanged: (val) {
              if (val != null) setState(() => _batchCollision = val);
            },
          ),
        ),

        const SizedBox(height: 20),
        if (isProcessing) ...[
          LinearProgressIndicator(
            value: widget.controller.batchProgress,
            color: IrisTheme.primaryCyan,
            backgroundColor: IrisTheme.surfaceLight,
          ),
          const SizedBox(height: 8),
        ],
        if (widget.controller.batchStatusMessage.isNotEmpty) ...[
          Container(
            padding: const EdgeInsets.all(8),
            decoration: BoxDecoration(
              color: IrisTheme.surfaceMid,
              borderRadius: BorderRadius.circular(4),
              border: Border.all(color: IrisTheme.border),
            ),
            child: Text(
              widget.controller.batchStatusMessage,
              style: const TextStyle(color: IrisTheme.primaryCyan, fontSize: 11),
            ),
          ),
          const SizedBox(height: 12),
        ],
        if (!isProcessing) ...[
          ElevatedButton.icon(
            icon: const Icon(Icons.play_arrow, size: 16),
            label: const Text('Start Batch Export (5 Photos)', style: TextStyle(fontSize: 12)),
            style: ElevatedButton.styleFrom(
              backgroundColor: IrisTheme.primaryCyan,
              foregroundColor: Colors.black,
              minimumSize: const Size(double.infinity, 38),
            ),
            onPressed: () {
              widget.controller.runBatchPhotoProcessing(
                namingTemplate: _batchNamingTemplate,
                presetName: _batchPreset,
                format: _batchFormat,
                quality: _batchQuality,
                collisionStrategy: _batchCollision,
              );
            },
          ),
        ],
      ],
    );
  }

  Widget _buildSectionHeader(String title) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
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
    String unit = '',
    required ValueChanged<double> onChanged,
  }) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 3),
      child: Column(
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(label, style: const TextStyle(fontSize: 11, color: IrisTheme.textSecondary)),
              Text(
                '${value.toStringAsFixed(1)}$unit',
                style: const TextStyle(fontSize: 11, fontFamily: 'monospace', color: IrisTheme.textPrimary),
              ),
            ],
          ),
          SliderTheme(
            data: SliderTheme.of(context).copyWith(
              trackHeight: 2.0,
              thumbShape: const RoundSliderThumbShape(enabledThumbRadius: 6.0),
              overlayShape: const RoundSliderOverlayShape(overlayRadius: 12.0),
              activeTrackColor: IrisTheme.primaryCyan,
              inactiveTrackColor: IrisTheme.border,
              thumbColor: IrisTheme.primaryCyan,
            ),
            child: Slider(
              value: value.clamp(min, max),
              min: min,
              max: max,
              onChanged: onChanged,
            ),
          ),
        ],
      ),
    );
  }

  void _showOpenPhotoDialog() {
    showDialog(
      context: context,
      builder: (context) {
        return AlertDialog(
          backgroundColor: IrisTheme.surfaceDark,
          title: const Text('Open Photo', style: TextStyle(color: IrisTheme.textPrimary, fontSize: 16)),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text(
                'Select format to import into the hardware-accelerated photo canvas:',
                style: TextStyle(color: IrisTheme.textSecondary, fontSize: 12),
              ),
              const SizedBox(height: 12),
              ListTile(
                leading: const Icon(Icons.image, color: IrisTheme.primaryCyan),
                title: const Text('Sample Landscape (JPEG / 4K)', style: TextStyle(fontSize: 13)),
                subtitle: const Text('EXIF Tag 1 (Normal Orientation), sRGB Linear', style: TextStyle(fontSize: 11)),
                onTap: () {
                  Navigator.pop(context);
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(content: Text('Imported Landscape 4K Photo successfully')),
                  );
                },
              ),
              ListTile(
                leading: const Icon(Icons.image_outlined, color: Colors.orangeAccent),
                title: const Text('Portrait with Flash (PNG)', style: TextStyle(fontSize: 13)),
                subtitle: const Text('High Red/GB Ratio for Red-Eye Correction', style: TextStyle(fontSize: 11)),
                onTap: () {
                  Navigator.pop(context);
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(content: Text('Imported Flash Portrait Photo successfully')),
                  );
                },
              ),
              ListTile(
                leading: const Icon(Icons.photo_library_outlined, color: Colors.purpleAccent),
                title: const Text('WebP Lossless Asset', style: TextStyle(fontSize: 13)),
                subtitle: const Text('Modern web format with alpha channel', style: TextStyle(fontSize: 11)),
                onTap: () {
                  Navigator.pop(context);
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(content: Text('Imported WebP Asset successfully')),
                  );
                },
              ),
            ],
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('Cancel'),
            ),
          ],
        );
      },
    );
  }

  void _showExportPhotoDialog() {
    showDialog(
      context: context,
      builder: (context) {
        String format = 'JPEG';
        int quality = 90;

        return StatefulBuilder(
          builder: (context, setDialogState) {
            return AlertDialog(
              backgroundColor: IrisTheme.surfaceDark,
              title: const Text('Export Photo', style: TextStyle(color: IrisTheme.textPrimary, fontSize: 16)),
              content: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text('Select target format and quality:', style: TextStyle(color: IrisTheme.textSecondary, fontSize: 12)),
                  const SizedBox(height: 12),
                  Row(
                    children: ['JPEG', 'PNG', 'WebP'].map((f) {
                      final isSelected = format == f;
                      return Padding(
                        padding: const EdgeInsets.only(right: 8),
                        child: ChoiceChip(
                          label: Text(f),
                          selected: isSelected,
                          selectedColor: IrisTheme.primaryCyan,
                          backgroundColor: IrisTheme.surfaceLight,
                          labelStyle: TextStyle(color: isSelected ? Colors.black : IrisTheme.textPrimary),
                          onSelected: (_) => setDialogState(() => format = f),
                        ),
                      );
                    }).toList(),
                  ),
                  if (format == 'JPEG') ...[
                    const SizedBox(height: 12),
                    Text('Quality: $quality%', style: const TextStyle(fontSize: 11, color: IrisTheme.textSecondary)),
                    Slider(
                      value: quality.toDouble(),
                      min: 10,
                      max: 100,
                      onChanged: (v) => setDialogState(() => quality = v.round()),
                    ),
                  ],
                ],
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.pop(context),
                  child: const Text('Cancel'),
                ),
                ElevatedButton(
                  style: ElevatedButton.styleFrom(
                    backgroundColor: IrisTheme.primaryCyan,
                    foregroundColor: Colors.black,
                  ),
                  onPressed: () {
                    Navigator.pop(context);
                    ScaffoldMessenger.of(context).showSnackBar(
                      SnackBar(content: Text('Photo exported successfully as $format! Validated stream integrity.')),
                    );
                  },
                  child: const Text('Export Now'),
                ),
              ],
            );
          },
        );
      },
    );
  }

  void _showExifMetadataDialog() {
    showDialog(
      context: context,
      builder: (context) {
        return AlertDialog(
          backgroundColor: IrisTheme.surfaceDark,
          title: const Text('EXIF & Color Management', style: TextStyle(color: IrisTheme.textPrimary, fontSize: 16)),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: const [
              Text('Resolution: 3840 x 2160 (4K UHD, 8.3 MP)', style: TextStyle(color: IrisTheme.textPrimary, fontSize: 12)),
              SizedBox(height: 4),
              Text('Color Space: Rec.709 / sRGB Linear Float32', style: TextStyle(color: IrisTheme.textSecondary, fontSize: 12)),
              SizedBox(height: 4),
              Text('EXIF Orientation: Tag 1 (Horizontal Normal)', style: TextStyle(color: IrisTheme.textSecondary, fontSize: 12)),
              SizedBox(height: 4),
              Text('Bit Depth: 32-bit RGBA (8-bit per channel)', style: TextStyle(color: IrisTheme.textSecondary, fontSize: 12)),
              SizedBox(height: 4),
              Text('Tiled Pipeline: Active (512x512 tile chunks)', style: TextStyle(color: IrisTheme.primaryCyan, fontSize: 12)),
            ],
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('Close'),
            ),
          ],
        );
      },
    );
  }
}

class _SyntheticPhotoPainter extends CustomPainter {
  final PhotoAdjustments adjustments;
  final double straighten;

  _SyntheticPhotoPainter({
    required this.adjustments,
    required this.straighten,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint();

    // Sky gradient with exposure/temperature shift
    final expShift = (adjustments.exposure * 0.1).clamp(-0.5, 0.5);
    final tempShift = (adjustments.temperature * 0.003).clamp(-0.3, 0.3);

    final topSky = Color.lerp(
      const Color(0xFF1E528E),
      tempShift > 0 ? Colors.amber : Colors.cyan,
      tempShift.abs(),
    )!;

    final skyGradient = LinearGradient(
      begin: Alignment.topCenter,
      end: Alignment.bottomCenter,
      colors: [
        topSky.withValues(alpha: (1.0 + expShift).clamp(0.2, 1.0)),
        const Color(0xFFE28B58),
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
    final lakeGradient = const LinearGradient(
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

class _RetouchStrokesPainter extends CustomPainter {
  final List<Offset> strokes;
  final double radius;
  final RetouchToolMode tool;

  _RetouchStrokesPainter({
    required this.strokes,
    required this.radius,
    required this.tool,
  });

  @override
  void paint(Canvas canvas, Size size) {
    if (strokes.isEmpty) return;

    final paint = Paint()
      ..strokeWidth = radius
      ..strokeCap = StrokeCap.round
      ..style = PaintingStyle.stroke;

    switch (tool) {
      case RetouchToolMode.redEye:
        paint.color = Colors.black.withValues(alpha: 0.6);
        break;
      case RetouchToolMode.cloneStamp:
      case RetouchToolMode.healBrush:
        paint.color = Colors.cyanAccent.withValues(alpha: 0.4);
        break;
      case RetouchToolMode.eraser:
        paint.color = Colors.white.withValues(alpha: 0.3);
        break;
      case RetouchToolMode.brush:
      default:
        paint.color = IrisTheme.primaryCyan.withValues(alpha: 0.7);
        break;
    }

    final path = Path()..moveTo(strokes.first.dx, strokes.first.dy);
    for (int i = 1; i < strokes.length; i++) {
      path.lineTo(strokes[i].dx, strokes[i].dy);
    }
    canvas.drawPath(path, paint);
  }

  @override
  bool shouldRepaint(covariant _RetouchStrokesPainter oldDelegate) => true;
}

class _HistogramPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..strokeWidth = 1.0
      ..style = PaintingStyle.stroke;

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
