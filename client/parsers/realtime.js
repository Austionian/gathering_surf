import {
  setText,
  setStyleAttribute,
  removeElements,
  nonNull,
  removeElement,
  removeStyle,
  removeHidden,
  setAttribute,
} from "../utilities";

/**
 * @typedef {Object} LatestData
 * @property {string} quality_color - The hexcode of the quality.
 * @property {string} quality_text - The computed text of the quality.
 * @property {string} water_temp - The latest water temperature.
 * @property {number} wind_direction - The current wind direction.
 * @property {string} wind_speed - The current wind speed.
 * @property {string} gusts - The current wind gust.
 * @property {string} air_temp
 * @property {?string} wave_height
 * @property {?string} wave_direction
 * @property {?string} wave_period
 * @property {string} as_of
 * @property {boolean} loaded_from_fallback - Wether the latest data used a bouy or land data.
 */

/**
 * Takes the latest data JSON and updates the HTML
 *
 * @param {LatestData} data
 */
export function parseRealtime(data) {
  if (data.wave_height) {
    setText("current-wave-height", data.wave_height);
    setStyleAttribute(
      "wave-icon",
      `transform: rotate(${data.wave_direction}deg);`,
    );
    removeElements(".wavey");
  }

  if (data.wave_period) {
    setText("current-wave-period", data.wave_period);
    removeElement("wavey-period-loader");
  }

  // Sometimes the forecast will have loaded before the realtime data.
  // Most of the time realtime quality should supersede forecast.
  if (
    // If it hasn't been set yet, always set it.
    nonNull(document.getElementById("wave-quality-text")).innerText === "" ||
    // Otherwise if realtime has wave height, this always supersedes forecast estimates
    // and 'Flat' condition will be handled appropriately
    data.wave_height ||
    // If the forecast data has loaded and the computed wave height is greater than
    // or equal to one, then load the realtime computed quality. If this isn't here
    // forecasted 'Flat' conditions wouldn't be handled correctly.
    parseFloat(
      nonNull(document.getElementById("current-wave-height")).innerText ?? "0",
    ) >= 1
  ) {
    setStyleAttribute(
      "wave-quality",
      `background-color: ${data.quality_color};`,
    );
    setText("wave-quality-text", data.quality_text);
    setStyleAttribute("wave-quality-text", `color: ${data.quality_color}`);
    removeElements(".wave-quality-loader");
  }

  setText("current-water-temp", data.water_temp);
  setText("current-air-temp", data.air_temp);
  setText("current-air-temp-2", data.air_temp);

  setText("wind", getWindData(data));
  setText("as-of", `As of ${data.as_of}`);
  setStyleAttribute(
    "wind-icon",
    `transform: rotate(${data.wind_direction + 180}deg);`,
  );

  removeElements(".latest-loader");
  removeHidden("wind-icon-container");
  removeHidden("wave-icon-container");
  removeStyle("as-of-container", "animate-pulse");
  removeHidden("wave-quality");

  if (data.loaded_from_fallback) {
    removeHidden("wave-fallback-icon");
  }

  let oneDayMs = 60 * 60 * 24 * 1_000;
  if (new Date(data.as_of) < new Date() - oneDayMs) {
    setStyleAttribute(
      "as-of-container",
      "background-color: #facc15; color: #000; display: flex; align-items: center; flex-direction: row-reverse; gap: 8px",
    );
    document.getElementById("as-of-container").innerHTML +=
      `<svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true" data-slot="icon">
                <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z" />
              </svg>`;
  }
}

/**
 * Takes the latest data JSON and creates the wind string
 *
 * @param {LatestData} data
 */
const getWindData = (data) =>
  data.wind_speed === data.gusts
    ? data.wind_speed
    : data.gusts === "0"
      ? data.wind_speed
      : `${data.wind_speed}-${data.gusts}`;
