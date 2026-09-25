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

  it("replaces the placeholder up to four levels down", () => {
    // The other tests only go one level down. This one has the
    // placeholder one, two, and three, and four levels down.
    const config = {
      one: { url: "PLACEHOLDER" },
      two: {
        inner: { url: "PLACEHOLDER" },
      },
      three: {
        mid: {
          inner: { url: "PLACEHOLDER" },
        },
      },

      four: {
        inner: {
          inner: { url: "PLACEHOLDER" },
        },
      },
    };

    nested_replace(config, "PLACEHOLDER", "REPLACED");

    expect(config).toEqual({
      one: { url: "REPLACED" },
      two: { 
        inner: { url: "REPLACED" }
       },
      three: {
        mid: {
          inner: { url: "REPLACED" },
        },
      },
      four: {
        inner: {
          inner: { url: "REPLACED" },
        },
      },
    });
  });

  it("leaves an empty object alone", () => {
    // There is nothing to walk inside "disk", and "bios" does not match.
    const config = {
      disk: {},
      bios: { url: "keep-this" },
    };

    nested_replace(config, "PLACEHOLDER", "REPLACED");

    expect(config).toEqual({
      disk: {},
      bios: { url: "keep-this" },
    });
  });

  it("does not replace keys, only values", () => {
    // The key is named PLACEHOLDER, but the function only looks at values.
    const config = {
      PLACEHOLDER: "keep-this",
      disk: { url: "PLACEHOLDER" },
    };

    nested_replace(config, "PLACEHOLDER", "REPLACED");

    expect(config).toEqual({
      PLACEHOLDER: "keep-this",
      disk: { url: "REPLACED" },
    });
  });


  it("does not replace a different case or extra space", () => {
    // The match has to be the whole string, with the same letters and
    // no extra characters.
    const config = {
      lower: "placeholder",
      spaced: "PLACEHOLDER ",
      exact: "PLACEHOLDER",
    };

    nested_replace(config, "PLACEHOLDER", "REPLACED");

    expect(config).toEqual({
      lower: "placeholder",
      spaced: "PLACEHOLDER ",
      exact: "REPLACED",
    });
  });

  it("replaces an empty string only when that is the target", () => {
    // "" is a real value. It is replaced only when we search for "".
    const config = {
      blank: "",
      disk: { url: "PLACEHOLDER" },
    };

    nested_replace(config, "", "REPLACED");

    expect(config).toEqual({
      blank: "REPLACED",
      disk: { url: "PLACEHOLDER" },
    });
  });
});
