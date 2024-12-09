import { execSync } from "child_process";

export async function fetchLatestVersionForNode(
  nodePackage: string,
): Promise<string> {
  const cmd = `npm view ${nodePackage} version`;

  const output: string = execSync(cmd, { encoding: "utf8" });
  const versionString = output.trim();

  return versionString;
}
