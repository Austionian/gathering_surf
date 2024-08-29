import { removeHidden, removeElements, setText } from "../utilities";

/**
 * @typedef {Object} WaterQualityData
 * @property {string} water_quality - The latest water quality.
 * @property {string} water_quality_text - The latest water quality information.
 */

/**
 * Takes the latest data JSON and updates the HTML
 *
 * @param {WaterQualityData} data
 */
export function parseWaterQuality(data) {
  removeHidden(`current-water-quality-${data.water_quality.toLowerCase()}`);
  setText(
    `current-water-quality-${data.water_quality.toLowerCase()}-status-text`,
    data.water_quality_text,
  );

  removeElements(".water-quality-loader");
}
