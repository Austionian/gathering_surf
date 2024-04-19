// Select the node that will be observed for mutations
const targetNode = document.querySelector("body");

// Options for the observer (which mutations to observe)
const config = { attributes: true, childList: true, subtree: true };

// Callback function to execute when mutations are observed
const callback = (mutationList, _observer) => {
  for (const mutation of mutationList) {
    if (mutation.target.id === "latest-data") {
      parseLatestData(JSON.parse(mutation.target.innerText));
    }
  }
};

// Create an observer instance linked to the callback function
const observer = new MutationObserver(callback);

// Start observing the target node for configured mutations
observer.observe(targetNode, config);

const qualityMap = {
  "#0bd674": "Good",
  "#ffcd1e": "Fair to Good",
  "#ff9500": "Poor",
  "#f4496d": "Very Poor",
};

/**
 *   @typeof {Object} LatestData
 *   @property {string} quality_color - The hexcode of the quality.
 *   @property {string} quality_text - The computed text of the quality.
 *   @property {string} water_temp - The latest water temperature.
 *   @property {number} wind_direction - The current wind direction.
 *   @property {string} wind_speed - The current wind speed.
 *   @property {string} gusts - The current wind gust.
 */

/**
 * Takes the latest data JSON and updates the HTML
 *
 * @param {LatestData} data
 */
function parseLatestData(data) {
  document
    .getElementById("wave-quality")
    .setAttribute("style", `background-color: ${data.quality_color};`);
  document.getElementById("wave-quality-text").innerText = data.quality_text;
  document
    .getElementById("wave-quality-text")
    .setAttribute("style", `color: ${data.quality_color}`);
  document.getElementById("current-water-temp").innerText = data.water_temp;
  document.getElementById("current-air-temp").innerText = data.air_temp;
  document.getElementById("wind").innerText = getWindData(data);
  document.getElementById("as-of").innerText = `Live as of ${data.as_of}`;
  document
    .getElementById("wind-icon")
    .setAttribute(
      "style",
      `transform: rotate(${data.wind_direction + 180}deg);`,
    );

  document
    .querySelector("#legend")
    .setAttribute("style", `background-color: ${data.quality_color};`);

  document.querySelectorAll(".latest-loader").forEach((e) => e.remove());
  document.getElementById("wind-icon-container").classList.remove("hidden");
  document.getElementById("wave-icon-container").classList.remove("hidden");
  document.getElementById("as-of-container").classList.remove("animate-pulse");
  document.getElementById("wave-quality").classList.remove("hidden");
}

const getWindData = (data) =>
  data.wind_speed == data.gusts
    ? data.wind_speed
    : `${data.wind_speed}-${data.gusts}`;
