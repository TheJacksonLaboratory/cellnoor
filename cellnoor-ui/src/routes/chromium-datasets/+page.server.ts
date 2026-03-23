import { getApiClient } from "$lib/server/cellnoor-client";
import type { ApiErrorResponse, ChromiumDataset, Project, TenxAssay } from "cellnoor-client";

type ReturnType =
  | {
      chromiumDatasets: (ChromiumDataset & {
        files: Map<string, string[]>;
      })[];
      assays: TenxAssay[];
      projects: Project[];
    }
  | { error: ApiErrorResponse };

export async function load({ url }) {
  return await loadData(url.searchParams.get("q") || `{"limit": 5000}`);
}

export const actions = {
  search: async ({ request }) => {
    const formData = await request.formData();
    return await loadData(formData.get("q")?.toString());
  },
};

async function loadData(q?: string): Promise<ReturnType> {
  const apiClient = await getApiClient();

  const [chromiumDatasets, assays, projects] = await Promise.all([
    apiClient.GET("/chromium-datasets", {
      params: {
        query: {
          q,
        },
      },
    }),
    apiClient.GET("/10x-assays"),
    apiClient.GET("/projects"),
  ]);

  if (!chromiumDatasets.data || !assays.data || !projects.data) {
    return {
      error: chromiumDatasets.error ||
        assays.error || {
          error: { type: "other", message: "something went wrong :(" },
        },
    };
  }

  // In theory, mutating each dataset and just adding the `files` property would be more performant, as this probably
  // involves a copy, but I don't think it matters.
  return {
    chromiumDatasets: chromiumDatasets.data.map((ds) => {
      return { files: createFileTree(ds.links.files), ...ds };
    }),
    assays: assays.data,
    projects: projects.data,
  };
}

function createFileTree(fileLinks: (string | null)[]) {
  const linkMap: Map<string, string[]> = new Map();

  for (const link of fileLinks) {
    if (link === null) {
      continue;
    }

    const parts = link.split("/");

    const directoryName = parts.at(-2) as string;

    const existingLinks = linkMap.get(directoryName);
    if (existingLinks) {
      existingLinks.push(link);
    } else {
      linkMap.set(directoryName, [link]);
    }
  }

  return linkMap;
}
