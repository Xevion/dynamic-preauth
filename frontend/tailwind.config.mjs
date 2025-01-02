/** @type {import('tailwindcss').Config} */
export default {
  content: ["./src/**/*.{astro,html,js,jsx,ts,tsx}"],
  theme: {
    extend: {
      animation: {
        "pulse-border": "pulse-border 1s cubic-bezier(0.4, 0, 0.6, 1) infinite",
        "pulse-dark": "pulse-dark 2.5s ease-in-out infinite",
      },
      keyframes: {
        "pulse-border": {
          "50%": {
            borderColor: "rgba(100, 100, 100)",
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
