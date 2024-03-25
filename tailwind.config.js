/** @type {import('tailwindcss').Config} */
const colors = require("tailwindcss/colors");

module.exports = {
  content: ["./templates/**/*.{html,js}"],
  theme: {
    extend: {
      colors: {
        primary: colors.orange,
        secondary: colors.pink,
        third: colors.purple,
      },
    },
  },
};
