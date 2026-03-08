import 'package:flutter/material.dart';

/// Экран настроек
class SettingsScreen extends StatelessWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Настройки'),
      ),
      body: ListView(
        children: [
          const _SettingsSection(
            title: 'Сообщения',
            children: [
              _SettingsTile(
                icon: Icons.chat,
                title: 'Чаты',
                subtitle: 'История, синхронизация',
              ),
              _SettingsTile(
                icon: Icons.notifications,
                title: 'Уведомления',
                subtitle: 'Звук, вибрация',
              ),
            ],
          ),
          const _SettingsSection(
            title: 'Приватность',
            children: [
              _SettingsTile(
                icon: Icons.lock,
                title: 'Конфиденциальность',
                subtitle: 'Кто видит мой профиль',
              ),
              _SettingsTile(
                icon: Icons.shield,
                title: 'Безопасность',
                subtitle: '2FA, шифрование',
              ),
            ],
          ),
          const _SettingsSection(
            title: 'Внешний вид',
            children: [
              _SettingsTile(
                icon: Icons.palette,
                title: 'Тема',
                subtitle: 'Светлая / Тёмная',
              ),
              _SettingsTile(
                icon: Icons.wallpaper,
                title: 'Обои',
                subtitle: 'Фон чатов',
              ),
            ],
          ),
          const _SettingsSection(
            title: 'Данные',
            children: [
              _SettingsTile(
                icon: Icons.storage,
                title: 'Хранилище',
                subtitle: 'Использование памяти',
              ),
              _SettingsTile(
                icon: Icons.cloud_upload,
                title: 'Экспорт',
                subtitle: 'Сохранить данные',
              ),
            ],
          ),
          const _SettingsSection(
            title: 'О приложении',
            children: [
              _SettingsTile(
                icon: Icons.info,
                title: 'Версия',
                subtitle: '2.0.0',
              ),
              _SettingsTile(
                icon: Icons.code,
                title: 'Исходный код',
                subtitle: 'GitHub',
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class _SettingsSection extends StatelessWidget {
  final String title;
  final List<Widget> children;
  
  const _SettingsSection({
    required this.title,
    required this.children,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
          child: Text(
            title,
            style: TextStyle(
              color: Colors.blue,
              fontWeight: FontWeight.bold,
            ),
          ),
        ),
        ...children,
      ],
    );
  }
}

class _SettingsTile extends StatelessWidget {
  final IconData icon;
  final String title;
  final String subtitle;
  
  const _SettingsTile({
    required this.icon,
    required this.title,
    required this.subtitle,
  });

  @override
  Widget build(BuildContext context) {
    return ListTile(
      leading: Icon(icon),
      title: Text(title),
      subtitle: Text(subtitle),
      trailing: const Icon(Icons.chevron_right),
      onTap: () {},
    );
  }
}
