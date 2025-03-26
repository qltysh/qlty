import { execFile, ExecOptions } from "child_process";
import Debug, { Debugger } from "debug";
import * as fs from "fs";
import * as os from "os";
import path from "path";
import * as git from "simple-git";
import * as util from "util";
import { getKnownGoodVersion } from "./runLinterTest";
import { OPTIONS } from "./utils";

interface Issue {
  tool: string;
  ruleKey: string;
  path: string;
  message: string;
}

const execFilePromise = util.promisify(execFile);

const FIXTURES_DIR = "fixtures";
const TEMP_PREFIX = "plugins_";
const TEMP_SUBDIR = ".qlty/tmp";
const SNAPSHOTS_DIR = "__snapshots__";
export const REPO_ROOT = path.resolve(__dirname, "..");

let TMPDIR = os.tmpdir();
if (process.platform === "win32") {
  // Manually override on Windows to avoid using 8.3 style paths, which throw off some snapshot comparisons
  // when scrubbing the sandbox path from the output. See: https://stackoverflow.com/questions/56620398
  TMPDIR = path.join(process.env.LOCALAPPDATA!, "Temp");
}

export const executionEnv = (sandbox: string) => {
  const { PWD, INIT_CWD, ...strippedEnv } = process.env;
  return {
    ...strippedEnv,
    QLTY_TELEMETRY: "off",
    // This is necessary to prevent launcher collision of non-atomic operations
    TMPDIR: path.resolve(sandbox, TEMP_SUBDIR),
    TEMP: path.resolve(sandbox, TEMP_SUBDIR),
    PATH: [
      path.resolve(REPO_ROOT, "..", "..", "target", "debug"),
      process.env.PATH,
    ].join(path.delimiter),
  };
};

const testCreationFilter = (input: string) => (file: string) => {
  // Don't copy snapshot files
  if (file.endsWith(".shot")) {
    return false;
  }

  // Only copy the input file if it matches the target
  const name = path.basename(file) || "";
  if (name.includes(".in") && input != name) {
    return false;
  }

  return true;
};

export class QltyDriver {
  fixturesDir: string;
  sandboxPath: string;
  linterName: string;
  linterVersion: string;
  pluginDir: string;
  debug: Debugger;

  constructor(linterName: string, linterVersion: string) {
    this.pluginDir = path.resolve(REPO_ROOT, "linters", linterName);
    this.fixturesDir = path.resolve(this.pluginDir, FIXTURES_DIR);
    this.linterName = linterName;
    this.linterVersion = linterVersion;
    this.sandboxPath = fs.realpathSync(
      fs.mkdtempSync(path.resolve(TMPDIR, TEMP_PREFIX)),
    );
    this.debug = Debug(`qlty:${linterName}`);
  }

  async setUp(input: string) {
    fs.mkdirSync(path.resolve(this.sandboxPath, TEMP_SUBDIR), {
      recursive: true,
    });
    this.debug(
      "Created sandbox %s from %s",
      this.sandboxPath,
      this.fixturesDir,
    );

    const input_path = path.join(this.fixturesDir, input);

    // Copy contents of input if it is a directory, otherwise copy the input file
    const stats = fs.statSync(input_path);
    if (stats.isDirectory()) {
      fs.cpSync(input_path, this.sandboxPath, {
        recursive: true,
      });
    } else {
      fs.cpSync(this.fixturesDir, this.sandboxPath, {
        recursive: true,
        filter: testCreationFilter(input),
      });
    }

    if (!fs.existsSync(path.resolve(path.resolve(this.sandboxPath, ".qlty")))) {
      fs.mkdirSync(path.resolve(this.sandboxPath, ".qlty"), {});
    }

    fs.writeFileSync(
      path.resolve(this.sandboxPath, ".gitignore"),
      this.getGitIgnoreContents(),
    );

    const gitDriver = git.simpleGit(this.sandboxPath);
    await gitDriver
      .init({ "--initial-branch": "main" })
      .add(".")
      .addConfig("user.name", "User")
      .addConfig("user.email", "user@example.com")
      .addConfig("commit.gpgsign", "false")
      .addConfig("core.autocrlf", "input")
      .commit("first commit");

    await this.runQlty(["--help"]);

    const qltyTomlPath = path.resolve(this.sandboxPath, ".qlty", "qlty.toml");
    const qltyTomlExists = fs.existsSync(qltyTomlPath);
    if (!qltyTomlExists) {
      fs.writeFileSync(qltyTomlPath, this.qltyTomlConfigVersion());
    }
    fs.appendFileSync(qltyTomlPath, this.qltyTomlSource());
    if (!qltyTomlExists) {
      const linterVersion = OPTIONS.linterVersion
        ? OPTIONS.linterVersion
        : this.linterVersion;

      await this.runQltyCmd(
        `plugins enable ${this.linterName}=${linterVersion}`,
      );
    }
  }

  tearDown() {
    if (this.sandboxPath && !OPTIONS.sandboxDebug) {
      this.debug("Cleaning up %s", this.sandboxPath);
      fs.rmSync(this.sandboxPath, { recursive: true });
    } else {
      this.debug("Leaving sandbox %s", this.sandboxPath);
    }
  }

  testTargets(): string[] {
    return fs
      .readdirSync(this.fixturesDir)
      .sort()
      .filter(
        (target) => !target.includes(SNAPSHOTS_DIR) && !target.startsWith("."),
      );
  }

  snapshotPath(prefix: string): string {
    if (OPTIONS.testAgainstKnownGoodVersion) {
      const knownGoodVersion = getKnownGoodVersion(
        this.pluginDir,
        this.linterName,
      );
      const knownGoodSnapshot = path.resolve(
        this.fixturesDir,
        SNAPSHOTS_DIR,
        `${prefix}_v${knownGoodVersion}.shot`,
      );

      return knownGoodSnapshot;
    }

    const snapshotName = `${prefix}_v${this.linterVersion}.shot`;
    return path.resolve(this.fixturesDir, SNAPSHOTS_DIR, snapshotName);
  }

  async runCheck() {
    const fullArgs = `check --all --json --no-fail --no-cache --no-progress --filter=${this.linterName}`;

    let output = { stdout: "", stderr: "" };
    let exitCode = 0;
    try {
      const env = {
        ...executionEnv(this.sandboxPath ?? ""),
        QLTY_LOG_STDERR: "1",
        QLTY_LOG: process.env.QLTY_LOG ?? "debug",
      };

      output = await this.runQltyCmd(fullArgs, { env });
    } catch (error) {
      const err = error as {
        code: number;
        stdout?: string;
        stderr?: string;
      };
      output = { stdout: err.stdout ?? "", stderr: err.stderr ?? "" };
      exitCode = err.code;
    }

    let outputJson = [];
    try {
      outputJson = JSON.parse(output.stdout);
    } catch {
      /* empty */
    }

    return this.parseRunResult({
      exitCode,
      outputJson,
      ...output,
    });
  }

  async runQltyCmd(
    argStr: string,
    execOptions?: ExecOptions,
  ): Promise<{ stdout: string; stderr: string }> {
    this.debug("Running qlty %s", argStr);
    return await this.runQlty(
      argStr.split(" ").filter((arg) => arg.length > 0),
      execOptions,
    );
  }

  async runQlty(
    args: string[],
    execOptions?: ExecOptions,
  ): Promise<{ stdout: string; stderr: string }> {
    return await execFilePromise(...this.buildExecArgs(args, execOptions));
  }

  buildExecArgs(
    args: string[],
    execOptions?: ExecOptions,
  ): [string, string[], ExecOptions] {
    return [
      "qlty",
      args.filter((arg) => arg.length > 0),
      {
        cwd: this.sandboxPath,
        ...execOptions,
        windowsHide: true,
      },
    ];
  }

  parseRunResult(runResult: {
    stdout: string;
    stderr: string;
    exitCode: number;
    outputJson: Issue[];
  }) {
    return {
      success: [0].includes(runResult.exitCode),
      runResult,
      deterministicResults: this.tryParseDeterministicResults(
        this.sandboxPath,
        runResult.outputJson,
      ),
    };
  }

  tryParseDeterministicResults(sandboxPath: string, outputJson: Issue[]) {
    // return function to lazy evaluate sorting and skip if not needed
    return () => {
      if (!outputJson) {
        return undefined;
      }

      outputJson.sort((a, b) => {
        if (a.tool < b.tool) {
          return -1;
        }
        if (a.tool > b.tool) {
          return 1;
        }
        if (a.ruleKey < b.ruleKey) {
          return -1;
        }
        if (a.ruleKey > b.ruleKey) {
          return 1;
        }
        if (a.path < b.path) {
          return -1;
        }
        if (a.path > b.path) {
          return 1;
        }
        if (a.message < b.message) {
          return -1;
        }
        if (a.message > b.message) {
          return 1;
        }
        return 0;
      });

      // make results deterministic by removing the sandbox path
      outputJson.forEach((issue: Issue) => {
        if (issue.message.includes(sandboxPath)) {
          issue.message = issue.message.replace(sandboxPath, "");
        }
      });

      return {
        issues: outputJson,
      };
    };
  }

  qltyTomlConfigVersion() {
    return `config_version = "0"\n`;
  }

  qltyTomlSource() {
    return `[[source]]
name = "default"
default = true
`;
  }

  getGitIgnoreContents(): string {
    return `.qlty/logs/
.qlty/out/
.qlty/tmp/
`;
  }
}
