import { describe, it } from "node:test";
import assert from "node:assert";
import { getWindData } from "./realtime.js";

describe("getWindData", () => {
  it("getWindData returns just the wind_speed if gusts and wind_speed are the same", () => {
    assert.strictEqual(getWindData({ wind_speed: "55", gusts: "55" }), "55");
  });

  it("getWindData returns just the wind speed if gusts are 0", () => {
    assert.strictEqual(getWindData({ wind_speed: "56", gusts: "0" }), "56");
  });

  it("getWindData returns the wind speed and gusts if they're different", () => {
    assert.strictEqual(getWindData({ wind_speed: "56", gusts: "57" }), "56-57");
  });
});
