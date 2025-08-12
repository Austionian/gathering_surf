import {
  parseWaterQuality,
  parseRealtime,
  parseForecast,
} from "./parsers/index";
import { forecastFailed } from "./fallback";
import { setTimeoutError, nonNull } from "./utilities";

// Select the node that will be observed for mutations
const targetNode = nonNull(document.querySelector("body"));

// Options for the observer (which mutations to observe)
const config = { attributes: true, childList: true, subtree: true };

/**
 * Callback function to execute when mutations are observed
 *
 * @param {MutationRecord[]} mutationList
 */
const observerCallback = async (mutationList) => {
  // Set a timeout error for each section, without this user could be
  // presented with an endless loading state.
  ["latest", "forecast", "water-quality"].forEach((section) => {
    setTimeoutError(section);
  });

  for (const mutation of mutationList) {
    if (mutation.target instanceof HTMLElement) {
      if (mutation.target.id === "realtime-data") {
        parseRealtime(JSON.parse(mutation.target.innerText));
      }
      if (mutation.target.id === "water-quality-data") {
        parseWaterQuality(JSON.parse(mutation.target.innerText));
      }
      for (let i = 0; i < mutation.addedNodes.length; i++) {
        if (mutation.addedNodes[i].id === "forecast-complete") {
          parseForecastData();
        }
      }
    }
  }
};

/**
 * JSON.parse sometimes executes before all the data has gotten to the DOM.
 */
function parseForecastData() {
  const forecastData = document.getElementById("forecast-data");
  try {
    parseForecast(JSON.parse(forecastData.innerText));
  } catch (e) {
    forecastFailed(e);
  }
}

// Create an observer instance linked to the callback function
const observer = new MutationObserver(observerCallback);

// Start observing the target node for configured mutations
observer.observe(targetNode, config);
