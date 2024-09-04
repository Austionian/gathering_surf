import {
  removeHidden,
  removeElements,
  setText,
  setStyleAttribute,
} from "../utilities";

/**
 * @typedef {Object} WaterQualityData
 * @property {'Open' | 'Closed' | 'Advisory' | 'Closed for season'} water_quality - The latest water quality.
 * @property {string} water_quality_text - The latest water quality information.
 */

/**
 * Takes the latest data JSON and updates the HTML
 *
 * @param {WaterQualityData} data
 */
export function parseWaterQuality(data) {
  removeElements(".water-quality-loader");
  removeHidden(`current-water-quality`);
  setText(
    "current-water-quality-title",
    data.water_quality === "Closed for season"
      ? "-----"
      : data.water_quality.toUpperCase(),
  );

  setText(`current-water-quality-status-text`, data.water_quality_text);

  if (data.water_quality === "Advisory") {
    setStyleAttribute("current-water-quality-title", "color: #facc15;");
  }

  if (data.water_quality === "Closed") {
    setStyleAttribute("current-water-quality-title", "color: #ef4444;");
  }
}
