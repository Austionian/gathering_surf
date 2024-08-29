import {
  parseWaterQuality,
  parseRealtime,
  parseForecast,
} from "./parsers/index";
import { nonNull } from "./utilities";

// Select the node that will be observed for mutations
const targetNode = nonNull(document.querySelector("body"));

// Options for the observer (which mutations to observe)
const config = { attributes: true, childList: true, subtree: true };

/**
 * Callback function to execute when mutations are observed
 *
 * @param {MutationRecord[]} mutationList
 */
const observerCallback = (mutationList) => {
  for (const mutation of mutationList) {
    if (mutation.target instanceof HTMLElement) {
      if (mutation.target.id === "latest-data") {
        parseRealtime(JSON.parse(mutation.target.innerText));
      }
      if (mutation.target.id === "water-quality-data") {
        parseWaterQuality(JSON.parse(mutation.target.innerText));
      }
      if (mutation.target.id === "forecast-data") {
        try {
          const data = JSON.parse(mutation.target.innerText);
          parseForecast(data);
        } catch {
          console.log("failed to parse forecast, trying again.");
          setTimeout(() => {
            if (mutation.target instanceof HTMLElement) {
              const data = JSON.parse(mutation.target.innerText);
              parseForecast(data);
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
