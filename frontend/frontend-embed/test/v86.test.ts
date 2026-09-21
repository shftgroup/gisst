import { describe, it, expect } from "vitest";
import { nested_replace } from "../src/v86";

// nested_replace walks an object and swaps any value that is exactly
// equal to the target string. .

describe("nested_replace", () => {
  it("replaces the placeholder wherever it appears", () => {
    // The placeholder sits one level down, so the function has to walk
    // into "disk" to find it. "bios" does not match and should survive.
    const config = {
      bios: { url: "keep-this" },
      disk: { url: "PLACEHOLDER" },
    };

    nested_replace(config, "PLACEHOLDER", "REPLACED");

    expect(config).toEqual({
      bios: { url: "keep-this" },
      disk: { url: "REPLACED" },
    });
  });

  it("leaves everything else alone", () => {
    // Nothing here equals the placeholder, so nothing should change.
    const config = {
      bios: { url: "keep-this" },
      disk: { url: "keep-this-too" },
    };

    nested_replace(config, "PLACEHOLDER", "REPLACED");

    expect(config).toEqual({
      bios: { url: "keep-this" },
      disk: { url: "keep-this-too" },
    });
  });

  it("does not replace substrings", () => {
    // "PLACEHOLDER-plus-extra" contains the placeholder but is not equal
    // to it, so it is left untouched.
    const config = {
      exact: "PLACEHOLDER",
      partial: "PLACEHOLDER-plus-extra",
    };

    nested_replace(config, "PLACEHOLDER", "REPLACED");

    expect(config).toEqual({
      exact: "REPLACED",
      partial: "PLACEHOLDER-plus-extra",
    });
  });
});
