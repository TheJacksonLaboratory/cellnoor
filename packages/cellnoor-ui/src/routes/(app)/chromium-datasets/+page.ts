import { cellnoorClient } from '#lib/client.ts';

export async function load() {
	const datasetsResponse = await cellnoorClient.GET('/chromium-datasets');

	return { datasets: datasetsResponse.data };
}
