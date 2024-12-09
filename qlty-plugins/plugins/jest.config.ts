import type { Config } from "jest";

const config: Config = {
  preset: "ts-jest",
  modulePaths: ["<rootDir>"],
  setupFilesAfterEnv: ["<rootDir>/tests/jest_setup.ts"],
  modulePathIgnorePatterns: ["<rootDir>/.qlty/"],
};

export default config;
