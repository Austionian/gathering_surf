import {
  removeHidden,
  removeElements,
  setText,
  setStyleAttribute,
} from "../utilities";

type WaterQualityData = {
  water_quality: "Open" | "Closed" | "Advisory" | "Closed for season";
  water_quality_text: string;
};

/**
 * Takes the latest data JSON and updates the HTML
 */
export function parseWaterQuality(data: WaterQualityData) {
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
