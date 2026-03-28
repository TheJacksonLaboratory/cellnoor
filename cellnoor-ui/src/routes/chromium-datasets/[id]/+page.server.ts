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

async function downloadParsedFile(link: string): Promise<FileNode> {
  const isHtml = link.endsWith(".html");
  const name = link.slice(FILE_LINK_PREFIX.length);

  // The HTML files are 10x Genomics web summaries, which are relatively large, so we download them on-demand in the browser
  if (isHtml) {
    return {
      name,
      src: link,
      type: "html",
    };
  }

  // At this point, we know that the file isn't HTML, so the backend can represent it as JSON
  const content = await downloadFile({ link, accept: "application/json" }).then((r) => r.json());

  return { name, content, type: "json" };
}
