import { cellnoorClient, MUTABLE_DATA, unwrap } from '#lib/client.ts';

export async function load({ depends }) {
	depends(MUTABLE_DATA);

	const [apiKeys, services] = await Promise.all([
		cellnoorClient.GET('/api-keys').then(unwrap),
		cellnoorClient.GET('/services').then(unwrap)
	]);

	const serviceNames = new Map(services.map((s) => [s.id, s.description ?? s.id]));

	return { apiKeys, serviceNames };
}
