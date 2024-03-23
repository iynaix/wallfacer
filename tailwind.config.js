/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    // include all rust, html and css files in the src directory
    "./src/**/*.{rs,html,css}",
    // include all html files in the output (dist) directory
    "./dist/**/*.html",
  ],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        // indigo-600
        active: "#4f46e5",
      },
    },
  },
  plugins: [
    require("@catppuccin/tailwindcss")({
      // which flavour of colours to use by default, in the `:root`
      defaultFlavour: "mocha",
    }),
  ],
}
