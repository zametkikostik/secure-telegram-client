import { useState, useEffect } from 'react'
import { api } from './services/apiClient'
import { fallbackApi } from './services/fallbackClient'
import { useConnectionStore } from './services/connectionStore'
import { AuthPage } from './components/AuthPage'
import { ChatList } from './components/ChatListReal'
import { ChatWindow } from './components/ChatWindowReal'
import ThemeToggle from './components/ThemeToggle'
import LanguageSwitcher from './components/LanguageSwitcher'
import { ConnectionStatus } from './components/ConnectionStatus'
import { IncomingCallModal } from './components/IncomingCallModal'
import { ActiveCallOverlay } from './components/ActiveCallOverlay'
import { useCallStore } from './services/callStore'
import { WebRTCService } from './services/webrtcService'

function App() {
  const [authenticated, setAuthenticated] = useState(false)
  const [checking, setChecking] = useState(true)
  const [selectedChat, setSelectedChat] = useState<string | null>(null)

  useEffect(() => {
    const check = async () => {
      if (api.isAuthenticated()) {
        try { await api.getMe(); setAuthenticated(true) } catch { api.logout(); setAuthenticated(false) }
      }
      setChecking(false)
    }
    check()
  }, [])

  // Initialize WebRTC service after authentication
  useEffect(() => {
    if (!authenticated) return;

    const service = WebRTCService.getInstance();
    // WebSocket client will be set when available
    // Handlers are set in IncomingCallModal and ActiveCallOverlay

    return () => {
      // Cleanup on unmount
    };
  }, [authenticated]);

  const handleAuth = () => setAuthenticated(true)
  const handleLogout = () => {
    api.logout();
    setAuthenticated(false);
    setSelectedChat(null);
    // End any active call
    WebRTCService.getInstance().endCall('logout');
    useCallStore.getState().resetCallState();
  }

  if (checking) return <div className="min-h-screen bg-gray-900 flex items-center justify-center text-white">Loading...</div>
  if (!authenticated) return <AuthPage onAuth={handleAuth} />

  return (
    <div className="flex h-screen bg-gray-900 text-white" dir="ltr">
      {/* Global Call Modals */}
      <IncomingCallModal />
      <ActiveCallOverlay />

      {/* Connection Status Indicator */}
      <ConnectionStatus />

      <ThemeToggle />
      <LanguageSwitcher />
      <ChatList selectedChat={selectedChat} onSelectChat={setSelectedChat} onLogout={handleLogout} />
      {selectedChat ? <ChatWindow chatId={selectedChat} onBack={() => setSelectedChat(null)} />
        : <div className="flex-1 flex items-center justify-center text-gray-500">
            <div className="text-center">
              <p className="text-2xl mb-2">🔒 Secure Messenger</p>
              <p className="text-sm">Select a chat or create a new one</p>
            </div>
          </div>}
    </div>
  )
}

export default App
