import { cellnoorClient } from '#lib/client.ts';

export async function load() {
	const { data, error } = await cellnoorClient.GET('/chromium-datasets');

	return { datasets: data, error };
}
