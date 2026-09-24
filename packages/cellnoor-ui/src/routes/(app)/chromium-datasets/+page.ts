import { cellnoorClient } from '#lib/client.ts';
import { error } from '@sveltejs/kit';

export async function load() {
	const response = await cellnoorClient.POST('/chromium-datasets/search/detailed', { body: {} });

	if (response.error) {
		error(response.response.status, { ...response.error, message: 'something went wrong' });
	}

	return { datasets: response.data, datasetIds: response.data.map((ds) => ds.id) };
}
