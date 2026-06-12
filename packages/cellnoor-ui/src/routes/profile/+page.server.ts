import { getApiClient } from "$lib/server/cellnoor-client.js";


async function createNewApiKey(description: string | null) {
  const client = getApiClient();

  const { data } = await client.POST("/api-keys", { body: { description } });

  if (!data) {
    return { error: "something went wrong" };
  }

  return { apiKey: data.secret }
}

function validateFormData(data: FormData) {
  const description = data.get("description");

  return {
    description: description ? description.toString() : null,
  };
}

export const actions = {
  createApiToken: async ({request}) => {
    const { description } = validateFormData(await request.formData());

    return await createNewApiKey(description);
  },
  deleteApiToken: async ({request,}) => {
    const data = await request.formData();
    const apiKeyId = data.get("apiKeyId")?.toString();

    if (!apiKeyId) {
      return;
    }

    const client = getApiClient();
    await client.DELETE("/api-keys/{id}", { params: { path: { id: apiKeyId } } });
  },
};

export async function load() {
  const client = getApiClient();
  const { data, error } = await client.GET("/api-keys");

  return data ? {apiKeys: data} : error
}
