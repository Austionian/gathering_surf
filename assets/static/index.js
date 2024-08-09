// Select the node that will be observed for mutations
const targetNode = /** @type {HTMLElement} */ (document.querySelector("body"));

// Options for the observer (which mutations to observe)
const config = { attributes: true, childList: true, subtree: true };

/**
 * Callback function to execute when mutations are observed
 *
 * @param {MutationRecord[]} mutationList
 */
const observerCallback = (mutationList) => {
  for (const mutation of mutationList) {
    if (mutation.target.id === "latest-data") {
      parseLatestData(JSON.parse(mutation.target.innerText));
    }
    if (mutation.target.id === "water-quality-data") {
      parseWaterQualityData(JSON.parse(mutation.target.innerText));
    }
    if (mutation.target.id === "forecast-data") {
      try {
        const data = JSON.parse(mutation.target.innerText);
        parseForecastData(data);
      } catch {
        console.log("failed to parse forecast, trying again.");
        setTimeout(() => {
          const data = JSON.parse(mutation.target.innerText);
          parseForecastData(data);
        }, 100);
      }
    }
  }
};

// Create an observer instance linked to the callback function
const observer = new MutationObserver(observerCallback);

// Start observing the target node for configured mutations
observer.observe(targetNode, config);

const FLAT_COLOR = "#a8a29e";

const qualityMap = {
  "#0bd674": "Good",
  "#ffcd1e": "Fair to Good",
  "#ff9500": "Poor",
  "#f4496d": "Very Poor",
  "#a8a29e": "Flat",
};

/**
 * Convenient non-null assertion helper function allows asserting that something is not
 * null or undefined without having to write a JSDoc type cast that has to
 * explicitly know the non-null type (which is error prone).
 *
 * @template {any} T
 * @param {T} item
 */
function NonNull(item) {
  if (item === null || item === undefined) throw "item is null or undefined";
  return item;
}

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
function parseWaterQualityData(data) {
  NonNull(
    document.getElementById(
      `current-water-quality-${data.water_quality.toLowerCase()}`,
    ),
  )?.classList.remove("hidden");
  NonNull(
    document.getElementById(
      `current-water-quality-${data.water_quality.toLowerCase()}-status-text`,
    ),
  ).innerText = data.water_quality_text;

  document.querySelectorAll(".water-quality-loader").forEach((e) => e.remove());
}

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
 */

/**
 * Takes the latest data JSON and updates the HTML
 *
 * @param {LatestData} data
 */
function parseLatestData(data) {
  if (data.wave_height) {
    NonNull(document.getElementById("current-wave-height")).innerText =
      data.wave_height;
    document
      .getElementById("wave-icon")
      ?.setAttribute("style", `transform: rotate(${data.wave_direction}deg);`);

    document.querySelectorAll(".wavey").forEach((e) => e.remove());
  }

  if (data.wave_period) {
    NonNull(document.getElementById("current-wave-period")).innerText =
      data.wave_period;
    document.getElementById("wavey-period-loader")?.remove();
  }

  // Sometimes the forecast will have loaded before the realtime data.
  // Most of the time realtime quality should supersede forecast.
  if (
    // If it hasn't been set yet, always set it.
    NonNull(document.getElementById("wave-quality-text")).innerText === "" ||
    // Otherwise if realtime has wave height, this always supersedes forecast estimates
    // and 'Flat' condition will be handled appropriately
    data.wave_height ||
    // If the forecast data has loaded and the computed wave height is greater than
    // or equal to one, then load the realtime computed quality. If this isn't here
    // forecasted 'Flat' conditions wouldn't be handled correctly.
    parseFloat(
      NonNull(document.getElementById("current-wave-height")).innerText ?? "0",
    ) >= 1
  ) {
    NonNull(document.getElementById("wave-quality")).setAttribute(
      "style",
      `background-color: ${data.quality_color};`,
    );
    NonNull(document.getElementById("wave-quality-text")).innerText =
      data.quality_text;
    NonNull(document.getElementById("wave-quality-text")).setAttribute(
      "style",
      `color: ${data.quality_color}`,
    );
  }

  NonNull(document.getElementById("current-water-temp")).innerText =
    data.water_temp;
  NonNull(document.getElementById("current-air-temp")).innerText =
    data.air_temp;

  NonNull(document.getElementById("wind")).innerText = getWindData(data);
  NonNull(document.getElementById("as-of")).innerText = `As of ${data.as_of}`;
  document
    .getElementById("wind-icon")
    ?.setAttribute(
      "style",
      `transform: rotate(${data.wind_direction + 180}deg);`,
    );

  // document
  //   .getElementById("legend")
  //   ?.setAttribute("style", `background-color: ${data.quality_color};`);
  //
  document.querySelectorAll(".latest-loader").forEach((e) => e.remove());
  document.getElementById("wind-icon-container")?.classList.remove("hidden");
  document.getElementById("wave-icon-container")?.classList.remove("hidden");
  document.getElementById("as-of-container")?.classList.remove("animate-pulse");
  document.getElementById("wave-quality")?.classList.remove("hidden");
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

let qualities;
let wave_height_labels;
let wave_heights;
let wind_speeds;
let wind_directions;
let wind_gusts;
let wave_period;
let graph_max;
let temperature;
let dewpoint;
let cloud_cover;
let probability_of_precipitation;
let probability_of_thunder;

/**
 * @typedef {Object} ForecastData
 * @property {string[]} graph_max
 * @property {string[]} wave_height_data
 * @property {string} current_wave_height
 * @property {string} current_wave_direction
 * @property {string} current_wave_period
 * @property {string[]} wind_speed_data
 * @property {string[]} wind_direction_data
 * @property {string[]} wind_gust_data
 * @property {string[]} wave_period_data
 * @property {string[]} wave_height_labels
 * @property {string[]} forecast_as_of
 * @property {string[]} temperature
 * @property {string[]} probability_of_precipitation
 * @property {string[]} dewpoint
 * @property {string[]} cloud_cover
 * @property {string[]} probability_of_thunder
 * @property {string[]} qualities
 * @property {string} starting_at
 */

/**
 * Takes the forecast data JSON and updates the HTML
 *
 * @param {ForecastData} data
 */
function parseForecastData(data) {
  qualities = data.qualities;
  wave_height_labels = data.wave_height_labels;
  wave_heights = data.wave_height_data;
  wind_speeds = data.wind_speed_data;
  wind_directions = data.wind_direction_data;
  wind_gusts = data.wind_gust_data;
  wave_period = data.wave_period_data;
  graph_max = data.graph_max;
  temperature = data.temperature;
  dewpoint = data.dewpoint;
  cloud_cover = data.cloud_cover;
  probability_of_precipitation = data.probability_of_precipitation;
  probability_of_thunder = data.probability_of_thunder;

  let startingAt =
    new Date().getHours() - new Date(data.starting_at).getHours();

  // If the forecast starting_at time hasn't been updated in a while, it will show
  // a PM time the follow morning.
  startingAt = startingAt < 0 ? new Date().getHours() : startingAt;

  // -- Init wave legend --
  const wave_height_container = document.getElementById("current-wave-height");
  if (wave_height_container?.innerText === "") {
    wave_height_container.innerText = data.current_wave_height;

    // if the current wave height data is under a foot, update the
    // quality to be Flat
    if (data.current_wave_height == "0") {
      NonNull(document.getElementById("wave-quality"))?.setAttribute(
        "style",
        `background-color: ${FLAT_COLOR}`,
      );

      const text = NonNull(document.getElementById("wave-quality-text"));
      text.innerText = qualityMap[FLAT_COLOR];
      text.setAttribute("style", `color: ${FLAT_COLOR}`);
    }

    document
      .getElementById("wave-icon")
      ?.setAttribute(
        "style",
        `transform: rotate(${data.current_wave_direction}deg);`,
      );

    document.querySelectorAll(".wavey").forEach((e) => e.remove());
  }

  // The period can be undefined separate from the wave height
  const wave_period_container = document.getElementById("current-wave-period");
  if (wave_period_container?.innerText === "") {
    wave_period_container.innerText = data.current_wave_period;
    document.getElementById("wavey-period-loader")?.remove();
  }

  NonNull(document.getElementById("legend-label")).innerText =
    wave_height_labels[startingAt];
  NonNull(document.getElementById("legend-quality")).innerText =
    qualityMap[qualities[startingAt]];
  NonNull(document.getElementById("legend-wave-height")).innerText =
    wave_heights[startingAt];
  NonNull(document.getElementById("legend-wind-speed")).innerText =
    wind_speeds[startingAt];
  NonNull(document.getElementById("legend-wind-icon")).setAttribute(
    "style",
    `transform: rotate(${wind_directions[startingAt]}deg);`,
  );
  NonNull(document.getElementById("legend-wave-period")).innerText =
    wave_period[startingAt];
  NonNull(document.getElementById("legend-wind-gust")).innerText =
    wind_gusts[startingAt];
  NonNull(document.getElementById("forecast-as-of")).innerText =
    `Last updated at ${data.forecast_as_of}`;

  document.querySelectorAll(".loader").forEach((e) => e.remove());
  document.getElementById("forecast")?.classList.remove("hidden");
  document.getElementById("wave-quality")?.classList.remove("hidden");
  document.getElementById("legend-container")?.classList.remove("hidden");
  document
    .getElementById("forecast-as-of-container")
    ?.classList.remove("animate-pulse");

  // -- Init temperature legend --
  document
    .getElementById("temperature-legend-container")
    ?.classList.remove("hidden");
  NonNull(document.getElementById("temperature-legend-label")).innerText =
    wave_height_labels[startingAt];
  NonNull(document.getElementById("temperature-legend-temperature")).innerText =
    temperature[startingAt];
  NonNull(document.getElementById("temperature-legend-dewpoint")).innerText =
    dewpoint[startingAt];

  // -- Init precipitation legend --
  document
    .getElementById("precipitation-legend-container")
    ?.classList.remove("hidden");
  NonNull(document.getElementById("precipitation-legend-label")).innerText =
    wave_height_labels[startingAt];
  NonNull(
    document.getElementById("precipitation-legend-precipitation"),
  ).innerText = probability_of_precipitation[startingAt];
  NonNull(document.getElementById("precipitation-legend-thunder")).innerText =
    probability_of_thunder[startingAt];
  NonNull(
    document.getElementById("precipitation-legend-cloud-cover"),
  ).innerText = cloud_cover[startingAt];

  NonNull(document.getElementById("legend")).setAttribute(
    "style",
    `background-color: ${qualities[startingAt]};`,
  );

  const ctx = document.getElementById("forecast");

  const quality = (ctx) => qualities[ctx.p0.parsed.x];

  const plugin = {
    id: "vert",
    defaults: {
      width: 1,
      dash: [3, 3],
    },
    afterInit: (chart, _args, _opts) => {
      chart.corsair = {
        x: 0,
        y: 0,
      };
    },
    afterDraw: (chart, _args, opts) => {
      if (chart.tooltip?._active?.length) {
        let x = chart.tooltip._active[0].element.x;
        let yAxis = chart.scales.y;
        let ctx = chart.ctx;
        ctx.save();
        ctx.beginPath();
        ctx.moveTo(x, yAxis.top);
        ctx.lineTo(x, yAxis.bottom);
        ctx.lineWidth = 1;
        ctx.setLineDash(opts.dash);
        ctx.strokeStyle = "#fff";
        ctx.stroke();
        ctx.restore();
      }
    },
  };

  const onHover = (e, _, chart) => {
    const canvasPosition = Chart.helpers.getRelativePosition(e, chart);
    // Substitute the appropriate scale IDs
    const x_value = chart.scales.x.getValueForPixel(canvasPosition.x);
    const x =
      x_value && x_value > 0
        ? x_value >= wave_heights.length
          ? wave_heights.length - 1
          : x_value
        : 0;
    const color = qualities[x];

    // Update wave legend
    NonNull(document.getElementById("legend")).setAttribute(
      "style",
      `background-color: ${color}`,
    );
    NonNull(document.getElementById("legend-label")).innerText =
      wave_height_labels[x];
    NonNull(document.getElementById("legend-quality")).innerText =
      qualityMap[color];
    NonNull(document.getElementById("legend-wave-height")).innerText =
      wave_heights[x];
    NonNull(document.getElementById("legend-wind-speed")).innerText =
      wind_speeds[x];
    NonNull(document.getElementById("legend-wind-icon")).setAttribute(
      "style",
      `transform: rotate(${wind_directions[x]}deg);`,
    );
    NonNull(document.getElementById("legend-wave-period")).innerText =
      wave_period[x];
    NonNull(document.getElementById("legend-wind-gust")).innerText =
      wind_gusts[x];

    // Update temperature legend
    NonNull(document.getElementById("temperature-legend-label")).innerText =
      wave_height_labels[x];
    NonNull(
      document.getElementById("temperature-legend-temperature"),
    ).innerText = temperature[x];
    NonNull(document.getElementById("temperature-legend-dewpoint")).innerText =
      dewpoint[x];

    // Update precipitation legend
    NonNull(document.getElementById("precipitation-legend-label")).innerText =
      wave_height_labels[x];
    NonNull(
      document.getElementById("precipitation-legend-precipitation"),
    ).innerText = probability_of_precipitation[x];
    NonNull(document.getElementById("precipitation-legend-thunder")).innerText =
      probability_of_thunder[x];
    NonNull(
      document.getElementById("precipitation-legend-cloud-cover"),
    ).innerText = cloud_cover[x];
  };

  const xTicksCallback = (_value, i, _ticks) =>
    i % 48 === 0 ? wave_height_labels[i] : null;

  const waveForecast = new Chart(ctx, {
    type: "line",
    plugins: [plugin],
    data: {
      labels: wave_height_labels,
      datasets: [
        {
          label: "wave height (feet)",
          data: wave_heights,
          pointStyle: false,
          segment: {
            backgroundColor: (ctx) => quality(ctx) || "#4ade80",
            borderColor: (ctx) => quality(ctx) || "#4ade80",
          },
        },
      ],
    },
    options: {
      onHover,
      maintainAspectRatio: false,
      plugins: {
        legend: {
          display: false,
        },
        tooltip: {
          enabled: false,
        },
      },
      elements: {
        line: {
          tension: 0.4,
        },
      },
      responsive: true,
      interaction: {
        intersect: false,
        axis: "x",
      },
      scales: {
        x: {
          ticks: {
            callback: xTicksCallback,
          },
        },
        y: {
          beginAtZero: true,
          max: graph_max,
          ticks: {
            callback: function (value, _index, _ticks) {
              if (value % 2 !== 0) {
                return "";
              }
              return value;
            },
            font: {
              size: 18,
              weight: "bold",
            },
          },
        },
      },
    },
  });

  const getConfig = (data, color, label, max) => ({
    type: "line",
    plugins: [plugin],
    data: {
      labels: wave_height_labels,
      datasets: [
        {
          label,
          data,
          pointStyle: false,
          segment: color
            ? {
                backgroundColor: color,
                borderColor: color,
              }
            : {},
        },
      ],
    },
    options: {
      onHover,
      maintainAspectRatio: false,
      plugins: {
        legend: {
          display: false,
        },
        tooltip: {
          enabled: false,
        },
      },
      elements: {
        line: {
          tension: 0.4,
        },
      },
      responsive: true,
      interaction: {
        intersect: false,
        axis: "x",
      },
      scales: {
        x: {
          ticks: {
            callback: xTicksCallback,
          },
        },
        y: {
          beginAtZero: true,
          max,
          ticks: {
            font: {
              size: 18,
              weight: "bold",
            },
          },
        },
      },
    },
  });

  const temperatureCanvas = document.getElementById("temperature-forecast");
  const temperatureForecast = new Chart(
    temperatureCanvas,
    getConfig(temperature, "pink", "F", null),
  );

  const precipitationCanvas = document.getElementById("precipitation-forecast");
  const precipitationForecast = new Chart(
    precipitationCanvas,
    getConfig(probability_of_precipitation, null, "%", 100),
  );

  function waveHover(e) {
    const points = waveForecast.getElementsAtEventForMode(
      e,
      "nearest",
      {
        axis: "x",
        intersect: false,
      },
      true,
    );
    if (points[0]) {
      const datasetIndex = points[0].datasetIndex;
      const index = points[0].index;

      temperatureForecast.tooltip.setActiveElements([
        {
          datasetIndex,
          index,
        },
      ]);
      temperatureForecast.setActiveElements([
        {
          datasetIndex,
          index,
        },
      ]);
      temperatureForecast.update();

      precipitationForecast.tooltip.setActiveElements([
        {
          datasetIndex,
          index,
        },
      ]);

      precipitationForecast.setActiveElements([
        {
          datasetIndex,
          index,
        },
      ]);
      precipitationForecast.update();
    } else {
      temperatureForecast.tooltip.setActiveElements([], { x: 0, y: 0 });
      temperatureForecast.setActiveElements([], { x: 0, y: 0 });
      temperatureForecast.update();

      precipitationForecast.tooltip.setActiveElements([], { x: 0, y: 0 });
      precipitationForecast.setActiveElements([], { x: 0, y: 0 });
      precipitationForecast.update();
    }
  }

  function tempHover(e) {
    const points = temperatureForecast.getElementsAtEventForMode(
      e,
      "nearest",
      {
        axis: "x",
        intersect: false,
      },
      true,
    );
    if (points[0]) {
      const datasetIndex = points[0].datasetIndex;
      const index = points[0].index;

      waveForecast.tooltip.setActiveElements([
        {
          datasetIndex,
          index,
        },
      ]);
      waveForecast.setActiveElements([
        {
          datasetIndex,
          index,
        },
      ]);
      waveForecast.update();

      precipitationForecast.tooltip.setActiveElements([
        {
          datasetIndex,
          index,
        },
      ]);

      precipitationForecast.setActiveElements([
        {
          datasetIndex,
          index,
        },
      ]);
      precipitationForecast.update();
    } else {
      waveForecast.tooltip.setActiveElements([], { x: 0, y: 0 });
      waveForecast.setActiveElements([], { x: 0, y: 0 });
      waveForecast.update();

      precipitationForecast.tooltip.setActiveElements([], { x: 0, y: 0 });
      precipitationForecast.setActiveElements([], { x: 0, y: 0 });
      precipitationForecast.update();
    }
  }

  function precipitationHover(e) {
    const points = precipitationForecast.getElementsAtEventForMode(
      e,
      "nearest",
      {
        axis: "x",
        intersect: false,
      },
      true,
    );
    if (points[0]) {
      const datasetIndex = points[0].datasetIndex;
      const index = points[0].index;

      temperatureForecast.tooltip.setActiveElements([
        {
          datasetIndex,
          index,
        },
      ]);
      temperatureForecast.setActiveElements([
        {
          datasetIndex,
          index,
        },
      ]);
      temperatureForecast.update();

      waveForecast.tooltip.setActiveElements([
        {
          datasetIndex,
          index,
        },
      ]);

      waveForecast.setActiveElements([
        {
          datasetIndex,
          index,
        },
      ]);
      waveForecast.update();
    } else {
      temperatureForecast.tooltip.setActiveElements([], { x: 0, y: 0 });
      temperatureForecast.setActiveElements([], { x: 0, y: 0 });
      temperatureForecast.update();

      waveForecast.tooltip.setActiveElements([], { x: 0, y: 0 });
      waveForecast.setActiveElements([], { x: 0, y: 0 });
      waveForecast.update();
    }
  }

  waveForecast.canvas.onmousemove = waveHover;
  temperatureForecast.canvas.onmousemove = tempHover;
  precipitationForecast.canvas.onmousemove = precipitationHover;

  waveForecast.canvas.ontouchmove = waveHover;
  temperatureForecast.canvas.ontouchmove = tempHover;
  precipitationForecast.canvas.ontouchmove = precipitationHover;
}
