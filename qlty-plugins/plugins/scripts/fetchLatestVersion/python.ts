import fetch from "node-fetch";
import { z } from "zod";

const PyPiResponseSchema = z.object({
  info: z.object({
    version: z.string(),
  }),
});

export async function fetchLatestVersionForPython(
  pipPackage: string,
): Promise<string> {
  const url = `https://pypi.org/pypi/${pipPackage}/json`;

  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to fetch from PyPI, status: ${response.status}`);
  }

  const rawJson = await response.json();

  const parsed = PyPiResponseSchema.safeParse(rawJson);
  if (!parsed.success) {
    throw new Error(`Invalid PyPI response: ${parsed.error.message}`);
  }

  return parsed.data.info.version;
}
