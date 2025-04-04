import fetch from "node-fetch";
import { z } from "zod";

const PackagistResponseSchema = z.object({
  packages: z.record(
    z.string(),
    z.array(
      z.object({
        version: z.string(),
      }),
    ),
  ),
});

export async function fetchLatestVersionForPhp(
  phpPackage: string,
): Promise<string> {
  const url = `https://repo.packagist.org/p2/${phpPackage}.json`;

  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(
      `Failed to fetch from Packagist, status: ${response.status}`,
    );
  }

  const rawJson = await response.json();

  const parsed = PackagistResponseSchema.safeParse(rawJson);
  if (!parsed.success) {
    throw new Error(`Invalid Packagist response: ${parsed.error.message}`);
  }

  const versionString = parsed.data.packages[phpPackage]?.[0]?.version;

  if (!versionString) {
    throw new Error("Version not found in the response");
  }

  return versionString;
}
