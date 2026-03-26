import { getApiClient } from "$lib/server/cellnoor-client.js";
import { downloadFile } from "$lib/server/download-chromium-dataset-file.js";
import { createFileTree, type FileNode } from "$lib/file-tree.js";

export async function load({ params: { id } }) {
  const apiClient = await getApiClient();

  const params = { path: { id } };

  const [dataset, specimens, libraries] = await Promise.all([
    apiClient.GET("/chromium-datasets/{id}", {
      params,
    }),
    apiClient.GET("/chromium-datasets/{id}/specimens", { params }),
    apiClient.GET("/chromium-datasets/{id}/libraries", { params }),
  ]);

  // @ts-expect-error we know that there are no null links
  const downloadedFiles = await Promise.all(dataset.data!.links.files.map(downloadParsedFile));
  const fileTree = createFileTree(downloadedFiles);

  return {
    dataset: dataset.data,
    fileTree,
    specimens: specimens.data,
    libraries: libraries.data,
  };
}

const FILE_LINK_PREFIX = "/chromium-datasets/00000000-0000-0000-0000-000000000000/files/";

interface DownloadedFile extends FileNode {
  name: string;
  content: Record<string, unknown> | Record<string, unknown>[] | Blob;
  type: "json" | "html" | "unknown";
}

async function downloadParsedFile(link: string): Promise<DownloadedFile> {
  // If it's not an HTML file, we know the backend (written by a virtuoso) has a JSON representation of the file, which we want for a nice (shitty) data display
  const downloadHtml = link.endsWith(".html");
  const accept = downloadHtml ? "text/html" : "application/json";

  const response = await downloadFile({ link, accept });

  const name = link.slice(FILE_LINK_PREFIX.length);
  const contentType = response.headers.get("Content-Type");

  if (contentType === "application/json") {
    return { name, content: await response.json(), type: "json" };
  } else if (contentType === "text/html") {
    return { name, content: await response.blob(), type: "html" };
  } else {
    return { name, content: await response.blob(), type: "unknown" };
  }
}
