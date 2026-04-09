// Mock Tauri shell for web development
export const open = async (url: string) => {
  console.log(`[Tauri Mock] open: ${url}`);
  window.open(url, '_blank');
};
