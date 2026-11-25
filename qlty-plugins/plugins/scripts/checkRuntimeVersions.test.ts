import fs from "fs";
import path from "path";
import {
  isOlder,
  issuesForRuntime,
  parseRuntimeDefaults,
  releaseLine,
} from "./checkRuntimeVersions";

const supported = (latest: string) => ({
  isEol: false,
  eolFrom: "2030-01-01",
  latest,
  link: null,
});

describe("parseRuntimeDefaults", () => {
  test("reads every runtime default from tool_builder.rs", () => {
    const source = fs.readFileSync(
      path.resolve(__dirname, "../../../qlty-check/src/tool/tool_builder.rs"),
      "utf8",
    );
    expect(
      parseRuntimeDefaults(source)
        .map((d) => d.runtime)
        .sort(),
    ).toEqual(["Go", "Java", "Node", "Php", "Python", "Ruby", "Rust"]);
  });
});

describe("isOlder", () => {
  test("compares patch numbers numerically", () => {
    expect(isOlder("3.3.9", "3.3.10")).toBe(true);
  });

  test("treats an extra Temurin component as newer than a higher build", () => {
    expect(isOlder("25.0.5+8", "25.0.5.1+2")).toBe(true);
  });

  test("treats a higher Temurin build as newer", () => {
    expect(isOlder("25.0.5+8", "25.0.5+9")).toBe(true);
  });

  test("is false when the default is the latest", () => {
    expect(isOlder("22.23.5", "22.23.5")).toBe(false);
  });
});

describe("releaseLine", () => {
  test("drops the Temurin build before taking the line", () => {
    expect(releaseLine("25.0.5+8", 1)).toBe("25");
  });
});

describe("issuesForRuntime", () => {
  test("reports a default behind the latest patch", () => {
    const issues = issuesForRuntime(
      { runtime: "Ruby", version: "3.3.7" },
      "3.3",
      supported("3.3.14"),
    );
    expect(issues.map((i) => i.title)).toEqual([
      "Default Ruby runtime 3.3.7 is behind 3.3.14",
    ]);
  });

  test("reports nothing for a current default", () => {
    expect(
      issuesForRuntime(
        { runtime: "Ruby", version: "3.3.14" },
        "3.3",
        supported("3.3.14"),
      ),
    ).toEqual([]);
  });

  test("reports a current default on an unsupported line", () => {
    const issues = issuesForRuntime(
      { runtime: "Rust", version: "1.77.2" },
      "1.77",
      { isEol: true, eolFrom: "2024-05-02", latest: "1.77.2", link: null },
    );
    expect(issues.map((i) => i.title)).toEqual([
      "Default Rust runtime is on unsupported line 1.77",
    ]);
  });

  test("skips the patch check for a default that names only a line", () => {
    expect(
      issuesForRuntime(
        { runtime: "Php", version: "8.3" },
        "8.3",
        supported("8.3.37"),
      ),
    ).toEqual([]);
  });
});
