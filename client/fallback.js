import { removeElement, appendElements } from "./utilities";

/**
 * @param {Error} e
 */
export function forecastFailed(e) {
  removeElement("forecast-container");

  appendElements(
    "forecast-error",
    `
    <div class="p-12 flex flex-col items-center align-middle justify-center text-center">
      <h2 class="text-xl font-mono">
      Error loading forecast data - please refresh the page or try again later.
      </h2>
      <p>${e}</p>
    </div>
    `,
  );
}
