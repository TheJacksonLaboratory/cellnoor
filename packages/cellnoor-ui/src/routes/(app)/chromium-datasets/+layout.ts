import { cellnoorClient, unwrap } from '#lib/client.ts';

export async function load() {
	const [projects, assays] = await Promise.all([
		cellnoorClient.GET('/projects').then(unwrap),
		cellnoorClient.GET('/10x-assays').then(unwrap)
	]);

	return {
		projects,
		assays
	};
}
