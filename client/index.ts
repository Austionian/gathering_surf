import {
  parseWaterQuality,
  parseRealtime,
  parseForecast,
  // @ts-ignore
} from "./parsers/index.js";
import { forecastFailed } from "./fallback";

// Select the node that will be observed for mutations
const targetNode = document.querySelector("body") as HTMLElement;

// Options for the observer (which mutations to observe)
const config = { attributes: true, childList: true, subtree: true };

/**
 * Callback function to execute when mutations are observed
 */
const observerCallback = (mutationList: MutationRecord[]) => {
  for (const mutation of mutationList) {
    if (mutation.target instanceof HTMLElement) {
      if (mutation.target.id === "realtime-data") {
        parseRealtime(JSON.parse(mutation.target.innerText));
      }
      if (mutation.target.id === "water-quality-data") {
        parseWaterQuality(JSON.parse(mutation.target.innerText));
      }
      if (mutation.target.id === "forecast-data") {
        try {
          parseForecast(JSON.parse(mutation.target.innerText));
        } catch {
          console.log("failed to parse forecast, trying again.");
          setTimeout(() => {
            if (mutation.target instanceof HTMLElement) {
              try {
                parseForecast(JSON.parse(mutation.target.innerText));
              } catch (e) {
                forecastFailed(e as Error);
              }
            }
          }, 100);
        }
      }
    }
  }
};

// Create an observer instance linked to the callback function
const observer = new MutationObserver(observerCallback);

// Start observing the target node for configured mutations
observer.observe(targetNode, config);