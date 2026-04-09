import { useEffect } from 'react';
import { useConnectionStore, selectConnectionMode } from '../services/connectionStore';

// ============================================================================
// Styles
// ============================================================================

const statusColors = {
  backend: 'bg-green-500',
  p2p: 'bg-yellow-500',
  offline: 'bg-red-500',
};

const statusText = {
  backend: 'Connected to server',
  p2p: 'P2P Mode (Cloudflare)',
  offline: 'Offline',
};

const statusIcons = {
  backend: '🟢',
  p2p: '🟡',
  offline: '🔴',
};

// ============================================================================
// Component
// ============================================================================

export function ConnectionStatus() {
  const mode = useConnectionStore(selectConnectionMode);
  const checkConnections = useConnectionStore((s) => s.checkConnections);
  const startMonitoring = useConnectionStore((s) => s.startMonitoring);
  const stopMonitoring = useConnectionStore((s) => s.stopMonitoring);

  useEffect(() => {
    // Check on mount
    checkConnections();

    // Start monitoring
    startMonitoring();

    return () => {
      stopMonitoring();
    };
  }, [checkConnections, startMonitoring, stopMonitoring]);

  const handleClick = async () => {
    await checkConnections();
  };

  return (
    <button
      onClick={handleClick}
      className="fixed top-4 right-4 z-50 flex items-center gap-2 px-3 py-1.5 rounded-full bg-gray-800/90 backdrop-blur-sm border border-gray-700 hover:bg-gray-700/90 transition-colors cursor-pointer"
      title={`Connection: ${statusText[mode]}. Click to refresh.`}
    >
      <span className="text-sm">{statusIcons[mode]}</span>
      <span className="text-xs text-gray-300">{statusText[mode]}</span>
    </button>
  );
}
