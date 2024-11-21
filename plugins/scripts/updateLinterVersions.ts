import { execSync, ExecException } from "child_process";
import fs from "fs";
import path from "path";
import toml from "toml";
import { fetchLatestVersion } from "./fetchLatestVersion/fetchLatestVersion";

const REPO_ROOT: string = path.resolve(__dirname, "..");
const LINTERS_PATH: string = path.resolve(REPO_ROOT, "linters");

// Return type for getLinterDef
interface LinterDefinition {
  plugins: {
    releases: {
      [key: string]: {
        github: string;
      };
    };
    definitions: {
      [key: string]: {
        known_good_version: string;
        releases: string[];
        runtime?: string;
        runnable_archive_url?: string;
        package?: string;
      };
    };
  };
}

// Return type for GitHub issues
interface GitHubIssue {
  title: string;
  body: string;
  assignees: string[];
}

const getLintersList = async (): Promise<string[]> => {
  if (process.env.LINTER) {
    return [process.env.LINTER];
  }

  try {
    const dirContents = await fs.promises.readdir(LINTERS_PATH);
    const folders = await Promise.all(
      dirContents.filter(async (dirContent) => {
        const linterPath = path.join(LINTERS_PATH, dirContent);
        return (await fs.promises.stat(linterPath)).isDirectory();
      }),
    );
    return folders;
  } catch (err) {
    throw new Error(`Failed to read linters directory: ${err}`);
  }
};

const getLinterTomlPath = (linter: string): string => {
  return path.resolve(LINTERS_PATH, linter, "plugin.toml");
};

const getKnownGoodVersion = (linterName: string): string => {
  const linterDef = getLinterDef(linterName);
  return linterDef.plugins.definitions[linterName].known_good_version;
};

const updateLinterTomlVersions = (
  linterName: string,
  latestLinterVersion: string,
  updateKnownGoodVersion: boolean,
): void => {
  const tomlPath = getLinterTomlPath(linterName);
  let linterToml = fs.readFileSync(tomlPath, "utf8");

  // Preserve the formatting and comments in the file
  linterToml = linterToml.replace(
    /(latest_version\s*=\s*)".*"/,
    `$1"${latestLinterVersion}"`,
  );

  if (updateKnownGoodVersion) {
    linterToml = linterToml.replace(
      /(known_good_version\s*=\s*)".*"/,
      `$1"${latestLinterVersion}"`,
    );
  }

  fs.writeFileSync(tomlPath, linterToml);
};

export const getLinterDef = (linter: string): LinterDefinition => {
  const linterFile = fs.readFileSync(getLinterTomlPath(linter), "utf8");
  return toml.parse(linterFile);
};

async function main(): Promise<void> {
  const githubIssues: GitHubIssue[] = [];
  const successfulLinters: string[] = [];
  const failedLinter: Map<string, string> = new Map();
  const latestLinters: string[] = [];

  try {
    const linters = await getLintersList();

    for (const linter of linters) {
      let latestLinterVersion: string;

      try {
        latestLinterVersion = await fetchLatestVersion(linter);
      } catch (error) {
        console.error(
          `Failed to get the latest version for ${linter}. Skipping...`,
        );
        console.error(error);
        failedLinter.set(linter, error as string);
        continue; // Move to the next linter
      }

      const linterLabel = `${linter}:v${latestLinterVersion}`;
      const currentKnownGoodVersion = getKnownGoodVersion(linter);

      if (currentKnownGoodVersion === latestLinterVersion) {
        console.log(
          `The linter ${linter} is already at the latest version: ${latestLinterVersion}. Skipping...`,
        );
        latestLinters.push(linter);
        continue; // Move to the next linter
      }

      try {
        console.log(`Testing ${linterLabel}...`);
        execSync(
          `QLTY_PLUGINS_LINTER_VERSION=${latestLinterVersion} QLTY_PLUGINS_TEST_AGAINST_KNOWN_GOOD_VERSION=true npm test ${linter}.test.ts`,
          { stdio: "inherit" },
        );

        execSync(
          `QLTY_PLUGINS_LINTER_VERSION=${latestLinterVersion} npm test ${linter}.test.ts -- --updateSnapshot`,
          { stdio: "inherit" },
        );

        console.log(`Yay! ${linterLabel} passed the tests!`);
        updateLinterTomlVersions(linter, latestLinterVersion, true);
        successfulLinters.push(linter);
      } catch (error) {
        console.error(`Tests failed for ${linterLabel}.`);
        console.error(error);

        updateLinterTomlVersions(linter, latestLinterVersion, false);

        failedLinter.set(linter, error as string);

        githubIssues.push({
          title: `Tests failed for ${linterLabel}`,
          body: `Error encountered:\n${error}`,
          assignees: ["marschattha"],
        });
      }
    }

    fs.writeFileSync(
      path.resolve(REPO_ROOT, "github_issues.json"),
      JSON.stringify(githubIssues, null, 2),
    );

    console.log("Successfully updated the following linters:");
    console.log(successfulLinters);

    console.log("Linters already at the latest version:");
    console.log(latestLinters);

    console.log("Failed to update the following linters:");
    console.log(failedLinter);
  } catch (error) {
    console.error(`Error during processing: ${error}`);
  }
}

main();
