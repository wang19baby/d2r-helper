import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 7340,
    strictPort: true,
    host: '127.0.0.1',
  },
  build: {
    target: ['es2022', 'chrome108', 'safari16'],
    // Switch to esbuild to avoid oxc parser bugs with catch {} and inline SVGs
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
  },
})
