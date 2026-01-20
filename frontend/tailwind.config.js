/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        coffee: {
          50: '#FDF8F3',
          100: '#F5E6D3',
          200: '#E8CBA7',
          300: '#D4A574',
          400: '#B8824D',
          500: '#8B5A2B',
          600: '#6B4423',
          700: '#4A2C17',
          800: '#2D1A0E',
          900: '#1A0F08',
        }
      },
      fontFamily: {
        sans: ['Sarabun', 'sans-serif'],
      }
    },
  },
  plugins: [],
};
