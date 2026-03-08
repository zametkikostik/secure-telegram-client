import 'dart:async';
import 'dart:ffi';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import '../rust/frb_generated.dart';

/// FFI мост к Rust ядру Liberty Reach
class RustBridge {
  static RustBridge? _instance;
  RustApi? _api;
  
  RustBridge._();
  
  factory RustBridge() {
    _instance ??= RustBridge._();
    return _instance!;
  }
  
  /// Инициализация FFI моста
  Future<void> init() async {
    await RustApiImpl.init();
    _api = RustApiImpl();
    print('✅ Rust FFI bridge initialized');
  }
  
  /// Создать ядро
  Future<void> createCore({
    required String dbPath,
    required List<int> encryptionKey,
    required String userId,
    required String username,
  }) async {
    await _api?.frbCreateCore(
      dbPath: dbPath,
      encryptionKey: encryptionKey,
      userId: userId,
      username: username,
    );
  }
  
  /// Отправить команду
  Future<void> sendCommand(dynamic command) async {
    await _api?.frbSendCommand(command: command);
  }
  
  /// Получить Peer ID
  Future<String> getPeerId() async {
    return await _api?.frbGetPeerId() ?? 'unknown';
  }
  
  /// Остановить ядро
  Future<void> shutdown() async {
    await _api?.frbShutdown();
  }
  
  /// Получить версию
  Future<String> getVersion() async {
    return await _api?.frbGetVersion() ?? '2.0.0';
  }
}
