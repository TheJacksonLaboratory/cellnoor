import { authClient } from '#lib/auth.ts';
import { cellnoorClient, unwrap } from '#lib/client.ts';
import { toStringMap } from '#lib/to-map.js';
import { redirect } from '@sveltejs/kit';

export async function load({ url }) {
	const session = await authClient.getSession();

	const user = session.data?.user;

	if (!user) {
		redirect(307, `/sign-in?redirect_to=${url.pathname}`);
	}

	const [projectList, tenxAssayList] = await Promise.all([
		cellnoorClient.GET('/projects').then(unwrap),
		cellnoorClient.GET('/10x-assays').then(unwrap)
	]);

	return {
		projects: new Map(projectList.map((p) => [p.id, p])),
		tenxAssays: toStringMap(tenxAssayList, (a) => [a.name, a.name])
	};
}
