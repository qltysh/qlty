export default [
  {
    languageOptions: {
      ecmaVersion: "latest",
      sourceType: "module",
      globals: {
        console: "readonly",
      },
    },
    rules: {
      "no-cond-assign": "error",
      "no-constant-condition": "error",
      "no-unused-vars": "error",
    },
  },
];
