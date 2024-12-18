/** @type {import('eslint').Linter.Config} */
module.exports = {
  root: true,
  ignorePatterns: ["/linters"],
  parserOptions: {
    ecmaVersion: "latest",
    sourceType: "module",
  },
  plugins: ["unused-imports"],
  extends: ["eslint:recommended"],
  rules: {
    "unused-imports/no-unused-imports": "warn",
  },
  overrides: [
    {
      files: ["**/*.{ts,tsx}"],
      plugins: ["@typescript-eslint", "import"],
      parser: "@typescript-eslint/parser",
      parserOptions: {
        project: "qlty-plugins/plugins/tsconfig.json",
      },
      extends: [
        "plugin:@typescript-eslint/recommended",
        "plugin:import/recommended",
        "plugin:import/typescript",
      ],
      rules: {
        "import/no-unresolved": "off",
        "@typescript-eslint/no-explicit-any": "warn",
        "@typescript-eslint/no-floating-promises": "warn",
        "@typescript-eslint/no-misused-promises": "warn",
      },
    },
  ],
};
