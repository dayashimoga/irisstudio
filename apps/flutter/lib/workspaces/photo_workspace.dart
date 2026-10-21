import 'dart:io';
import 'package:flutter/material.dart';
import '../bridge/iris_engine_controller.dart';
import '../bridge/iris_ffi_bindings.dart';
import '../models/photo_state.dart';
import '../models/preset_model.dart';
import '../theme/iris_theme.dart';
import '../widgets/media_picker_dialog.dart';

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

  // Presets and Auto-Enhance state
  String _presetSearchQuery = '';
  String _selectedPresetCategory = 'All';
  bool _isAutoEnhancing = false;

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
    if (widget.controller.uiImage == null) {
      return Container(
        width: 800,
        height: 500,
        decoration: BoxDecoration(
          color: IrisTheme.surfaceDark,
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: IrisTheme.border),
        ),
        child: Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(Icons.photo_size_select_actual_outlined, size: 56, color: IrisTheme.textMuted),
              const SizedBox(height: 16),
              const Text(
                'No Photo Open',
                style: TextStyle(
                  color: IrisTheme.textPrimary,
                  fontSize: 16,
                  fontWeight: FontWeight.w600,
                ),
              ),
              const SizedBox(height: 8),
              const Text(
                'Open a photo from disk (JPEG, PNG, WebP) to begin\nhardware-accelerated linear sRGB editing.',
                textAlign: TextAlign.center,
                style: TextStyle(color: IrisTheme.textSecondary, fontSize: 12),
              ),
              const SizedBox(height: 20),
              ElevatedButton.icon(
                icon: const Icon(Icons.folder_open, size: 16),
                label: const Text('Open Photo From Disk'),
                style: ElevatedButton.styleFrom(
                  backgroundColor: IrisTheme.primaryCyan,
                  foregroundColor: Colors.black,
                  padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 12),
                ),
                onPressed: _showOpenPhotoDialog,
              ),
            ],
          ),
        ),
      );
    }

    final width = widget.controller.photoWidth.toDouble();
    final height = widget.controller.photoHeight.toDouble();
    final adj = widget.controller.photoAdjustments;

    return Transform.rotate(
      angle: (widget.controller.photoStraighten + widget.controller.photoRotation) *
          (3.14159265 / 180.0),
      child: Transform.scale(
        scaleX: widget.controller.photoFlipH ? -1.0 : 1.0,
        scaleY: widget.controller.photoFlipV ? -1.0 : 1.0,
        child: Container(
          width: width > 0 ? width : 800,
          height: height > 0 ? height : 500,
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
              // Full processed image
              Positioned.fill(
                child: RawImage(
                  image: widget.controller.uiImage,
                  fit: BoxFit.contain,
                ),
              ),

              // Before / After Split View
              if (widget.controller.isBeforeAfterSplit && widget.controller.uiImageOriginal != null)
                Positioned.fill(
                  child: ClipRect(
                    clipper: _SplitClipper(widget.controller.splitPosition),
                    child: RawImage(
                      image: widget.controller.uiImageOriginal,
                      fit: BoxFit.contain,
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
            child: _activeTab == 'presets'
                ? _buildPresetsPanel()
                : SingleChildScrollView(
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
              painter: _HistogramPainter(histogram: widget.controller.photoHistogram),
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
    final activePreset = widget.controller.activePreset;
    final isModified = widget.controller.isPresetModified;
    final intensity = widget.controller.presetIntensity;

    // Filter presets
    final List<PresetModel> allPresets = widget.controller.photoPresets;
    final categories = ['All', 'Portrait', 'Landscape', 'Vivid', 'Atmosphere', 'Vintage', 'Monochrome', 'My Presets', 'Favorites'];

    final filtered = allPresets.where((p) {
      if (_selectedPresetCategory == 'Favorites') {
        if (!widget.controller.favoritePresetIds.contains(p.id)) return false;
      } else if (_selectedPresetCategory == 'My Presets') {
        if (p.isBuiltin) return false;
      } else if (_selectedPresetCategory != 'All') {
        if (!p.category.toLowerCase().contains(_selectedPresetCategory.toLowerCase())) return false;
      }

      if (_presetSearchQuery.trim().isNotEmpty) {
        final q = _presetSearchQuery.trim().toLowerCase();
        final match = p.name.toLowerCase().contains(q) ||
            p.category.toLowerCase().contains(q) ||
            p.description.toLowerCase().contains(q);
        if (!match) return false;
      }
      return true;
    }).toList();

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        // 1. Auto Enhance Card
        Container(
          margin: const EdgeInsets.all(12),
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            gradient: LinearGradient(
              colors: [
                IrisTheme.primaryCyan.withValues(alpha: 0.15),
                IrisTheme.surfaceMid,
              ],
              begin: Alignment.topLeft,
              end: Alignment.bottomRight,
            ),
            borderRadius: BorderRadius.circular(8),
            border: Border.all(color: IrisTheme.primaryCyan.withValues(alpha: 0.4)),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  const Icon(Icons.auto_fix_high, size: 16, color: IrisTheme.primaryCyan),
                  const SizedBox(width: 8),
                  const Expanded(
                    child: Text(
                      'AI & CONTENT-AWARE AUTO ENHANCE',
                      style: TextStyle(
                        fontSize: 11,
                        fontWeight: FontWeight.w700,
                        color: IrisTheme.primaryCyan,
                        letterSpacing: 0.5,
                      ),
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 6),
              const Text(
                'Bounded statistical analysis of luminance, Gray-World white balance, and highlight recovery.',
                style: TextStyle(fontSize: 10, color: IrisTheme.textMuted),
              ),
              const SizedBox(height: 10),
              SizedBox(
                width: double.infinity,
                child: ElevatedButton.icon(
                  icon: _isAutoEnhancing
                      ? const SizedBox(
                          width: 12,
                          height: 12,
                          child: CircularProgressIndicator(strokeWidth: 2, color: Colors.black),
                        )
                      : const Icon(Icons.flash_on, size: 14),
                  label: Text(
                    _isAutoEnhancing ? 'Analyzing Pixels...' : 'Auto Enhance Photo',
                    style: const TextStyle(fontSize: 11, fontWeight: FontWeight.bold),
                  ),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: IrisTheme.primaryCyan,
                    foregroundColor: Colors.black,
                    padding: const EdgeInsets.symmetric(vertical: 8),
                    elevation: 0,
                  ),
                  onPressed: _isAutoEnhancing
                      ? null
                      : () async {
                          setState(() => _isAutoEnhancing = true);
                          try {
                            final success = await widget.controller.runPhotoAutoEnhance();
                            if (mounted) {
                              ScaffoldMessenger.of(context).showSnackBar(
                                SnackBar(
                                  content: Text(success
                                      ? 'Photo Auto Enhanced with content-aware tone & color!'
                                      : 'No photo loaded to enhance'),
                                  backgroundColor: success ? IrisTheme.accentGreen : IrisTheme.accentAmber,
                                ),
                              );
                            }
                          } finally {
                            if (mounted) setState(() => _isAutoEnhancing = false);
                          }
                        },
                ),
              ),
            ],
          ),
        ),

        // 2. Active Preset & Intensity Banner
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          decoration: const BoxDecoration(
            color: IrisTheme.surfaceLight,
            border: Border(
              top: BorderSide(color: IrisTheme.border),
              bottom: BorderSide(color: IrisTheme.border),
            ),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  const Icon(Icons.palette_outlined, size: 14, color: IrisTheme.primaryCyan),
                  const SizedBox(width: 6),
                  Expanded(
                    child: Text(
                      activePreset != null ? activePreset.name : 'No Preset Active',
                      style: const TextStyle(fontSize: 12, fontWeight: FontWeight.w700, color: IrisTheme.textPrimary),
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                  if (activePreset != null && isModified)
                    Container(
                      margin: const EdgeInsets.only(right: 6),
                      padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 2),
                      decoration: BoxDecoration(
                        color: IrisTheme.accentAmber.withValues(alpha: 0.2),
                        borderRadius: BorderRadius.circular(3),
                        border: Border.all(color: IrisTheme.accentAmber.withValues(alpha: 0.6)),
                      ),
                      child: const Text(
                        'MODIFIED',
                        style: TextStyle(fontSize: 9, fontWeight: FontWeight.bold, color: IrisTheme.accentAmber),
                      ),
                    ),
                  if (activePreset != null)
                    TextButton(
                      style: TextButton.styleFrom(
                        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                        visualDensity: VisualDensity.compact,
                        foregroundColor: IrisTheme.textMuted,
                      ),
                      onPressed: () => widget.controller.resetPreset(isVideo: false),
                      child: const Text('Reset', style: TextStyle(fontSize: 10)),
                    ),
                ],
              ),
              if (activePreset != null) ...[
                const SizedBox(height: 6),
                Row(
                  children: [
                    const Text('Intensity: ', style: TextStyle(fontSize: 10, color: IrisTheme.textMuted)),
                    Text(
                      '${(intensity * 100).round()}%',
                      style: const TextStyle(fontSize: 10, fontWeight: FontWeight.bold, color: IrisTheme.primaryCyan),
                    ),
                    Expanded(
                      child: SliderTheme(
                        data: SliderTheme.of(context).copyWith(
                          trackHeight: 2,
                          thumbShape: const RoundSliderThumbShape(enabledThumbRadius: 6),
                        ),
                        child: Slider(
                          value: intensity,
                          min: 0.0,
                          max: 1.0,
                          onChanged: (val) => widget.controller.setPresetIntensity(val, isVideo: false),
                        ),
                      ),
                    ),
                  ],
                ),
              ],
            ],
          ),
        ),

        // 3. Action Toolbar (Save Custom, Import)
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
          decoration: const BoxDecoration(
            color: IrisTheme.surfaceDark,
            border: Border(bottom: BorderSide(color: IrisTheme.border)),
          ),
          child: Row(
            children: [
              Expanded(
                child: OutlinedButton.icon(
                  icon: const Icon(Icons.bookmark_add_outlined, size: 12),
                  label: const Text('Save Preset', style: TextStyle(fontSize: 10)),
                  style: OutlinedButton.styleFrom(
                    foregroundColor: IrisTheme.primaryCyan,
                    side: BorderSide(color: IrisTheme.primaryCyan.withValues(alpha: 0.5)),
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
                    visualDensity: VisualDensity.compact,
                  ),
                  onPressed: () => _showSaveCustomPresetDialog(context),
                ),
              ),
              const SizedBox(width: 8),
              Expanded(
                child: OutlinedButton.icon(
                  icon: const Icon(Icons.file_download_outlined, size: 12),
                  label: const Text('Import Look', style: TextStyle(fontSize: 10)),
                  style: OutlinedButton.styleFrom(
                    foregroundColor: IrisTheme.textPrimary,
                    side: const BorderSide(color: IrisTheme.border),
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
                    visualDensity: VisualDensity.compact,
                  ),
                  onPressed: () => _handleImportPreset(context),
                ),
              ),
            ],
          ),
        ),

        // 4. Search Bar
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
          child: TextField(
            style: const TextStyle(fontSize: 11, color: IrisTheme.textPrimary),
            decoration: InputDecoration(
              hintText: 'Search photo presets...',
              hintStyle: const TextStyle(fontSize: 11, color: IrisTheme.textMuted),
              prefixIcon: const Icon(Icons.search, size: 14, color: IrisTheme.textMuted),
              isDense: true,
              contentPadding: const EdgeInsets.symmetric(vertical: 8),
              filled: true,
              fillColor: IrisTheme.surfaceLight,
              border: OutlineInputBorder(
                borderRadius: BorderRadius.circular(4),
                borderSide: BorderSide.none,
              ),
            ),
            onChanged: (val) => setState(() => _presetSearchQuery = val),
          ),
        ),

        // 5. Category Chips
        SizedBox(
          height: 32,
          child: ListView.separated(
            padding: const EdgeInsets.symmetric(horizontal: 12),
            scrollDirection: Axis.horizontal,
            itemCount: categories.length,
            separatorBuilder: (_, __) => const SizedBox(width: 6),
            itemBuilder: (context, idx) {
              final cat = categories[idx];
              final isSelected = _selectedPresetCategory == cat;
              return ChoiceChip(
                label: Text(cat, style: TextStyle(fontSize: 10, color: isSelected ? Colors.black : IrisTheme.textPrimary)),
                selected: isSelected,
                selectedColor: IrisTheme.primaryCyan,
                backgroundColor: IrisTheme.surfaceLight,
                visualDensity: VisualDensity.compact,
                padding: EdgeInsets.zero,
                onSelected: (selected) {
                  if (selected) setState(() => _selectedPresetCategory = cat);
                },
              );
            },
          ),
        ),
        const SizedBox(height: 6),

        // 6. Presets List
        Expanded(
          child: filtered.isEmpty
              ? Center(
                  child: Text(
                    'No presets found in "$_selectedPresetCategory"',
                    style: const TextStyle(fontSize: 11, color: IrisTheme.textMuted),
                  ),
                )
              : ListView.builder(
                  padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
                  itemCount: filtered.length,
                  itemBuilder: (context, idx) {
                    final p = filtered[idx];
                    final isCurrent = activePreset?.id == p.id;
                    final isFav = widget.controller.favoritePresetIds.contains(p.id);

                    return Container(
                      margin: const EdgeInsets.only(bottom: 6),
                      decoration: BoxDecoration(
                        color: isCurrent
                            ? IrisTheme.primaryCyan.withValues(alpha: 0.12)
                            : IrisTheme.surfaceLight,
                        borderRadius: BorderRadius.circular(6),
                        border: Border.all(
                          color: isCurrent ? IrisTheme.primaryCyan : IrisTheme.border,
                        ),
                      ),
                      child: InkWell(
                        borderRadius: BorderRadius.circular(6),
                        onTap: () => widget.controller.applyPreset(p, isVideo: false),
                        child: Padding(
                          padding: const EdgeInsets.all(8),
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Row(
                                children: [
                                  Expanded(
                                    child: Text(
                                      p.name,
                                      style: TextStyle(
                                        fontSize: 12,
                                        fontWeight: FontWeight.w700,
                                        color: isCurrent ? IrisTheme.primaryCyan : IrisTheme.textPrimary,
                                      ),
                                    ),
                                  ),
                                  Container(
                                    padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 1),
                                    decoration: BoxDecoration(
                                      color: IrisTheme.surfaceDark,
                                      borderRadius: BorderRadius.circular(3),
                                    ),
                                    child: Text(
                                      p.category.toUpperCase(),
                                      style: const TextStyle(fontSize: 8, color: IrisTheme.textMuted, fontWeight: FontWeight.bold),
                                    ),
                                  ),
                                  const SizedBox(width: 4),
                                  IconButton(
                                    icon: Icon(
                                      isFav ? Icons.star : Icons.star_border,
                                      size: 14,
                                      color: isFav ? IrisTheme.accentAmber : IrisTheme.textMuted,
                                    ),
                                    visualDensity: VisualDensity.compact,
                                    padding: EdgeInsets.zero,
                                    constraints: const BoxConstraints(minWidth: 20, minHeight: 20),
                                    onPressed: () => widget.controller.toggleFavoritePreset(p.id),
                                  ),
                                  if (!p.isBuiltin)
                                    PopupMenuButton<String>(
                                      icon: const Icon(Icons.more_vert, size: 14, color: IrisTheme.textMuted),
                                      padding: EdgeInsets.zero,
                                      constraints: const BoxConstraints(minWidth: 20, minHeight: 20),
                                      onSelected: (val) async {
                                        if (val == 'duplicate') {
                                          await widget.controller.duplicatePreset(p);
                                        } else if (val == 'delete') {
                                          await widget.controller.deleteCustomPreset(p.id);
                                        } else if (val == 'export') {
                                          final savePaths = await MediaPickerDialog.show(
                                            context,
                                            title: 'Export Preset',
                                            isSaveMode: true,
                                            defaultSaveFileName: '${p.name.replaceAll(' ', '_')}.irispreset',
                                          );
                                          if (savePaths != null && savePaths.isNotEmpty) {
                                            await widget.controller.exportPresetToFile(p, savePaths.first);
                                          }
                                        }
                                      },
                                      itemBuilder: (ctx) => [
                                        const PopupMenuItem(value: 'duplicate', child: Text('Duplicate')),
                                        const PopupMenuItem(value: 'export', child: Text('Export .irispreset')),
                                        const PopupMenuItem(
                                          value: 'delete',
                                          child: Text('Delete', style: TextStyle(color: IrisTheme.accentRed)),
                                        ),
                                      ],
                                    ),
                                ],
                              ),
                              if (p.description.isNotEmpty) ...[
                                const SizedBox(height: 3),
                                Text(
                                  p.description,
                                  style: const TextStyle(fontSize: 10, color: IrisTheme.textMuted),
                                  maxLines: 2,
                                  overflow: TextOverflow.ellipsis,
                                ),
                              ],
                            ],
                          ),
                        ),
                      ),
                    );
                  },
                ),
        ),
      ],
    );
  }

  Future<void> _showSaveCustomPresetDialog(BuildContext context) async {
    final nameCtrl = TextEditingController(text: 'My Custom Look');
    final descCtrl = TextEditingController();
    String category = 'Custom';

    final result = await showDialog<bool>(
      context: context,
      builder: (ctx) => StatefulBuilder(
        builder: (ctx, setDlgState) => AlertDialog(
          backgroundColor: IrisTheme.surfaceDark,
          title: const Text('Save Custom Photo Preset', style: TextStyle(fontSize: 14, fontWeight: FontWeight.bold)),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text('Preset Name', style: TextStyle(fontSize: 11, color: IrisTheme.textMuted)),
              const SizedBox(height: 4),
              TextField(
                controller: nameCtrl,
                style: const TextStyle(fontSize: 12),
                decoration: InputDecoration(
                  filled: true,
                  fillColor: IrisTheme.surfaceLight,
                  border: OutlineInputBorder(borderRadius: BorderRadius.circular(4), borderSide: BorderSide.none),
                ),
              ),
              const SizedBox(height: 12),
              const Text('Category', style: TextStyle(fontSize: 11, color: IrisTheme.textMuted)),
              const SizedBox(height: 4),
              DropdownButtonFormField<String>(
                initialValue: category,
                dropdownColor: IrisTheme.surfaceDark,
                items: ['Custom', 'Portrait', 'Landscape', 'Vivid', 'Vintage', 'Atmosphere', 'Monochrome']
                    .map((c) => DropdownMenuItem(value: c, child: Text(c, style: const TextStyle(fontSize: 12))))
                    .toList(),
                onChanged: (val) {
                  if (val != null) setDlgState(() => category = val);
                },
                decoration: InputDecoration(
                  filled: true,
                  fillColor: IrisTheme.surfaceLight,
                  border: OutlineInputBorder(borderRadius: BorderRadius.circular(4), borderSide: BorderSide.none),
                ),
              ),
              const SizedBox(height: 12),
              const Text('Description (Optional)', style: TextStyle(fontSize: 11, color: IrisTheme.textMuted)),
              const SizedBox(height: 4),
              TextField(
                controller: descCtrl,
                style: const TextStyle(fontSize: 12),
                maxLines: 2,
                decoration: InputDecoration(
                  filled: true,
                  fillColor: IrisTheme.surfaceLight,
                  border: OutlineInputBorder(borderRadius: BorderRadius.circular(4), borderSide: BorderSide.none),
                ),
              ),
            ],
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(ctx, false),
              child: const Text('Cancel', style: TextStyle(color: IrisTheme.textMuted)),
            ),
            ElevatedButton(
              style: ElevatedButton.styleFrom(
                backgroundColor: IrisTheme.primaryCyan,
                foregroundColor: Colors.black,
              ),
              onPressed: () => Navigator.pop(ctx, true),
              child: const Text('Save Preset'),
            ),
          ],
        ),
      ),
    );

    if (result == true && context.mounted) {
      final messenger = ScaffoldMessenger.of(context);
      await widget.controller.saveCustomPreset(
        name: nameCtrl.text,
        category: category,
        description: descCtrl.text,
        isVideo: false,
      );
      if (mounted) {
        messenger.showSnackBar(
          SnackBar(
            content: Text('Saved custom photo preset: "${nameCtrl.text}"'),
            backgroundColor: IrisTheme.accentGreen,
          ),
        );
      }
    }
  }

  Future<void> _handleImportPreset(BuildContext context) async {
    final messenger = ScaffoldMessenger.of(context);
    final paths = await MediaPickerDialog.show(
      context,
      title: 'Import Preset (.irispreset or JSON)',
      initialFilter: FileFilterMode.allFiles,
      allowMultiple: false,
    );
    if (paths != null && paths.isNotEmpty) {
      final ok = await widget.controller.importPresetFromFile(paths.first);
      if (mounted) {
        messenger.showSnackBar(
          SnackBar(
            content: Text(ok ? 'Preset imported successfully!' : 'Failed to import preset: Invalid or corrupted schema'),
            backgroundColor: ok ? IrisTheme.accentGreen : IrisTheme.accentRed,
          ),
        );
      }
    }
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

  Future<void> _showOpenPhotoDialog() async {
    final messenger = ScaffoldMessenger.of(context);
    final selectedPaths = await MediaPickerDialog.show(
      context,
      title: 'Open Photo (Native Disk Probe)',
      initialFilter: FileFilterMode.photos,
      allowMultiple: false,
    );
    if (selectedPaths != null && selectedPaths.isNotEmpty) {
      final path = selectedPaths.first;
      final ok = await widget.controller.loadPhoto(path);
      if (!mounted) return;
      if (ok) {
        messenger.showSnackBar(
          SnackBar(
            content: Text(
              'Loaded ${path.split(Platform.pathSeparator).last} (${widget.controller.photoWidth}x${widget.controller.photoHeight})',
            ),
            backgroundColor: IrisTheme.accentGreen,
          ),
        );
      } else {
        messenger.showSnackBar(
          SnackBar(
            content: Text('Failed to decode $path. Unsupported raster or corrupt image.'),
            backgroundColor: IrisTheme.accentRed,
          ),
        );
      }
    }
  }

  Future<void> _showExportPhotoDialog() async {
    final messenger = ScaffoldMessenger.of(context);
    if (widget.controller.uiImage == null) {
      messenger.showSnackBar(
        const SnackBar(
          content: Text('No photo loaded to export. Please open an image first.'),
          backgroundColor: IrisTheme.accentAmber,
        ),
      );
      return;
    }

    String format = 'PNG';
    int quality = 95;
    final originalPath = widget.controller.currentPhotoPath ?? 'photo';
    final dir = File(originalPath).parent.path;
    final baseName = originalPath.split(Platform.pathSeparator).last.split('.').first;
    String targetPath = '$dir${Platform.pathSeparator}${baseName}_edited.png';

    await showDialog(
      context: context,
      builder: (dialogCtx) {
        return StatefulBuilder(
          builder: (context, setDialogState) {
            return AlertDialog(
              backgroundColor: IrisTheme.surfaceDark,
              title: const Text('Export Photo to Disk', style: TextStyle(color: IrisTheme.textPrimary, fontSize: 16)),
              content: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text('Select target format and quality:', style: TextStyle(color: IrisTheme.textSecondary, fontSize: 12)),
                  const SizedBox(height: 12),
                  Row(
                    children: ['PNG', 'JPEG', 'WebP'].map((f) {
                      final isSelected = format == f;
                      return Padding(
                        padding: const EdgeInsets.only(right: 8),
                        child: ChoiceChip(
                          label: Text(f),
                          selected: isSelected,
                          selectedColor: IrisTheme.primaryCyan,
                          backgroundColor: IrisTheme.surfaceLight,
                          labelStyle: TextStyle(color: isSelected ? Colors.black : IrisTheme.textPrimary),
                          onSelected: (_) {
                            setDialogState(() {
                              format = f;
                              final ext = f.toLowerCase() == 'jpeg' ? 'jpg' : f.toLowerCase();
                              targetPath = '$dir${Platform.pathSeparator}${baseName}_edited.$ext';
                            });
                          },
                        ),
                      );
                    }).toList(),
                  ),
                  if (format != 'PNG') ...[
                    const SizedBox(height: 12),
                    Text('Quality: $quality%', style: const TextStyle(fontSize: 11, color: IrisTheme.textSecondary)),
                    Slider(
                      value: quality.toDouble(),
                      min: 10,
                      max: 100,
                      activeColor: IrisTheme.primaryCyan,
                      onChanged: (v) => setDialogState(() => quality = v.round()),
                    ),
                  ],
                  const SizedBox(height: 12),
                  Text('Output file: $targetPath', style: const TextStyle(fontSize: 11, color: IrisTheme.textMuted)),
                ],
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.pop(dialogCtx),
                  child: const Text('Cancel'),
                ),
                ElevatedButton(
                  style: ElevatedButton.styleFrom(
                    backgroundColor: IrisTheme.primaryCyan,
                    foregroundColor: Colors.black,
                  ),
                  onPressed: () async {
                    Navigator.pop(dialogCtx);
                    final formatCode = format == 'PNG' ? 0 : (format == 'JPEG' ? 1 : 2);
                    final success = await widget.controller.exportPhoto(
                      outputPath: targetPath,
                      formatCode: formatCode,
                      quality: quality,
                    );
                    if (!mounted) return;
                    if (success) {
                      final size = File(targetPath).lengthSync();
                      messenger.showSnackBar(
                        SnackBar(
                          content: Text('Exported successfully: $targetPath ($size bytes)'),
                          backgroundColor: IrisTheme.accentGreen,
                        ),
                      );
                    } else {
                      messenger.showSnackBar(
                        SnackBar(
                          content: Text('Export failed for $targetPath'),
                          backgroundColor: IrisTheme.accentRed,
                        ),
                      );
                    }
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
    final hw = widget.controller.project.hardwareInfo;
    final w = widget.controller.photoWidth;
    final h = widget.controller.photoHeight;
    final mp = w > 0 && h > 0 ? ((w * h) / 1000000.0).toStringAsFixed(2) : '0';

    showDialog(
      context: context,
      builder: (context) {
        return AlertDialog(
          backgroundColor: IrisTheme.surfaceDark,
          title: const Text('EXIF & Native Pipeline Diagnostics', style: TextStyle(color: IrisTheme.textPrimary, fontSize: 16)),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text('Source File: ${widget.controller.currentPhotoPath ?? "None (No photo open)"}',
                  style: const TextStyle(color: IrisTheme.textPrimary, fontSize: 12)),
              const SizedBox(height: 6),
              Text('Resolution: ${w}x$h ($mp Megapixels)',
                  style: const TextStyle(color: IrisTheme.textSecondary, fontSize: 12)),
              const SizedBox(height: 6),
              const Text('Color Space: Rec.709 / sRGB Linear Float32 internal',
                  style: TextStyle(color: IrisTheme.textSecondary, fontSize: 12)),
              const SizedBox(height: 6),
              const Text('Pixel Format: 32-bit RGBA (8-bit per channel linear)',
                  style: TextStyle(color: IrisTheme.textSecondary, fontSize: 12)),
              const SizedBox(height: 6),
              Text('Hardware Acceleration: ${hw.deviceName} (${hw.backendName})',
                  style: const TextStyle(color: IrisTheme.primaryCyan, fontSize: 12)),
              const SizedBox(height: 6),
              const Text('Vector Pipeline: SIMD AVX2 / SSE4.1 zero-copy tile memory',
                  style: TextStyle(color: IrisTheme.textSecondary, fontSize: 12)),
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

class _SplitClipper extends CustomClipper<Rect> {
  final double splitRatio;
  _SplitClipper(this.splitRatio);

  @override
  Rect getClip(Size size) {
    return Rect.fromLTWH(0, 0, size.width * splitRatio.clamp(0.0, 1.0), size.height);
  }

  @override
  bool shouldReclip(covariant _SplitClipper oldClipper) => oldClipper.splitRatio != splitRatio;
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
  final IrisHistogram histogram;
  _HistogramPainter({required this.histogram});

  @override
  void paint(Canvas canvas, Size size) {
    if (size.width <= 0 || size.height <= 0) return;

    int maxVal = 1;
    for (int i = 0; i < 256; i++) {
      if (histogram.rBins[i] > maxVal) maxVal = histogram.rBins[i];
      if (histogram.gBins[i] > maxVal) maxVal = histogram.gBins[i];
      if (histogram.bBins[i] > maxVal) maxVal = histogram.bBins[i];
      if (histogram.lumaBins[i] > maxVal) maxVal = histogram.lumaBins[i];
    }

    final paint = Paint()
      ..strokeWidth = 1.2
      ..style = PaintingStyle.stroke;

    // Draw Red channel
    paint.color = Colors.redAccent.withValues(alpha: 0.75);
    _drawBins(canvas, size, paint, histogram.rBins, maxVal);

    // Draw Green channel
    paint.color = Colors.greenAccent.withValues(alpha: 0.75);
    _drawBins(canvas, size, paint, histogram.gBins, maxVal);

    // Draw Blue channel
    paint.color = Colors.lightBlueAccent.withValues(alpha: 0.85);
    _drawBins(canvas, size, paint, histogram.bBins, maxVal);

    // Draw Luma channel (white overlay)
    paint.color = Colors.white.withValues(alpha: 0.4);
    paint.strokeWidth = 0.8;
    _drawBins(canvas, size, paint, histogram.lumaBins, maxVal);
  }

  void _drawBins(Canvas canvas, Size size, Paint paint, List<int> bins, int maxVal) {
    if (bins.length < 256 || maxVal <= 0) return;
    final path = Path();
    final dx = size.width / 255.0;

    for (int i = 0; i < 256; i++) {
      final normalized = (bins[i] / maxVal).clamp(0.0, 1.0);
      final y = size.height - (normalized * (size.height - 4));
      final x = i * dx;
      if (i == 0) {
        path.moveTo(x, y);
      } else {
        path.lineTo(x, y);
      }
    }
    canvas.drawPath(path, paint);
  }

  @override
  bool shouldRepaint(covariant _HistogramPainter oldDelegate) {
    return oldDelegate.histogram != histogram;
  }
}
