import { authClient } from '#lib/auth.ts';
import { cellnoorClient, unwrap } from '#lib/client.ts';
import { redirect } from '@sveltejs/kit';

export async function load({ url }) {
	const session = await authClient.getSession();

	const user = session.data?.user;

	if (!user) {
		redirect(307, `/sign-in?redirect_to=${url.pathname}`);
	}

	const [projectList, tenxAssayList, personList] = await Promise.all([
		cellnoorClient.GET('/projects').then(unwrap),
		cellnoorClient.GET('/10x-assays').then(unwrap),
		cellnoorClient.GET('/people').then(unwrap)
	]);

	return {
		projects: new Map(projectList.map((p) => [p.id, p])),
		people: new Map(personList.map((p) => [p.id, p])),
		tenxAssays: new Map(tenxAssayList.map((a) => [a.name, a.name]))
	};
}
