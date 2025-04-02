import {
  setText,
  setStyleAttribute,
  removeHidden,
  removeStyle,
  removeElement,
  removeElements,
  asButton,
  outOfDate,
} from "../utilities";

const FLAT_COLOR = "#a8a29e";

const QUALITY_MAP = {
  "#0bd674": "Good",
  "#ffcd1e": "Fair to Good",
  "#ff9500": "Poor",
  "#f4496d": "Very Poor",
  "#a8a29e": "Flat",
};

let qualities;
let wave_height_labels;
let wave_heights;
let wind_speeds;
let wind_directions;
let wind_gusts;
let wave_period;
let temperature;
let dewpoint;
let cloud_cover;
let probability_of_precipitation;
let probability_of_thunder;

/**
 * @typedef {Object} ForecastData
 * @property {string[]} wave_height
 * @property {string} current_wave_height
 * @property {string} current_wave_direction
 * @property {string} current_wave_period
 * @property {string[]} wind_speed
 * @property {string[]} wind_direction
 * @property {string[]} wind_gust
 * @property {string[]} wave_period
 * @property {string[]} wave_height_labels
 * @property {string[]} as_of
 * @property {string[]} temperature
 * @property {string[]} probability_of_precipitation
 * @property {string[]} dewpoint
 * @property {string[]} cloud_cover
 * @property {string[]} probability_of_thunder
 * @property {string[]} quality
 * @property {string} starting_at
 */

/**
 * Takes the forecast data JSON and updates the HTML
 *
 * @param {ForecastData} data
 */
export function parseForecast(data) {
  // prefillLength represents the time in hours that the data from the forecast starts at, e.g.
  // if the forecast data starts at 2 PM, prefillLength = 14;
  const prefillLength = new Date(data.starting_at).getHours();
  let dataStartingAt = new Date(data.starting_at).getHours();

  // Might be simpler to just align this to be the same day no matter what, then the
  // dayoffset below isn't needed and this logic just ensures starting_at is within the current day of the user
  // if (new Date(data.starting_at).getDay() < new Date().getDay()) {
  if (prefillLength > 20) {
    let offset = 24 - prefillLength;
    const dayAlign =
      data.wave_height_labels.length -
      ((data.wave_height_labels.length - offset) % 24);

    dataStartingAt = 0;
    wave_height_labels = data.wave_height_labels.slice(offset, dayAlign);
    qualities = data.quality.slice(offset, dayAlign);
    wave_heights = data.wave_height.slice(offset, dayAlign);
    wind_speeds = data.wind_speed.slice(offset, dayAlign);
    wind_directions = data.wind_direction.slice(offset, dayAlign);
    wind_gusts = data.wind_gust.slice(offset, dayAlign);
    wave_period = data.wave_period.slice(offset, dayAlign);
    temperature = data.temperature.slice(offset, dayAlign);
    dewpoint = data.dewpoint.slice(offset, dayAlign);
    cloud_cover = data.cloud_cover.slice(offset, dayAlign);
    probability_of_precipitation = data.probability_of_precipitation.slice(
      offset,
      dayAlign,
    );
    probability_of_thunder = data.probability_of_thunder.slice(
      offset,
      dayAlign,
    );
  } else {
    const weekday = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const dayLabel = weekday[new Date(data.starting_at).getDay()];

    const prefillLabels = [];
    for (let i = 0; i < prefillLength; i++) {
      if (i === 0) {
        prefillLabels.push(`${dayLabel} 12 AM`);
        continue;
      }
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

    const dayAlign =
      data.wave_height_labels.length +
      prefillLength -
      ((data.wave_height_labels.length + prefillLength) % 24);

    wave_height_labels = prefillLabels
      .concat(data.wave_height_labels)
      .slice(0, dayAlign);

    qualities = new Array(prefillLength)
      .fill("#a8a29e")
      .concat(data.quality)
      .slice(0, dayAlign);
    wave_heights = new Array(prefillLength)
      .fill(0)
      .concat(data.wave_height)
      .slice(0, dayAlign);
    wind_speeds = new Array(prefillLength)
      .fill(0)
      .concat(data.wind_speed)
      .slice(0, dayAlign);
    wind_directions = new Array(prefillLength)
      .fill(0)
      .concat(data.wind_direction)
      .slice(0, dayAlign);
    wind_gusts = new Array(prefillLength)
      .fill(0)
      .concat(data.wind_gust)
      .slice(0, dayAlign);
    wave_period = new Array(prefillLength)
      .fill(0)
      .concat(data.wave_period)
      .slice(0, dayAlign);
    temperature = new Array(prefillLength)
      .fill(0)
      .concat(data.temperature)
      .slice(0, dayAlign);
    dewpoint = new Array(prefillLength)
      .fill(0)
      .concat(data.dewpoint)
      .slice(0, dayAlign);
    cloud_cover = new Array(prefillLength)
      .fill(0)
      .concat(data.cloud_cover)
      .slice(0, dayAlign);
    probability_of_precipitation = new Array(prefillLength)
      .fill(0)
      .concat(data.probability_of_precipitation)
      .slice(0, dayAlign);
    probability_of_thunder = new Array(prefillLength)
      .fill(0)
      .concat(data.probability_of_thunder)
      .slice(0, dayAlign);
  }

  let startingAt = new Date().getHours();

  const wave_height_container = document.getElementById("current-wave-height");

  // This means there was no wave height data from the bouy
  if (wave_height_container?.innerText === "") {
    wave_height_container.innerText = data.current_wave_height;

    // if the current wave height data is under a foot, update the
    // quality to be Flat
    if (data.current_wave_height == "0") {
      setStyleAttribute("wave-quality", `background-color: ${FLAT_COLOR}`);

      setText("wave-quality-text", QUALITY_MAP[FLAT_COLOR]);
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

  // -- Init wave legend --
  setText("legend-label", wave_height_labels[startingAt]);
  setText("legend-quality", QUALITY_MAP[qualities[startingAt]]);
  setText("legend-wave-height", wave_heights[startingAt]);
  setText("legend-wind-speed", wind_speeds[startingAt]);
  setStyleAttribute(
    "legend-wind-icon",
    `transform: rotate(${wind_directions[startingAt] + 180}deg);`,
  );
  setText("legend-wave-period", wave_period[startingAt]);
  setText("legend-wind-gust", wind_gusts[startingAt]);
  setText("forecast-as-of", `Updated ${data.as_of}`);
  setText("forecast-as-of-2", `Updated ${data.as_of}`);

  let oneDayMs = 60 * 60 * 24 * 1_000;
  if (new Date(data.as_of) < new Date() - oneDayMs) {
    outOfDate(["forecast-as-of-container-2", "forecast-as-of-container"]);
  }

  removeElements(".loader");
  removeHidden("forecast");
  removeHidden("wave-quality");
  removeHidden("legend-container");
  removeStyle("forecast-as-of-container", "animate-pulse");
  removeStyle("forecast-as-of-container-2", "animate-pulse");

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
  setStyleAttribute(
    [
      "legend-container",
      "precipitation-legend-container",
      "temperature-legend-container",
    ],
    "margin-top: 1rem;",
  );

  const quality = (ctx) =>
    start === 0
      ? qualities[ctx.dataIndex + dataStartingAt]
      : qualities[ctx.dataIndex + start];

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
      if (startingAt - dataStartingAt > start) {
        const ctx = chart.ctx;
        const xAxis = chart.scales.x;

        const xValue = xAxis.getPixelForValue(startingAt - dataStartingAt);
        ctx.save();
        ctx.strokeStyle = "#5b5b58";
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.moveTo(xValue, 0);
        ctx.lineTo(xValue, chart.height);
        ctx.stroke();
        ctx.fillStyle = "#5b5b58";
        ctx.font = "bold 1rem ui-sans-serif, system-ui, sans-serif";
        ctx.fillText(
          "Now",
          startingAt > 14 && stepBy === 24 ? xValue - 45 : xValue + 5,
          15,
        );
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
   * Updates values shown in legends with the current time indicy of
   * data.
   */
  function updateLegendsToStart() {
    start === 0 ? applyUpdateToLegends(startingAt) : updateLegends(0);
  }

  /**
   * Updates values shown in legends with selected indicy of
   * data.
   *
   * @param {number} x
   */
  function updateLegends(x) {
    let v;
    if (start === 0) {
      if (wave_height_labels.length - prefillLength - 1 <= x) {
        v = wave_height_labels.length - prefillLength - 1;
      } else {
        v = x;
      }
    } else if (x >= wave_height_labels.length - prefillLength) {
      v = wave_height_labels.length - prefillLength - 1;
    } else {
      v = x;
    }
    const beginning = start === 0 ? dataStartingAt : start;

    applyUpdateToLegends(v + beginning);
  }

  function applyUpdateToLegends(x) {
    const color = qualities[x];
    // Update wave legend
    setStyleAttribute("legend", `background-color: ${color}`);
    setText("legend-label", wave_height_labels[x]);
    setText("legend-quality", QUALITY_MAP[color]);
    setText("legend-wave-height", wave_heights[x]);
    setText("legend-wind-speed", wind_speeds[x]);
    setStyleAttribute(
      "legend-wind-icon",
      `transform: rotate(${wind_directions[x] + 180}deg);`,
    );
    setText("legend-wave-period", wave_period[x]);
    setText("legend-wind-gust", wind_gusts[x]);

    // Update temperature legend
    setText("temperature-legend-label", wave_height_labels[x]);
    setText("temperature-legend-temperature", temperature[x]);
    setText("temperature-legend-dewpoint", dewpoint[x]);

    // Update precipitation legend
    setText("precipitation-legend-label", wave_height_labels[x]);
    setText(
      "precipitation-legend-precipitation",
      probability_of_precipitation[x],
    );
    setText("precipitation-legend-thunder", probability_of_thunder[x]);
    setText("precipitation-legend-cloud-cover", cloud_cover[x]);
  }

  const onHover = (e, _, chart) => {
    const canvasPosition = Chart.helpers.getRelativePosition(e, chart);
    // Substitute the appropriate scale IDs
    const x_value = chart.scales.x.getValueForPixel(canvasPosition.x);
    const x = getX(x_value);

    updateLegends(x);
  };

  /**
   * Returns x ticks according to the labels and starting value.
   *
   * @param {string} _value
   * @param {number} i
   */
  const xTicksCallback = (_value, i) => {
    const beginning = start === 0 ? dataStartingAt : start;
    if (stepBy === 24) {
      return i % 6 === 0 ? wave_height_labels[i + beginning] : null;
    }
    if (stepBy === 48) {
      return i % 8 === 0 ? wave_height_labels[i + beginning] : null;
    }
    return i % 24 === 0 ? wave_height_labels[i + beginning] : null;
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
    weight: "semi-bold",
  };

  /**
   * @param {number | undefined} max
   */
  const options = (max) => ({
    aspectRatio: 1.75,
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
        grid: {
          color: "#1B1B1B",
        },
        beginAtZero: true,
        max: max ?? 10,
        ticks: {
          /**
           * Function to return only even y ticks
           *
           * @param {number} value
           */
          callback: function (value) {
            if (value % 2 !== 0) {
              return "";
            }
            return value;
          },
          font,
        },
      },
    },
  });

  // If a wave height is greater than 10, make sure the chart
  // includes that height. Otherwise keep the chart consistent at
  // 10 ft.
  let wave_height_max = wave_heights.reduce(
    (acc, curr) => (curr > acc ? curr : acc),
    10,
  );

  const waveForecast = new Chart(document.getElementById("forecast"), {
    type: "bar",
    plugins: [plugin],
    data: {
      labels:
        start === 0
          ? wave_height_labels.slice(dataStartingAt, end)
          : wave_height_labels.slice(start, end),
      datasets: [
        {
          label: "wave height (feet)",
          data:
            start === 0
              ? wave_heights.slice(dataStartingAt, end)
              : wave_heights.slice(start, end),
          pointStyle: false,
          minBarLength: 0.1,
        },
      ],
    },
    options: {
      ...options(wave_height_max),

      elements: {
        bar: {
          backgroundColor: colorize(),
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
    const beginning = start === 0 ? dataStartingAt : start;
    let labels = wave_height_labels.slice(beginning, end);

    waveForecast.data.labels = labels;
    waveForecast.data.datasets[0].data = wave_heights.slice(beginning, end);
    waveForecast.update();

    temperatureForecast.data.labels = labels;
    temperatureForecast.data.datasets[0].data = temperature.slice(
      beginning,
      end,
    );
    temperatureForecast.update();

    precipitationForecast.data.labels = labels;
    precipitationForecast.data.datasets[0].data =
      probability_of_precipitation.slice(beginning, end);
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
      asButton(document.getElementById("forecast-backward")).disabled = true;
    }
    if (start + stepBy < wave_heights.length) {
      asButton(document.getElementById("forecast-foreward")).disabled = false;
    }
    if (start + stepBy >= wave_heights.length) {
      asButton(document.getElementById("forecast-foreward")).disabled = true;
    }
    if (start > 0) {
      asButton(document.getElementById("forecast-backward")).disabled = false;
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
      updateLegendsToStart();

      if (start === 0) {
        asButton(document.getElementById("forecast-backward")).disabled = true;
      }
      if (start + stepBy < wave_heights.length) {
        asButton(document.getElementById("forecast-foreward")).disabled = false;
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
        asButton(document.getElementById("forecast-foreward")).disabled = true;
      }
      if (start > 0) {
        asButton(document.getElementById("forecast-backward")).disabled = false;
      }

      getForecastRangeLabel();
    });

  /**
   * Creates a config for a chart for ChartJS
   *
   * @param {number[]} data
   * @param {string | null} color
   * @param {string} label
   */
  const getConfig = (data, color, label) => ({
    type: "bar",
    plugins: [plugin],
    data: {
      labels:
        start === 0
          ? wave_height_labels.slice(dataStartingAt, end)
          : wave_height_labels.slice(start, end),
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
      ...options(100),
      elements: {
        bar: color
          ? {
              backgroundColor: color,
              borderColor: color,
            }
          : {},
      },
    },
  });

  const temperatureCanvas = document.getElementById("temperature-forecast");
  const temperatureForecast = new Chart(
    temperatureCanvas,
    getConfig(
      start === 0
        ? temperature.slice(dataStartingAt, end)
        : temperature.slice(start, end),
      "pink",
      "F",
    ),
  );

  const precipitationCanvas = document.getElementById("precipitation-forecast");
  const precipitationForecast = new Chart(
    precipitationCanvas,
    getConfig(
      start === 0
        ? probability_of_precipitation.slice(dataStartingAt, end)
        : probability_of_precipitation.slice(start, end),
      null,
      "%",
    ),
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
