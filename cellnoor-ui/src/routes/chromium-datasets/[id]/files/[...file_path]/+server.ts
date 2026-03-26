import { downloadFile } from "$lib/server/download-chromium-dataset-file.js";

// It would be faster to just grab the web summary from the database and send
// it directly. However, that requires us to implement an authorization check here and grant cellnoor-ui permission to
// read at least one table. I don't want to do that because the logic is implemented in Rust (bulletproof) already and
// I don't trust myself in this infernal language.

export async function GET({ params: { id, file_path } }) {
  const link = `/chromium-datasets/${id}/files/${file_path}`;

  const accept = file_path.endsWith(".csv") ? "text/csv" : "*/*";

  return downloadFile({ link, accept });
}
