import 'package:flutter/material.dart';

/// Экран звонков
class CallsScreen extends StatelessWidget {
  const CallsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Звонки'),
        actions: [
          IconButton(
            icon: const Icon(Icons.add_call),
            onPressed: () {},
          ),
        ],
      ),
      body: ListView.builder(
        itemCount: 5,
        itemBuilder: (context, index) {
          return ListTile(
            leading: CircleAvatar(
              child: Icon(
                index % 2 == 0 ? Icons.call_received : Icons.call_made,
                color: index % 2 == 0 ? Colors.green : Colors.red,
              ),
            ),
            title: Text('Пользователь $index'),
            subtitle: Text(
              index % 2 == 0 ? 'Входящий' : 'Исходящий',
              style: TextStyle(
                color: index % 2 == 0 ? Colors.green : Colors.red,
              ),
            ),
            trailing: IconButton(
              icon: const Icon(Icons.call),
              onPressed: () {},
            ),
          );
        },
      ),
    );
  }
}
