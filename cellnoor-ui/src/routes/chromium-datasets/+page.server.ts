import { getApiClient } from "$lib/server/cellnoor-client";
import type { ApiErrorResponse, ChromiumDatasetSummary, Project, TenxAssay } from "cellnoor-client";

type ReturnType =
  | {
      chromiumDatasets: (ChromiumDatasetSummary & {
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

  for (const ds of chromiumDatasets.data) {
    // @ts-expect-error I hate this language
    ds.files = createFileTree(ds.links.web_summaries as string[], ds.links.metrics as string[]);
  }

  return {
    // @ts-expect-error I hate this language
    chromiumDatasets: chromiumDatasets.data,
    assays: assays.data,
    projects: projects.data,
  };
}

function createFileTree(webSummaries: string[], metricsFiles: string[]) {
  const linkMap: Map<string, string[]> = new Map();

  for (const link of webSummaries.concat(metricsFiles)) {
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
