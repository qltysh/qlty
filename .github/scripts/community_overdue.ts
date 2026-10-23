// Finds open issues, pull requests, and discussions opened by people outside
// the org that are still waiting on a reply from the team, and posts the
// overdue ones to Slack. Runs with no dependencies on Node 24.

const REPO_OWNER = "qltysh";
const REPO_NAME = "qlty";
const TEAM_ASSOCIATIONS = new Set(["OWNER", "MEMBER", "COLLABORATOR"]);

const token = process.env.GITHUB_TOKEN;
const webhook = process.env.SLACK_WEBHOOK;
const slaBusinessDays = Number(process.env.SLA_BUSINESS_DAYS ?? "2");

if (!token) {
  throw new Error("GITHUB_TOKEN is required");
}

type Actor = { login: string; __typename: string } | null;

type Comment = {
  author: Actor;
  authorAssociation: string;
  createdAt: string;
};

type Item = {
  kind: "issue" | "pull request" | "discussion";
  number: number;
  title: string;
  url: string;
  author: Actor;
  authorAssociation: string;
  createdAt: string;
  comments: Comment[];
};

type Overdue = Item & { waitingSince: string; businessDays: number };

const ACTOR = "author { login __typename }";
const COMMENT = `{ ${ACTOR} authorAssociation createdAt }`;

const ISSUES_QUERY = `
  query($owner: String!, $name: String!, $after: String) {
    repository(owner: $owner, name: $name) {
      issues(first: 50, states: OPEN, after: $after) {
        pageInfo { hasNextPage endCursor }
        nodes {
          number title url createdAt ${ACTOR} authorAssociation
          comments(last: 30) { nodes ${COMMENT} }
        }
      }
    }
  }`;

const PULLS_QUERY = `
  query($owner: String!, $name: String!, $after: String) {
    repository(owner: $owner, name: $name) {
      pullRequests(first: 50, states: OPEN, after: $after) {
        pageInfo { hasNextPage endCursor }
        nodes {
          number title url createdAt ${ACTOR} authorAssociation
          comments(last: 30) { nodes ${COMMENT} }
          reviews(last: 30) { nodes ${COMMENT} }
        }
      }
    }
  }`;

const DISCUSSIONS_QUERY = `
  query($owner: String!, $name: String!, $after: String) {
    repository(owner: $owner, name: $name) {
      discussions(first: 50, states: OPEN, after: $after) {
        pageInfo { hasNextPage endCursor }
        nodes {
          number title url createdAt ${ACTOR} authorAssociation
          comments(last: 30) {
            nodes {
              ${ACTOR} authorAssociation createdAt
              replies(last: 30) { nodes ${COMMENT} }
            }
          }
        }
      }
    }
  }`;

async function graphql(query: string, variables: Record<string, unknown>) {
  const response = await fetch("https://api.github.com/graphql", {
    method: "POST",
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
      "User-Agent": "qlty-community-inbox",
    },
    body: JSON.stringify({ query, variables }),
  });

  if (!response.ok) {
    throw new Error(
      `GitHub GraphQL request failed: ${response.status} ${await response.text()}`,
    );
  }

  const body = (await response.json()) as { data?: unknown; errors?: unknown };

  if (body.errors) {
    throw new Error(`GitHub GraphQL errors: ${JSON.stringify(body.errors)}`);
  }

  return body.data as Record<string, { [field: string]: Page }>;
}

type Page = {
  pageInfo: { hasNextPage: boolean; endCursor: string | null };
  nodes: Record<string, unknown>[];
};

async function fetchAll(
  query: string,
  field: string,
): Promise<Record<string, unknown>[]> {
  const nodes: Record<string, unknown>[] = [];
  let after: string | null = null;

  for (;;) {
    const data = await graphql(query, {
      owner: REPO_OWNER,
      name: REPO_NAME,
      after,
    });
    const page = data.repository[field];
    nodes.push(...page.nodes);

    if (!page.pageInfo.hasNextPage) {
      return nodes;
    }

    after = page.pageInfo.endCursor;
  }
}

function isBot(actor: Actor): boolean {
  return (
    actor === null ||
    actor.__typename === "Bot" ||
    actor.login.endsWith("[bot]")
  );
}

function isTeam(association: string): boolean {
  return TEAM_ASSOCIATIONS.has(association);
}

function commentsOf(node: Record<string, unknown>): Comment[] {
  const comments: Comment[] = [];

  for (const field of ["comments", "reviews"]) {
    const connection = node[field] as
      | { nodes: (Comment & { replies?: { nodes: Comment[] } })[] }
      | undefined;

    for (const comment of connection?.nodes ?? []) {
      comments.push(comment);
      comments.push(...(comment.replies?.nodes ?? []));
    }
  }

  return comments
    .filter((comment) => !isBot(comment.author))
    .sort((a, b) => a.createdAt.localeCompare(b.createdAt));
}

function toItem(kind: Item["kind"], node: Record<string, unknown>): Item {
  return {
    kind,
    number: node.number as number,
    title: node.title as string,
    url: node.url as string,
    author: node.author as Actor,
    authorAssociation: node.authorAssociation as string,
    createdAt: node.createdAt as string,
    comments: commentsOf(node),
  };
}

function businessDaysSince(iso: string, now: Date): number {
  const start = new Date(iso);
  let count = 0;

  for (
    let day = new Date(start);
    day.getTime() + 86_400_000 <= now.getTime();
    day.setUTCDate(day.getUTCDate() + 1)
  ) {
    const weekday = day.getUTCDay();

    if (weekday !== 0 && weekday !== 6) {
      count += 1;
    }
  }

  return count;
}

function waitingSince(item: Item): string | null {
  const last = item.comments.at(-1);

  if (!last) {
    return item.createdAt;
  }

  return isTeam(last.authorAssociation) ? null : last.createdAt;
}

function escapeSlack(text: string): string {
  return text
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

const MAX_LISTED = 10;

function reference(item: Item): string {
  if (item.kind === "discussion") {
    return `${REPO_OWNER} discussion #${item.number}`;
  }

  return `${REPO_OWNER}/${REPO_NAME}#${item.number}`;
}

function formatMessage(overdue: Overdue[]): string {
  const noun =
    overdue.length === 1 ? "community item is" : "community items are";
  const listed = overdue.slice(0, MAX_LISTED);
  const lines = listed.map((item) => {
    const days =
      item.businessDays === 1
        ? "1 business day"
        : `${item.businessDays} business days`;
    const author = item.author?.login ?? "unknown";
    return `• ${item.kind} <${item.url}|${reference(item)}> by ${author}, waiting ${days}: ${escapeSlack(item.title)}`;
  });

  if (overdue.length > listed.length) {
    const rest = overdue.slice(listed.length);
    const oldest = rest.at(-1)!;
    lines.push(
      `• and ${rest.length} more, the oldest waiting ${oldest.businessDays} business days`,
    );
  }

  return [
    `*${overdue.length} ${noun} waiting for a reply from the team, newest first*`,
    ...lines,
  ].join("\n");
}

async function main() {
  const now = new Date();
  const [issues, pulls, discussions] = await Promise.all([
    fetchAll(ISSUES_QUERY, "issues"),
    fetchAll(PULLS_QUERY, "pullRequests"),
    fetchAll(DISCUSSIONS_QUERY, "discussions"),
  ]);

  const items = [
    ...issues.map((node) => toItem("issue", node)),
    ...pulls.map((node) => toItem("pull request", node)),
    ...discussions.map((node) => toItem("discussion", node)),
  ];

  const community = items.filter(
    (item) => !isBot(item.author) && !isTeam(item.authorAssociation),
  );
  const overdue: Overdue[] = [];

  for (const item of community) {
    const since = waitingSince(item);

    if (since === null) {
      continue;
    }

    const businessDays = businessDaysSince(since, now);

    if (businessDays >= slaBusinessDays) {
      overdue.push({ ...item, waitingSince: since, businessDays });
    }
  }

  overdue.sort((a, b) => a.businessDays - b.businessDays);

  console.log(
    `Open items: ${items.length} (${community.length} from outside the team)`,
  );
  console.log(
    `Overdue after ${slaBusinessDays} business days: ${overdue.length}`,
  );

  if (overdue.length === 0) {
    return;
  }

  const text = formatMessage(overdue);
  console.log(text);

  if (!webhook) {
    console.log("SLACK_WEBHOOK is not set, so nothing was posted to Slack");
    return;
  }

  const response = await fetch(webhook, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ text }),
  });

  if (!response.ok) {
    throw new Error(
      `Slack webhook request failed: ${response.status} ${await response.text()}`,
    );
  }
}

await main();
