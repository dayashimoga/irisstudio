import 'dart:convert';
import 'dart:io';
import 'package:flutter/foundation.dart';

class RecentProjectItem {
  final String id;
  String projectPath;
  String projectName;
  DateTime lastModified;
  String projectType; // 'video', 'photo', 'audio'
  int assetCount;
  double durationSeconds;
  bool isMissing;

  RecentProjectItem({
    required this.id,
    required this.projectPath,
    required this.projectName,
    required this.lastModified,
    required this.projectType,
    required this.assetCount,
    required this.durationSeconds,
    this.isMissing = false,
  });

  Map<String, dynamic> toJson() => {
        'id': id,
        'projectPath': projectPath,
        'projectName': projectName,
        'lastModified': lastModified.toIso8601String(),
        'projectType': projectType,
        'assetCount': assetCount,
        'durationSeconds': durationSeconds,
      };

  factory RecentProjectItem.fromJson(Map<String, dynamic> json) {
    final path = json['projectPath'] as String? ?? '';
    final exists = path.isNotEmpty && File(path).existsSync();

    return RecentProjectItem(
      id: json['id'] as String? ?? 'rp_${DateTime.now().millisecondsSinceEpoch}',
      projectPath: path,
      projectName: json['projectName'] as String? ?? 'Untitled Project',
      lastModified: json['lastModified'] != null
          ? DateTime.tryParse(json['lastModified'] as String) ?? DateTime.now()
          : DateTime.now(),
      projectType: json['projectType'] as String? ?? 'video',
      assetCount: json['assetCount'] as int? ?? 0,
      durationSeconds: (json['durationSeconds'] as num?)?.toDouble() ?? 0.0,
      isMissing: !exists,
    );
  }

  static String get storageFilePath {
    final home = Platform.environment['USERPROFILE'] ??
        Platform.environment['HOME'] ??
        '.';
    return '$home${Platform.pathSeparator}.iris_recent_projects.json';
  }

  static Future<List<RecentProjectItem>> loadRecentProjects() async {
    try {
      final f = File(storageFilePath);
      if (!await f.exists()) {
        return [];
      }
      final content = await f.readAsString();
      final list = jsonDecode(content) as List<dynamic>;
      final items = list.map((e) => RecentProjectItem.fromJson(e as Map<String, dynamic>)).toList();
      items.sort((a, b) => b.lastModified.compareTo(a.lastModified));
      return items;
    } catch (e) {
      debugPrint('Error loading recent projects: $e');
      return [];
    }
  }

  static Future<void> saveRecentProjects(List<RecentProjectItem> items) async {
    try {
      final f = File(storageFilePath);
      final jsonStr = jsonEncode(items.map((i) => i.toJson()).toList());
      await f.writeAsString(jsonStr, flush: true);
    } catch (e) {
      debugPrint('Error saving recent projects: $e');
    }
  }

  static Future<void> recordProject({
    required String projectPath,
    required String projectName,
    required String projectType,
    required int assetCount,
    required double durationSeconds,
  }) async {
    final items = await loadRecentProjects();
    items.removeWhere((i) => i.projectPath == projectPath);

    items.insert(
      0,
      RecentProjectItem(
        id: 'rp_${DateTime.now().millisecondsSinceEpoch}',
        projectPath: projectPath,
        projectName: projectName,
        lastModified: DateTime.now(),
        projectType: projectType,
        assetCount: assetCount,
        durationSeconds: durationSeconds,
        isMissing: false,
      ),
    );

    // Keep top 20 recent projects
    if (items.length > 20) {
      items.removeRange(20, items.length);
    }

    await saveRecentProjects(items);
  }
}
