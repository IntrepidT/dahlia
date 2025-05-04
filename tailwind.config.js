/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./index.html", "./src/*.rs", "./src/**/*.rs", "./style/**/*.{html,js,rs}"],
  theme: {
    extend: {
      backgroundColor: {
        'page': '#f9f9f8',
      },
      fontFamily: {
        'custom': ["KinderPeeps"],
      },
    },
  },
  plugins: [
    function({ addBase }) {
      addBase({
        '::-webkit-scrollbar': {
          width: '6px',
          height: '6px',
        },
        '::-webkit-scrollbar-track': {
          background: 'transparent',
        },
        '::-webkit-scrollbar-thumb': {
          backgroundColor: 'rgba(203, 213, 225, 0.5)',
          borderRadius: '20px',
        },
        '::-webkit-scrollbar-thumb:hover': {
          backgroundColor: 'rgba(148, 163, 184, 0.7)',
        },
        '*': {
          scrollbarWidth: 'thin',
          scrollbarColor: 'rgba(203, 213, 225, 0.5) transparent',
        },
      });
    },
  ],
}

