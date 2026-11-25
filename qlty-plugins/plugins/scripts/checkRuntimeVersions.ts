import fs from "fs";
import path from "path";

// Finds CLI runtime defaults that are behind the latest patch release on
// their line, or on a line upstream no longer supports, and writes one issue
// per finding to runtime_issues.json for the workflow to file.

const REPO_ROOT = path.resolve(__dirname, "../../..");
const TOOL_BUILDER_PATH = path.join(
  REPO_ROOT,
  "qlty-check/src/tool/tool_builder.rs",
);
const RUBY_SOURCE_PATH = path.join(
  REPO_ROOT,
  "qlty-check/src/tool/ruby_source.rs",
);
const OUTPUT_PATH = path.resolve(__dirname, "..", "runtime_issues.json");

// endoflife.date product for each Runtime variant, and how many leading
// version components name its release line.
const PRODUCTS: Record<string, { product: string; lineParts: number }> = {
  Go: { product: "go", lineParts: 2 },
  Node: { product: "nodejs", lineParts: 1 },
  Python: { product: "python", lineParts: 2 },
  Ruby: { product: "ruby", lineParts: 2 },
  Rust: { product: "rust", lineParts: 2 },
  Java: { product: "eclipse-temurin", lineParts: 1 },
  Php: { product: "php", lineParts: 2 },
};

export interface RuntimeDefault {
  runtime: string;
  version: string;
}

export interface ReleaseLine {
  isEol: boolean;
  eolFrom: string | null;
  latest: string;
  link: string | null;
}

// key names what an issue is about, such as "Ruby-behind", so the workflow
// can close an open issue when a newer finding about the same thing replaces it.
export interface Issue {
  key: string;
  title: string;
  body: string;
}

export const parseRuntimeDefaults = (rawSource: string): RuntimeDefault[] => {
  const source = rawSource.replace(/\r\n/g, "\n");
  const start = source.indexOf("fn latest_runtime_version");
  if (start === -1) {
    throw new Error("latest_runtime_version not found in tool_builder.rs");
  }
  const end = source.indexOf("\n    }\n", start);
  if (end === -1) {
    throw new Error(
      "End of latest_runtime_version not found in tool_builder.rs",
    );
  }
  const body = source.slice(start, end);
  const defaults = [...body.matchAll(/Runtime::(\w+) => Some\("([^"]+)"/g)].map(
    ([, runtime, version]) => ({ runtime, version }),
  );
  const arms = body.match(/Runtime::\w+ =>/g)?.length ?? 0;
  if (defaults.length === 0 || defaults.length !== arms) {
    throw new Error(
      `Parsed ${defaults.length} of ${arms} runtime defaults in latest_runtime_version`,
    );
  }
  return defaults;
};

// Splits "25.0.5+8" or "25.0.5.1+2" into comparable numbers: four version
// components, zero-padded, then the build number.
export const versionKey = (version: string): number[] => {
  const [main, build = "0"] = version.split("+");
  const parts = main.split(".").map(Number);
  if ([...parts, Number(build)].some(Number.isNaN)) {
    throw new Error(`Can't compare version ${version}`);
  }
  while (parts.length < 4) parts.push(0);
  return [...parts, Number(build)];
};

export const isOlder = (current: string, latest: string): boolean => {
  const a = versionKey(current);
  const b = versionKey(latest);
  for (let i = 0; i < Math.max(a.length, b.length); i++) {
    if ((a[i] ?? 0) !== (b[i] ?? 0)) return (a[i] ?? 0) < (b[i] ?? 0);
  }
  return false;
};

export const releaseLine = (version: string, lineParts: number): string =>
  version.split("+")[0].split(".").slice(0, lineParts).join(".");

export const issuesForRuntime = (
  runtimeDefault: RuntimeDefault,
  line: string,
  release: ReleaseLine,
): Issue[] => {
  const { runtime, version } = runtimeDefault;
  const location =
    "`latest_runtime_version` in `qlty-check/src/tool/tool_builder.rs`";
  const link = release.link ? `\n\nRelease notes: ${release.link}` : "";
  const issues: Issue[] = [];

  if (release.isEol) {
    issues.push({
      key: `${runtime}-eol`,
      title: `Default ${runtime} runtime is on unsupported line ${line}`,
      body: `The default ${runtime} runtime in ${location} is ${version}, on the ${line} line. Upstream support for ${line} ended on ${release.eolFrom}, so it gets no more security releases.`,
    });
  }
  // A default with only a release line, such as PHP's, has no patch to bump.
  if (version !== line && isOlder(version, release.latest)) {
    issues.push({
      key: `${runtime}-behind`,
      title: `Default ${runtime} runtime ${version} is behind ${release.latest}`,
      body: `The default ${runtime} runtime in ${location} is ${version}. ${runtime} ${release.latest} is the latest release on the ${line} line.${link}`,
    });
  }
  return issues;
};

const fetchJson = async (url: string, headers: Record<string, string> = {}) => {
  const response = await fetch(url, {
    headers,
    signal: AbortSignal.timeout(30_000),
  });
  if (!response.ok) {
    throw new Error(`GET ${url} failed: ${response.status}`);
  }
  return response.json();
};

const fetchReleaseLine = async (
  product: string,
  line: string,
): Promise<ReleaseLine> => {
  const { result } = await fetchJson(
    `https://endoflife.date/api/v1/products/${product}/releases/${line}`,
  );
  return {
    isEol: result.isEol === true,
    eolFrom: result.eolFrom,
    latest: result.latest.name,
    link: result.latest.link,
  };
};

const rubyBuildIssues = async (): Promise<Issue[]> => {
  const source = fs.readFileSync(RUBY_SOURCE_PATH, "utf8");
  const pinned = source.match(
    /rbenv\/ruby-build\/archive\/([0-9a-f]{40})/,
  )?.[1];
  if (!pinned) {
    throw new Error("ruby-build commit pin not found in ruby_source.rs");
  }
  const headers: Record<string, string> = process.env.GITHUB_TOKEN
    ? { Authorization: `Bearer ${process.env.GITHUB_TOKEN}` }
    : {};
  const api = "https://api.github.com/repos/rbenv/ruby-build";
  const { tag_name: tag } = await fetchJson(`${api}/releases/latest`, headers);
  const commit = await fetchJson(`${api}/commits/${tag}`, headers);
  if (commit.sha === pinned) return [];
  return [
    {
      key: "ruby-build-behind",
      title: `Pinned ruby-build is behind ${tag}`,
      body: `\`qlty-check/src/tool/ruby_source.rs\` pins ruby-build at commit ${pinned}, which is not the latest release, ${tag}. Source builds of new Ruby patch releases need the newer ruby-build.\n\nRelease notes: https://github.com/rbenv/ruby-build/releases/tag/${tag}`,
    },
  ];
};

const main = async () => {
  const defaults = parseRuntimeDefaults(
    fs.readFileSync(TOOL_BUILDER_PATH, "utf8"),
  );
  const checks: Array<() => Promise<Issue[]>> = defaults.map(
    (runtimeDefault) => async () => {
      const known = PRODUCTS[runtimeDefault.runtime];
      if (!known) {
        throw new Error(
          `No endoflife.date product for ${runtimeDefault.runtime}`,
        );
      }
      const line = releaseLine(runtimeDefault.version, known.lineParts);
      const release = await fetchReleaseLine(known.product, line);
      return issuesForRuntime(runtimeDefault, line, release);
    },
  );
  checks.push(rubyBuildIssues);

  // One failed lookup shouldn't hide the other findings: file what was found,
  // then fail so the run shows the error.
  const issues: Issue[] = [];
  let failed = false;
  for (const check of checks) {
    try {
      issues.push(...(await check()));
    } catch (error) {
      console.error(error);
      failed = true;
    }
  }

  for (const issue of issues) console.log(issue.title);
  fs.writeFileSync(OUTPUT_PATH, JSON.stringify(issues, null, 2));
  if (failed) process.exitCode = 1;
};

if (require.main === module) {
  main().catch((error) => {
    console.error(error);
    process.exit(1);
  });
}
