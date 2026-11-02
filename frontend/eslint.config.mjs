import js from "@eslint/js";
import tseslint from "typescript-eslint";
import eslintConfigPrettier from "eslint-config-prettier";
import { defineConfig, globalIgnores } from "eslint/config";

export default defineConfig([
  globalIgnores([
    "**/dist",
    "**/vite.config.js",
    "**/playwright.config.ts",
    "**/mock-api",
    "*/types/",
    "**/mock-data",
    "**/scripts/",
    "**/assets/",
    "**/test/",
    "*.test.ts",
    "**/*.worker.ts",
    "**/vitest.config.js",
  ]),
  {
    files: ["**/*.{js,ts}"],
    extends: [js.configs.recommended, tseslint.configs.recommendedTypeChecked],
    languageOptions: {
      parserOptions: { projectService: true },
    },

    rules: {
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          argsIgnorePattern: "^_",
          destructuredArrayIgnorePattern: "^_",
          varsIgnorePattern: "^_",
          caughtErrorsIgnorePattern: "^_",
        },
      ],
    },
  },
  eslintConfigPrettier,
]);
