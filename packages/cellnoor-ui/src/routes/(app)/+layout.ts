import { authClient } from '#lib/auth.ts';
import { cellnoorClient, unwrap } from '#lib/client.ts';
import { redirect } from '@sveltejs/kit';
import { resolve } from '$app/paths';

export async function load({ url }) {
	const session = await authClient.getSession();

	const user = session.data?.user;

	if (!user) {
		const query = new URLSearchParams({ redirect_to: url.pathname + url.search });
		redirect(307, `${resolve('/sign-in')}?${query}`);
	}

	const [projectList, tenxAssayList, personList, [currentUser]] = await Promise.all([
		cellnoorClient.GET('/projects').then(unwrap),
		cellnoorClient.GET('/10x-assays').then(unwrap),
		cellnoorClient.GET('/people').then(unwrap),
		cellnoorClient
			.POST('/people/search/permissions', { body: { filter: { id: user.id } } })
			.then(unwrap)
	]);

	return {
		projects: new Map(projectList.map((p) => [p.id, p])),
		people: new Map(personList.map((p) => [p.id, p])),
		tenxAssays: new Map(tenxAssayList.map((a) => [a.name, a.name])),
		canUpdatePeople: currentUser.permissions.some(
			({ resource, action }) => resource === 'person' && action === 'update'
		)
	};
}
