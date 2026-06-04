import { getApiClient } from "$lib/server/cellnoor-client.js";
import { downloadFile } from "$lib/server/download-chromium-dataset-file.js";
import { createFileTree, type FileNode } from "$lib/file-tree.js";

export async function load({ params: { id } }) {
  const apiClient = await getApiClient();

  const params = { path: { id } };

  const [dataset, specimens] = await Promise.all([
    apiClient.GET("/chromium-datasets/{id}", {
      params,
    }),
    apiClient.GET("/chromium-datasets/{id}/specimens", { params }),
    // apiClient.GET("/chromium-datasets/{id}/libraries", { params }),
  ]);

  // This is ugly because TypeScript's type-inference is shit
  const downloadedFiles = await Promise.all(
    dataset
      .data!.links.parsed_files.filter((link) => link !== null)
      .map(downloadParsedFile)
      .concat(
        dataset
          .data!.links.raw_files.filter((link) => link !== null)
          .filter((link) => link.endsWith(".html"))
          .map(createRawFileNode),
      ),
  );
  const fileTree = createFileTree(downloadedFiles);

  if (!dataset.data) {
    return { error: dataset.error };
  }

  return {
    dataset: dataset.data,
    fileTree,
    specimens: specimens.data,
  };
}

const RAW_FILE_LINK_PREFIX_LENGTH =
  "/chromium-datasets/00000000-0000-0000-0000-000000000000/raw-files/".length;
const PARSED_FILE_LINK_PREFIX_LENGTH =
  "/chromium-datasets/00000000-0000-0000-0000-000000000000/parsed-files/".length;

async function downloadParsedFile(link: string): Promise<FileNode> {
  const name = link.slice(PARSED_FILE_LINK_PREFIX_LENGTH);

  const content = await downloadFile({ link, accept: "application/json", acceptEncoding: "" }).then(
    (r) => r.json(),
  );

  return { name, src: link, content: content.data, type: "json" };
}

async function createRawFileNode(link: string): Promise<FileNode> {
  const name = link.slice(RAW_FILE_LINK_PREFIX_LENGTH);

  return {
    name,
    src: link,
    type: "html",
  };
}
