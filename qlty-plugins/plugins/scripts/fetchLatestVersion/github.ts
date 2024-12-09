import fetch from "node-fetch";

const GITHUB_API_VERSION = "2022-11-28";
const VERSION_REGEX = /(\d+\.\d+\.\d+)/;

export async function fetchLatestVersionFromGithub(
  githubRepo: string,
): Promise<string> {
  const url = `https://api.github.com/repos/${githubRepo}/releases/latest`;

  const response = await fetch(url, {
    method: "GET",
    headers: {
      "X-GitHub-Api-Version": GITHUB_API_VERSION,
    },
  });

  if (!response.ok) {
    throw new Error(`GitHub API request failed with status ${response.status}`);
  }

  const data = (await response.json()) as { tag_name: string };

  const tag = data.tag_name as string;
  if (!tag) {
    throw new Error("No tag_name found");
  }

  const match = tag.match(VERSION_REGEX);
  if (match) {
    return match[1];
  } else {
    throw new Error("Version not found in the response");
  }
}
