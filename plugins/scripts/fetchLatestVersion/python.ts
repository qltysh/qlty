import fetch from "node-fetch";

export async function fetchLatestVersionForPython(
  pipPackage: string,
): Promise<string> {
  const url = `https://pypi.org/pypi/${pipPackage}/json`;

  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to fetch from PyPI, status: ${response.status}`);
  }

  const json = await response.json();
  const versionString = json.info?.version as string;

  if (!versionString) {
    throw new Error("Version not found in the response");
  }

  return versionString;
}
