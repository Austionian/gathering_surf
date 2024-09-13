import { removeElement, appendElements } from "./utilities";

/**
 * Function to update the frontend when there is a JS error on the client,
 * prevents the front from staying in an endless loading state.
 */
export function forecastFailed(e: Error) {
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
