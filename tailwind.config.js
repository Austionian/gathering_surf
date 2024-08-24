/** @type {import('tailwindcss').Config} */
const colors = require("tailwindcss/colors");

module.exports = {
  content: ["./templates/**/*.{html,js}"],
  pulgins: [require("@tailwindcss/forms")],
  theme: {
    extend: {
      colors: {
        primary: colors.orange,
        secondary: colors.teal,
      },
    },
  },
};
