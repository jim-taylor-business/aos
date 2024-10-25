/** @type {import('tailwindcss').Config} */
module.exports = {
  theme: {
    extend: {
      screens: {
        "3xl": "1920px",
        "4xl": "2560px",
        "5xl": "3840px",
        "6xl": "5120px",
        "7xl": "8640px",
      },
      keyframes: {
        // popin: {
        //   from: { opacity: 0, display: "none" },
        //   to: { opacity: 1, display: "block" },
        // },
        // wiggle: {
        //   "0%, 100%": { transform: "rotate(-3deg)", opacity: 0 },
        //   "50%": { transform: "rotate(3deg)", opacity: 1 },
        // },
        // popout: {
        //   from: {
        //     display: "none",
        //   },
        //   to: {
        //     display: "block",
        //   },
        // },
        popdown: {
          from: {
            // opacity: 0,
            height: "0rem",
          },
          to: {
            // opacity: 1,
            height: "3rem",
          },
        },
      },
    },
  },
  content: {
    files: ["*.html", "./src/**/*.rs"],
  },
  plugins: [require("@tailwindcss/typography"), require("daisyui")],
  daisyui: {
    themes: ["retro", "light", "dark"],
  },
  darkMode: ["class", '[data-theme="dark"]'],
};
