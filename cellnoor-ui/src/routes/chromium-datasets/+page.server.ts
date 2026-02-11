import { getApiClient } from "$lib/server/cellnoor-client";
import type {
  ApiError,
  ChromiumDatasetSummary,
  TenxAssay,
} from "cellnoor-client";

type ReturnType =
  | {
    chromiumDatasets:
      (ChromiumDatasetSummary & { files: Map<string, string[]> })[];
    assays: TenxAssay[];
  }
  | { error: ApiError };

export async function load({ url }) {
  return await loadData(url.searchParams.get("q") || undefined);
}

export const actions = {
  search: async ({ request }) => {
    const formData = await request.formData();
    return await loadData(formData.get("q")?.toString());
  },
};

async function loadData(q?: string): Promise<ReturnType> {
  const apiClient = await getApiClient();

  const [chromiumDatasets, assays] = await Promise.all([
    apiClient.GET("/chromium-datasets", {
      params: {
        query: {
          q,
        },
      },
    }),
    apiClient.GET("/10x-assays"),
  ]);

  if (!chromiumDatasets.data || !assays.data) {
    return {
      error: chromiumDatasets.error ||
        (assays.error || { type: "other", message: "something went wrong :(" }),
    };
  }

  for (const ds of chromiumDatasets.data) {
    // @ts-ignore
    ds.files = createFileTree(
      ds.links["web-summaries"] as string[],
      ds.links["metrics-files"] as string[],
    );
  }

  return {
    // @ts-ignore
    chromiumDatasets: chromiumDatasets.data,
    assays: assays.data,
  };
}

function createFileTree(webSummaries: string[], metricsFiles: string[]) {
  const linkMap: Map<string, string[]> = new Map();

  for (
    const link of webSummaries.concat(
      metricsFiles,
    )
  ) {
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
