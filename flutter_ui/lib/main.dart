import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'screens/chat_list_screen.dart';
import 'services/auth_service.dart';
import 'services/p2p_service.dart';
import 'services/api_service.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  
  // Инициализация сервисов
  final authService = AuthService();
  final apiService = ApiService();
  final p2pService = P2PService();
  
  await apiService.init();
  await p2pService.connect();
  
  runApp(
    MultiProvider(
      providers: [
        Provider<AuthService>.value(value: authService),
        Provider<ApiService>.value(value: apiService),
        Provider<P2PService>.value(value: p2pService),
      ],
      child: const LibertyReachApp(),
    ),
  );
}

class LibertyReachApp extends StatelessWidget {
  const LibertyReachApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Liberty Reach Messenger',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: Colors.blue,
          brightness: Brightness.light,
        ),
        useMaterial3: true,
      ),
      darkTheme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: Colors.blue,
          brightness: Brightness.dark,
        ),
        useMaterial3: true,
      ),
      themeMode: ThemeMode.system,
      home: const ChatListScreen(),
    );
  }
}
