import 'dart:convert';
import 'package:crypto/crypto.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// Сервис аутентификации
class AuthService {
  String? _userId;
  String? _username;
  String? _sessionToken;
  
  String? get userId => _userId;
  String? get username => _username;
  bool get isAuthenticated => _userId != null;
  
  /// Войти
  Future<bool> login({
    required String username,
    required String password,
  }) async {
    // Генерация user ID из username
    _userId = sha256.convert(utf8.encode(username)).toString().substring(0, 16);
    _username = username;
    _sessionToken = sha256.convert(utf8.encode(DateTime.now().toString())).toString();
    
    // Сохранение сессии
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString('user_id', _userId!);
    await prefs.setString('username', _username!);
    await prefs.setString('session_token', _sessionToken!);
    
    print('✅ Logged in as $_username ($_userId)');
    return true;
  }
  
  /// Выйти
  Future<void> logout() async {
    _userId = null;
    _username = null;
    _sessionToken = null;
    
    final prefs = await SharedPreferences.getInstance();
    await prefs.clear();
    
    print('👋 Logged out');
  }
  
  /// Загрузить сессию
  Future<bool> loadSession() async {
    final prefs = await SharedPreferences.getInstance();
    
    _userId = prefs.getString('user_id');
    _username = prefs.getString('username');
    _sessionToken = prefs.getString('session_token');
    
    return isAuthenticated;
  }
}
