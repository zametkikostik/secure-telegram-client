import 'dart:async';

/// P2P сервис для подключения к децентрализованной сети
class P2PService {
  bool _isConnected = false;
  final _onMessageController = StreamController<Map<String, dynamic>>.broadcast();
  
  Stream<Map<String, dynamic>> get onMessage => _onMessageController.stream;
  bool get isConnected => _isConnected;
  
  /// Подключение к P2P сети
  Future<void> connect() async {
    print('🔌 Connecting to P2P network...');
    
    // Эмуляция подключения
    await Future.delayed(const Duration(seconds: 2));
    
    _isConnected = true;
    print('✅ Connected to P2P network');
  }
  
  /// Отправить сообщение
  Future<void> sendMessage({
    required String peerId,
    required Map<String, dynamic> message,
  }) async {
    if (!_isConnected) {
      throw Exception('Not connected to P2P network');
    }
    
    print('📤 Sending message to $peerId: $message');
  }
  
  /// Получить сообщения
  void receiveMessage(Map<String, dynamic> message) {
    _onMessageController.add(message);
  }
  
  /// Отключиться
  Future<void> disconnect() async {
    _isConnected = false;
    await _onMessageController.close();
    print('🔌 Disconnected from P2P network');
  }
}
