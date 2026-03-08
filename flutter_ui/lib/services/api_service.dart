import 'dart:convert';
import 'package:http/http.dart' as http;

/// API сервис для Cloudflare Worker
class ApiService {
  static const String baseUrl = 'https://secure-messenger-push.zametkikostik.workers.dev';
  
  Future<void> init() async {
    print('✅ API Service initialized');
  }
  
  /// Отправить сообщение
  Future<void> sendMessage({
    required String recipientId,
    required String content,
  }) async {
    final response = await http.post(
      Uri.parse('$baseUrl/api/send'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'recipient_id': recipientId,
        'content': content,
      }),
    );
    
    if (response.statusCode != 200) {
      throw Exception('Failed to send message');
    }
  }
  
  /// Получить сообщения
  Future<List<dynamic>> getMessages({
    required String chatId,
    int limit = 50,
  }) async {
    final response = await http.get(
      Uri.parse('$baseUrl/api/messages?chat_id=$chatId&limit=$limit'),
    );
    
    if (response.statusCode == 200) {
      return jsonDecode(response.body);
    }
    
    return [];
  }
  
  /// Начать звонок
  Future<String> startCall({
    required String calleeId,
    required String callType,
  }) async {
    final response = await http.post(
      Uri.parse('$baseUrl/api/call/start'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'callee_id': calleeId,
        'call_type': callType,
      }),
    );
    
    if (response.statusCode == 200) {
      final data = jsonDecode(response.body);
      return data['call_id'];
    }
    
    throw Exception('Failed to start call');
  }
  
  /// Проверка статуса
  Future<Map<String, dynamic>> getStatus() async {
    final response = await http.get(Uri.parse('$baseUrl/api/status'));
    
    if (response.statusCode == 200) {
      return jsonDecode(response.body);
    }
    
    return {'status': 'offline'};
  }
}
