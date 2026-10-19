import { cellnoorClient, unwrap } from '#lib/client.ts';

export const prerender = false;

export async function load({ params }) {
	const dataset = await cellnoorClient
		.GET('/chromium-datasets/{id}', { params: { path: { id: params.id } } })
		.then(unwrap);

	return { dataset };
}
