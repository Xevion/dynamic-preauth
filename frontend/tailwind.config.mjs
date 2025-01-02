/** @type {import('tailwindcss').Config} */
export default {
  content: ["./src/**/*.{astro,html,js,jsx,ts,tsx}"],
  theme: {
    extend: {
      animation: {
        "pulse-border": "pulse-border 1s ease-in-out infinite",
        "pulse-dark": "pulse-dark 2.5s ease-in-out infinite",
      },
      keyframes: {
        "pulse-border": {
          "0%, 100%": {
            "--tw-border-opacity": "1",
          },
          "50%": {
            "--tw-border-opacity": "0.5",
          },
        },
        "pulse-dark": {
          "0%, 100%": {
            backgroundColor: "#0A3026",
          },
          "50%": {
            backgroundColor: "#053B2D",
          },
        },
      },
      fontFamily: {
        bebas: ["Bebas Neue", "sans-serif"],
        inter: ["Inter", "sans-serif"],
      },
    },
  },
  plugins: [require("@tailwindcss/typography")],
};
