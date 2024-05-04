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

function parseForecastData(data) {
  qualities = data.qualities;
  wave_height_labels = data.wave_height_labels;
  wave_heights = data.wave_height_data;
  wind_speeds = data.wind_speed_data;
  wind_directions = data.wind_direction_data;
  wind_gusts = data.wind_gust_data;
  wave_period = data.wave_period_data;
  graph_max = data.graph_max;

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

  document.querySelector("#legend-label").innerText = wave_height_labels[0];
  document.querySelector("#legend-quality").innerText =
    qualityMap[qualities[0]];
  document.querySelector("#legend-wave-height").innerText = wave_heights[0];
  document.querySelector("#legend-wind-speed").innerText = wind_speeds[0];
  document
    .getElementById("legend-wind-icon")
    .setAttribute("style", `transform: rotate(${wind_directions[0]}deg);`);
  document.querySelector("#legend-wave-period").innerText = wave_period[0];
  document.querySelector("#legend-wind-gust").innerText = wind_gusts[0];
  document.getElementById("forecast-as-of").innerText =
    `Last updated at ${data.forecast_as_of}`;

  document.querySelectorAll(".loader").forEach((e) => e.remove());
  document.getElementById("forecast").classList.remove("hidden");
  document.getElementById("wave-quality").classList.remove("hidden");
  document.getElementById("legend-container").classList.remove("hidden");

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
    afterEvent: (chart, args) => {
      const { inChartArea } = args;
      const { x, y } = args.event;

      chart.corsair = { x, y, draw: inChartArea };
      chart.draw();
    },
    beforeDatasetsDraw: (chart, _args, opts) => {
      const { ctx } = chart;
      const { top, bottom } = chart.chartArea;
      const { x, draw } = chart.corsair;
      if (!draw) return;

      ctx.save();

      ctx.beginPath();
      ctx.lineWidth = opts.width;
      ctx.strokeStyle = opts.color;
      ctx.setLineDash(opts.dash);
      ctx.moveTo(x, bottom);
      ctx.lineTo(x, top);
      ctx.stroke();

      ctx.restore();
    },
  };

  new Chart(ctx, {
    type: "line",
    plugins: [plugin],
    data: {
      labels: wave_height_labels,
      datasets: [
        {
          label: "wave height (feet)",
          data: wave_heights,
          pointStyle: false,
          fill: false,
          segment: {
            backgroundColor: (ctx) => quality(ctx) || "#4ade80",
            borderColor: (ctx) => quality(ctx) || "#4ade80",
          },
        },
      ],
    },
    options: {
      onHover: (e, _, chart) => {
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
        document
          .querySelector("#legend")
          .setAttribute("style", `background-color: ${color}`);
        document.querySelector("#legend-label").innerText =
          wave_height_labels[x];
        document.querySelector("#legend-quality").innerText = qualityMap[color];
        document.querySelector("#legend-wave-height").innerText =
          wave_heights[x];
        document.querySelector("#legend-wind-speed").innerText = wind_speeds[x];
        document
          .getElementById("legend-wind-icon")
          .setAttribute(
            "style",
            `transform: rotate(${wind_directions[x]}deg);`,
          );
        document.querySelector("#legend-wave-period").innerText =
          wave_period[x];
        document.querySelector("#legend-wind-gust").innerText = wind_gusts[x];
      },
      maintainAspectRatio: false,
      plugins: {
        legend: {
          display: false,
        },
        vert: {
          color: "white",
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
            display: false,
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
}
