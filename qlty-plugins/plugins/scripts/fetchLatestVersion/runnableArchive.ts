import { fetchLatestVersionFromGithub } from "./github";

function extractRepo(url: string): string | null {
  const re = /https:\/\/github\.com\/([^/]+\/[^/]+)\/releases\/download\//;
  const match = url.match(re);
  return match ? match[1] : null;
}

export async function fetchLatestVersionForRunnableArchive(
  runnable_archive_url: string,
): Promise<string> {
  const githubRepo = extractRepo(runnable_archive_url);
  if (!githubRepo) {
    throw new Error("Cannot fetch latest version, not a GitHub repo");
  }

  return await fetchLatestVersionFromGithub(githubRepo);
}
