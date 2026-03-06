/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#f0f0ff',
          100: '#e0e0ff',
          200: '#c0c0ff',
          300: '#a0a0ff',
          400: '#8080ff',
          500: '#6200ee',
          600: '#5000d0',
          700: '#4000b0',
          800: '#300090',
          900: '#200070',
        },
      },
    },
  },
  plugins: [],
}
