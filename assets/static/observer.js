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
    if (mutation.target.id === "forecast-data") {
      setTimeout(() => {
        parseForecastData(JSON.parse(mutation.target.innerText));
      }, 100);
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
 * @typeof {Object} LatestData
 * @property {string} quality_color - The hexcode of the quality.
 * @property {string} quality_text - The computed text of the quality.
 * @property {string} water_temp - The latest water temperature.
 * @property {number} wind_direction - The current wind direction.
 * @property {string} wind_speed - The current wind speed.
 * @property {string} gusts - The current wind gust.
 * @property {?string} wave_height
 * @property {?string} wave_direction
 * @property {?string} wave_period
 */

/**
 * Takes the latest data JSON and updates the HTML
 *
 * @param {LatestData} data
 */
function parseLatestData(data) {
  if (data.wave_height) {
    document.getElementById("current-wave-height").innerText = data.wave_height;
    document
      .getElementById("wave-icon")
      .setAttribute("style", `transform: rotate(${data.wave_direction}deg);`);

    document.querySelectorAll(".wavey").forEach((e) => e.remove());
  }

  if (data.wave_period) {
    document.getElementById("current-wave-period").innerText = data.wave_period;
    document.getElementById("wavey-period-loader").remove();
  }

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

  // -- Init wave legend --
  const wave_height_container = document.getElementById("current-wave-height");
  if (wave_height_container.innerText === "") {
    wave_height_container.innerText = data.current_wave_height;
    document
      .getElementById("wave-icon")
      .setAttribute(
        "style",
        `transform: rotate(${data.current_wave_direction}deg);`,
      );

    document.querySelectorAll(".wavey").forEach((e) => e.remove());
  }

  // The period can be undefined separate from the wave height
  const wave_period_container = document.getElementById("current-wave-period");
  if (wave_period_container.innerText === "") {
    console.log(wave_period_container.innerText === "");
    wave_period_container.innerText = data.current_wave_period;
    document.getElementById("wavey-period-loader")?.remove();
  }

  document.getElementById("legend-label").innerText = wave_height_labels[0];
  document.getElementById("legend-quality").innerText =
    qualityMap[qualities[0]];
  document.getElementById("legend-wave-height").innerText = wave_heights[0];
  document.getElementById("legend-wind-speed").innerText = wind_speeds[0];
  document
    .getElementById("legend-wind-icon")
    .setAttribute("style", `transform: rotate(${wind_directions[0]}deg);`);
  document.getElementById("legend-wave-period").innerText = wave_period[0];
  document.getElementById("legend-wind-gust").innerText = wind_gusts[0];
  document.getElementById("forecast-as-of").innerText =
    `Last updated at ${data.forecast_as_of}`;

  document.querySelectorAll(".loader").forEach((e) => e.remove());
  document.getElementById("forecast").classList.remove("hidden");
  document.getElementById("wave-quality").classList.remove("hidden");
  document.getElementById("legend-container").classList.remove("hidden");

  // -- Init temperature legend --
  document
    .getElementById("temperature-legend-container")
    .classList.remove("hidden");
  document.getElementById("temperature-legend-label").innerText =
    wave_height_labels[0];
  document.getElementById("temperature-legend-temperature").innerText =
    temperature[0];
  document.getElementById("temperature-legend-dewpoint").innerText =
    dewpoint[0];

  // -- Init precipitation legend --
  document
    .getElementById("precipitation-legend-container")
    .classList.remove("hidden");
  document.getElementById("precipitation-legend-label").innerText =
    wave_height_labels[0];
  document.getElementById("precipitation-legend-precipitation").innerText =
    probability_of_precipitation[0];
  document.getElementById("precipitation-legend-thunder").innerText =
    probability_of_thunder[0];
  document.getElementById("precipitation-legend-cloud-cover").innerText =
    cloud_cover[0];

  // The default legend background is the same as the latest wave quality shown
  // in the header. If that script doesn't happen fallback to the first value in
  // the qualities array.
  const legend = document.querySelector("#legend");
  if (legend.style.length < 1) {
    legend.setAttribute("style", `background-color: ${qualities[0]};`);
  }

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
    document
      .getElementById("legend")
      .setAttribute("style", `background-color: ${color}`);
    document.getElementById("legend-label").innerText = wave_height_labels[x];
    document.getElementById("legend-quality").innerText = qualityMap[color];
    document.getElementById("legend-wave-height").innerText = wave_heights[x];
    document.getElementById("legend-wind-speed").innerText = wind_speeds[x];
    document
      .getElementById("legend-wind-icon")
      .setAttribute("style", `transform: rotate(${wind_directions[x]}deg);`);
    document.getElementById("legend-wave-period").innerText = wave_period[x];
    document.getElementById("legend-wind-gust").innerText = wind_gusts[x];

    // Update temperature legend
    document.getElementById("temperature-legend-label").innerText =
      wave_height_labels[x];
    document.getElementById("temperature-legend-temperature").innerText =
      temperature[x];
    document.getElementById("temperature-legend-dewpoint").innerText =
      dewpoint[x];

    // Update precipitation legend
    document.getElementById("precipitation-legend-label").innerText =
      wave_height_labels[x];
    document.getElementById("precipitation-legend-precipitation").innerText =
      probability_of_precipitation[x];
    document.getElementById("precipitation-legend-thunder").innerText =
      probability_of_thunder[x];
    document.getElementById("precipitation-legend-cloud-cover").innerText =
      cloud_cover[x];
  };

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
            callback: (_value, i, _ticks) =>
              i % 48 === 0 ? wave_height_labels[i] : null,
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

  const ctxTemp = document.getElementById("temperature-forecast");

  const temperatureForecast = new Chart(ctxTemp, {
    type: "line",
    plugins: [plugin],
    data: {
      labels: wave_height_labels,
      datasets: [
        {
          label: "temperature (f)",
          data: temperature,
          pointStyle: false,
          fill: false,
          segment: {
            backgroundColor: "pink",
            borderColor: "pink",
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
            callback: (_value, i, _ticks) =>
              i % 48 === 0 ? wave_height_labels[i] : null,
          },
        },
        y: {
          beginAtZero: true,
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

  const precipitationCanvas = document.getElementById("precipitation-forecast");

  const precipitationForecast = new Chart(precipitationCanvas, {
    type: "line",
    plugins: [plugin],
    data: {
      labels: wave_height_labels,
      datasets: [
        {
          label: "%",
          data: probability_of_precipitation,
          pointStyle: false,
          fill: false,
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
            callback: (_value, i, _ticks) =>
              i % 48 === 0 ? wave_height_labels[i] : null,
          },
        },
        y: {
          beginAtZero: true,
          max: 100,
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
}
