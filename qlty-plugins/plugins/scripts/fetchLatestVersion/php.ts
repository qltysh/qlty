import fetch from "node-fetch";

export async function fetchLatestVersionForPhp(
  phpPackage: string,
): Promise<string> {
  const url = `https://repo.packagist.org/p2/${phpPackage}.json`;

  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to fetch from PyPI, status: ${response.status}`);
  }

  const json = (await response.json()) as {
    packages: Record<string, { version: string }[]>;
  };
  const versionString = json.packages[phpPackage]?.[0]?.version;

  if (!versionString) {
    throw new Error("Version not found in the response");
  }

  return versionString;
}
