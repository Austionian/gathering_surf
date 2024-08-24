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

/* Utility Functions */

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
 * Sets an html element's innerText
 *
 * @param {string} id
 * @param {string} text
 */
function setText(id, text) {
  NonNull(document.getElementById(id)).innerText = text;
}

/**
 * Sets an html element's attribute
 *
 * @param {string} id
 * @param {string} attribute
 * @param {string} value
 */
function setAttribute(id, attribute, value) {
  document.getElementById(id)?.setAttribute(attribute, value);
}

/**
 * Sets an html element's style attribute
 *
 * @param {string} id
 * @param {string} value
 */
function setStyleAttribute(id, value) {
  setAttribute(id, "style", value);
}

/**
 * Removes all elements with the given class name.
 *
 * @param {string} className
 */
function removeElements(className) {
  document.querySelectorAll(className).forEach((e) => e.remove());
}

/**
 * Removes an element with the given id.
 *
 * @param {string} id
 */
function removeElement(id) {
  document.getElementById(id)?.remove();
}

/**
 *
 * Removes an element's style from its classList
 *
 * @param {string} id
 * @param {string} style
 */
function removeStyle(id, style) {
  document.getElementById(id)?.classList.remove(style);
}

/**
 * Removes the hidden classname from an element's classlist
 *
 * @param {string} id
 */
function removeHidden(id) {
  removeStyle(id, "hidden");
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
  removeHidden(`current-water-quality-${data.water_quality.toLowerCase()}`);
  setText(
    `current-water-quality-${data.water_quality.toLowerCase()}-status-text`,
    data.water_quality_text,
  );

  removeElements(".water-quality-loader");
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
    setText("current-wave-height", data.wave_height);
    setStyleAttribute(
      "wave-icon",
      `transform: rotate(${data.wave_direction}deg);`,
    );
    removeElements(".wavey");
  }

  if (data.wave_period) {
    setText("current-wave-period", data.wave_period);
    removeElement("wavey-period-loader");
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
    setStyleAttribute(
      "wave-quality",
      `background-color: ${data.quality_color};`,
    );
    setText("wave-quality-text", data.quality_text);
    setStyleAttribute("wave-quality-text", `color: ${data.quality_color}`);
    removeElements(".wave-quality-loader");
  }

  setText("current-water-temp", data.water_temp);
  setText("current-air-temp", data.air_temp);
  setText("current-air-temp-2", data.air_temp);

  setText("wind", getWindData(data));
  setText("as-of", `As of ${data.as_of}`);
  setStyleAttribute(
    "wind-icon",
    `transform: rotate(${data.wind_direction + 180}deg);`,
  );

  removeElements(".latest-loader");
  removeHidden("wind-icon-container");
  removeHidden("wave-icon-container");
  removeStyle("as-of-container", "animate-pulse");
  removeHidden("wave-quality");
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
  graph_max = data.graph_max;

  const prefillLength = new Date(data.starting_at).getHours();

  // Might be simpler to just align this to be the same day no matter what, then the
  // dayoffset below isn't needed and this logic just ensures starting_at is within the current day of the user
  if (prefillLength > 20) {
    let offset = 24 - prefillLength;
    wave_height_labels = data.wave_height_labels.slice(offset);

    qualities = data.qualities.slice(offset);
    wave_heights = data.wave_height_data.slice(offset);
    wind_speeds = data.wind_speed_data.slice(offset);
    wind_directions = data.wind_direction_data.slice(offset);
    wind_gusts = data.wind_gust_data.slice(offset);
    wave_period = data.wave_period_data.slice(offset);
    temperature = data.temperature.slice(offset);
    dewpoint = data.dewpoint.slice(offset);
    cloud_cover = data.cloud_cover.slice(offset);
    probability_of_precipitation =
      data.probability_of_precipitation.slice(offset);
    probability_of_thunder = data.probability_of_thunder.slice(offset);
  } else {
    const weekday = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const dayLabel = weekday[new Date(data.starting_at).getDay()];

    const prefillLabels = [`${dayLabel} 12 AM`];
    for (let i = 1; i < prefillLength; i++) {
      if (i < 10) {
        prefillLabels.push(`${dayLabel} 0${i} AM`);
      }
      if (i >= 10 && i < 12) {
        prefillLabels.push(`${dayLabel} ${i} AM`);
      }
      if (i === 12) {
        prefillLabels.push(`${dayLabel} ${i} PM`);
      }
      if (i > 12) {
        prefillLabels.push(`${dayLabel} 0${i - 12} PM`);
      }
    }

    wave_height_labels = prefillLabels.concat(data.wave_height_labels);

    qualities = new Array(prefillLength).fill("#a8a29e").concat(data.qualities);
    wave_heights = new Array(prefillLength)
      .fill(0)
      .concat(data.wave_height_data);
    wind_speeds = new Array(prefillLength).fill(0).concat(data.wind_speed_data);
    wind_directions = new Array(prefillLength)
      .fill(0)
      .concat(data.wind_direction_data);
    wind_gusts = new Array(prefillLength).fill(0).concat(data.wind_gust_data);
    wave_period = new Array(prefillLength)
      .fill(0)
      .concat(data.wave_period_data);
    temperature = new Array(prefillLength).fill(0).concat(data.temperature);
    dewpoint = new Array(prefillLength).fill(0).concat(data.dewpoint);
    cloud_cover = new Array(prefillLength).fill(0).concat(data.cloud_cover);
    probability_of_precipitation = new Array(prefillLength)
      .fill(0)
      .concat(data.probability_of_precipitation);
    probability_of_thunder = new Array(prefillLength)
      .fill(0)
      .concat(data.probability_of_thunder);
  }

  let startingAt = new Date().getHours();

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
      setStyleAttribute("wave-quality", `background-color: ${FLAT_COLOR}`);

      setText("wave-quality-text", qualityMap[FLAT_COLOR]);
      setStyleAttribute("wave-quality-text", `color: ${FLAT_COLOR}`);

      removeElements(".wave-quality-loader");
    }

    setStyleAttribute(
      "wave-icon",
      `transform: rotate(${data.current_wave_direction}deg);`,
    );
    removeElements(".wavey");
  }

  // The period can be undefined separate from the wave height
  const wave_period_container = document.getElementById("current-wave-period");
  if (wave_period_container?.innerText === "") {
    wave_period_container.innerText = data.current_wave_period;
    removeElement("wavey-period-loader");
  }

  setText("legend-label", wave_height_labels[startingAt]);
  setText("legend-quality", qualityMap[qualities[startingAt]]);
  setText("legend-wave-height", wave_heights[startingAt]);
  setText("legend-wind-speed", wind_speeds[startingAt]);
  setStyleAttribute(
    "legend-wind-icon",
    `transform: rotate(${wind_directions[startingAt]}deg);`,
  );
  setText("legend-wave-period", wave_period[startingAt]);
  setText("legend-wind-gust", wind_gusts[startingAt]);
  setText("forecast-as-of", `Updated @ ${data.forecast_as_of}`);

  removeElements(".loader");
  removeHidden("forecast");
  removeHidden("wave-quality");
  removeHidden("legend-container");
  removeStyle("forecast-as-of-container", "animate-pulse");

  // -- Init temperature legend --
  removeHidden("temperature-legend-container");
  setText("temperature-legend-label", wave_height_labels[startingAt]);
  setText("temperature-legend-temperature", temperature[startingAt]);
  setText("temperature-legend-dewpoint", dewpoint[startingAt]);

  // -- Init precipitation legend --
  removeHidden("precipitation-legend-container");
  setText("precipitation-legend-label", wave_height_labels[startingAt]);
  setText(
    "precipitation-legend-precipitation",
    probability_of_precipitation[startingAt],
  );
  setText("precipitation-legend-thunder", probability_of_thunder[startingAt]);
  setText("precipitation-legend-cloud-cover", cloud_cover[startingAt]);

  setStyleAttribute("legend", `background-color: ${qualities[startingAt]};`);

  const ctx = document.getElementById("forecast");

  const quality = (ctx) => qualities[ctx.dataIndex + start];

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
      // Draw a vertical line representing the current time on the graph
      if (startingAt > start) {
        const ctx = chart.ctx;
        const xAxis = chart.scales.x;

        const xValue = xAxis.getPixelForValue(startingAt);
        ctx.save();
        ctx.strokeStyle = "#3b3b42";
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.moveTo(xValue, 0);
        ctx.lineTo(xValue, chart.height);
        ctx.stroke();
        ctx.fillStyle = "#3b3b42";
        ctx.font = "bold 1rem ui-sans-serif, system-ui, sans-serif";
        ctx.fillText("Now", startingAt > 14 ? xValue - 45 : xValue + 5, 15);
        ctx.restore();
      }
    },
  };

  /**
   * Gets the correct x based on the data to be displayed
   *
   * @param {number} x_value
   */
  function getX(x_value) {
    if (stepBy === 24 || stepBy === 48) {
      return x_value && x_value > 0
        ? x_value >= wave_heights.length - start
          ? wave_heights.length - start - 1
          : x_value
        : 0;
    }
    return x_value && x_value > 0
      ? x_value >= wave_heights.length
        ? wave_heights.length - 1
        : x_value
      : 0;
  }

  /**
   * Updates values shown in legends with selected indicy of
   * data.
   *
   * @param {number} x
   */
  function updateLegends(x) {
    const color = qualities[x + start];

    // Update wave legend
    setStyleAttribute("legend", `background-color: ${color}`);
    setText("legend-label", wave_height_labels[x + start]);
    setText("legend-quality", qualityMap[color]);
    setText("legend-wave-height", wave_heights[x + start]);
    setText("legend-wind-speed", wind_speeds[x + start]);
    setStyleAttribute(
      "legend-wind-icon",
      `transform: rotate(${wind_directions[x + start]}deg);`,
    );
    setText("legend-wave-period", wave_period[x + start]);
    setText("legend-wind-gust", wind_gusts[x + start]);

    // Update temperature legend
    setText("temperature-legend-label", wave_height_labels[x + start]);
    setText("temperature-legend-temperature", temperature[x + start]);
    setText("temperature-legend-dewpoint", dewpoint[x + start]);

    // Update precipitation legend
    setText("precipitation-legend-label", wave_height_labels[x + start]);
    setText(
      "precipitation-legend-precipitation",
      probability_of_precipitation[x + start],
    );
    setText("precipitation-legend-thunder", probability_of_thunder[x + start]);
    setText("precipitation-legend-cloud-cover", cloud_cover[x + start]);
  }

  const onHover = (e, _, chart) => {
    const canvasPosition = Chart.helpers.getRelativePosition(e, chart);
    // Substitute the appropriate scale IDs
    const x_value = chart.scales.x.getValueForPixel(canvasPosition.x);
    const x = getX(x_value);

    updateLegends(x);
  };

  const xTicksCallback = (_value, i, _ticks) => {
    if (stepBy === 24) {
      return i % 6 === 0 ? wave_height_labels[i + start] : null;
    }
    if (stepBy === 48) {
      return i % 8 === 0 ? wave_height_labels[i + start] : null;
    }
    return i % 24 === 0 ? wave_height_labels[i + start] : null;
  };

  function colorize() {
    return (ctx) => quality(ctx) || "#4ade80";
  }

  // window widths align with tailwind md and lg
  const getStepBy = () =>
    window.innerWidth < 768
      ? 24
      : window.innerWidth < 1024
        ? 48
        : wave_heights.length;

  let stepBy = getStepBy();

  let start = 0;
  let end = start + stepBy;

  const font = {
    size: stepBy === 24 ? 14 : 18,
    weight: stepBy === 24 ? "lighter" : "semi-bold",
  };

  const waveForecast = new Chart(ctx, {
    type: "bar",
    plugins: [plugin],
    data: {
      labels: wave_height_labels.slice(start, end),
      datasets: [
        {
          label: "wave height (feet)",
          data: wave_heights.slice(start, end),
          pointStyle: false,
          minBarLength: 0.1,
        },
      ],
    },
    options: {
      onHover,
      borderRadius: 5,
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
        bar: {
          backgroundColor: colorize(),
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
            font,
          },
        },
      },
    },
  });

  /**
   * function to set update the dom with the forecast's range.
   *
   * @param {string} label
   */
  function setForecastRange(label) {
    setText("forecast-range", label);
  }
  function getForecastRangeLabel() {
    if (start === 0) {
      let dayOffset = prefillLength > 20 && new Date().getHours() > 20;
      if (stepBy === 24) {
        dayOffset ? setForecastRange("Tomorrow") : setForecastRange("Today");
      } else {
        dayOffset
          ? setForecastRange(
              `Tomorrow - ${wave_height_labels[start + 25].split(" ")[0]}`,
            )
          : setForecastRange("Today - Tomorrow");
      }
    } else {
      if (stepBy === 24) {
        setForecastRange(wave_height_labels[start].split(" ")[0]);
      } else {
        console.log(end);
        const last =
          wave_height_labels[end - 1] ??
          wave_height_labels[wave_height_labels.length - 1];
        setForecastRange(
          `${wave_height_labels[start].split(" ")[0]} - ${last.split(" ")[0]}`,
        );
      }
    }
  }
  getForecastRangeLabel();

  function updateCharts() {
    let labels = wave_height_labels.slice(start, end);
    waveForecast.data.labels = labels;
    waveForecast.data.datasets[0].data = wave_heights.slice(start, end);
    waveForecast.update();

    temperatureForecast.data.labels = labels;
    temperatureForecast.data.datasets[0].data = temperature.slice(start, end);
    temperatureForecast.update();

    precipitationForecast.data.labels = labels;
    precipitationForecast.data.datasets[0].data =
      probability_of_precipitation.slice(start, end);
    precipitationForecast.update();
  }

  window.addEventListener("resize", () => {
    stepBy = getStepBy();

    if (start + stepBy > wave_height_labels.length) {
      end = wave_height_labels.length;
      start = wave_height_labels.length - stepBy;
    } else {
      end = start + stepBy;
    }

    updateCharts();
    updateLegends(0);

    if (start === 0) {
      NonNull(document.getElementById("forecast-backward")).disabled = true;
    }
    if (start + stepBy < wave_heights.length) {
      NonNull(document.getElementById("forecast-foreward")).disabled = false;
    }
    if (start + stepBy >= wave_heights.length) {
      NonNull(document.getElementById("forecast-foreward")).disabled = true;
    }
    if (start > 0) {
      NonNull(document.getElementById("forecast-backward")).disabled = false;
    }

    getForecastRangeLabel();
  });

  document
    .getElementById("forecast-backward")
    ?.addEventListener("click", () => {
      if (start - stepBy < 0) {
        start = 0;
        end = stepBy;
      } else {
        end -= stepBy;
        start -= stepBy;
      }

      updateCharts();
      updateLegends(start === 0 ? startingAt : 0);

      if (start === 0) {
        NonNull(document.getElementById("forecast-backward")).disabled = true;
      }
      if (start + stepBy < wave_heights.length) {
        NonNull(document.getElementById("forecast-foreward")).disabled = false;
      }

      getForecastRangeLabel();
    });

  document
    .getElementById("forecast-foreward")
    ?.addEventListener("click", () => {
      if (end + stepBy > wave_height_labels.length) {
        end = wave_height_labels.length;
        start = end - stepBy;
      } else {
        start += stepBy;
        end += stepBy;
      }

      updateCharts();
      updateLegends(0);

      if (start + stepBy >= wave_heights.length) {
        NonNull(document.getElementById("forecast-foreward")).disabled = true;
      }
      if (start > 0) {
        NonNull(document.getElementById("forecast-backward")).disabled = false;
      }

      getForecastRangeLabel();
    });

  const getConfig = (data, color, label, max) => ({
    type: "bar",
    plugins: [plugin],
    data: {
      labels: wave_height_labels.slice(start, end),
      datasets: [
        {
          label,
          data,
          pointStyle: false,
          minBarLength: 0.1,
        },
      ],
    },
    options: {
      stacked: true,
      onHover,
      maintainAspectRatio: false,
      borderRadius: 5,
      plugins: {
        legend: {
          display: false,
        },
        tooltip: {
          enabled: false,
        },
      },
      elements: {
        bar: color
          ? {
              backgroundColor: color,
              borderColor: color,
            }
          : {},
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
            font,
          },
        },
      },
    },
  });

  const temperatureCanvas = document.getElementById("temperature-forecast");
  const temperatureForecast = new Chart(
    temperatureCanvas,
    getConfig(temperature.slice(start, end), "pink", "F", null),
  );

  const precipitationCanvas = document.getElementById("precipitation-forecast");
  const precipitationForecast = new Chart(
    precipitationCanvas,
    getConfig(probability_of_precipitation.slice(start, end), null, "%", 100),
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
