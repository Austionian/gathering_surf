import {
  parseWaterQuality,
  parseRealtime,
  parseForecast,
} from "./parsers/index";
import { forecastFailed } from "./fallback";
import { nonNull, wait } from "./utilities";

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
  for (const mutation of mutationList) {
    if (mutation.target instanceof HTMLElement) {
      if (mutation.target.id === "realtime-data") {
        parseRealtime(JSON.parse(mutation.target.innerText));
      }
      if (mutation.target.id === "water-quality-data") {
        parseWaterQuality(JSON.parse(mutation.target.innerText));
      }
      if (mutation.target.id === "forecast-data") {
        await parseForecastData(mutation);
      }
    }
  }
};

/**
 * JSON.parse sometimes executes before all the data has gotten to the DOM.
 *
 * @param {MutationRecord} mutation
 */
async function parseForecastData(mutation) {
  try {
    parseForecast(JSON.parse("jlkjk"));
  } catch {
    await wait(500);
    try {
      parseForecast(JSON.parse(mutation.target.innerText));
    } catch (e) {
      forecastFailed(e);
    }
  }
}

// Create an observer instance linked to the callback function
const observer = new MutationObserver(observerCallback);

// Start observing the target node for configured mutations
observer.observe(targetNode, config);
