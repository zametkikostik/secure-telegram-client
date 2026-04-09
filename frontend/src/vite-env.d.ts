/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_WEBRTC_VIDEO_ENABLED?: string;
  readonly VITE_WEBRTC_MAX_BITRATE?: string;
  readonly VITE_WEBRTC_TURN_SERVER?: string;
  readonly VITE_WEBRTC_TURN_USERNAME?: string;
  readonly VITE_WEBRTC_TURN_CREDENTIAL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
