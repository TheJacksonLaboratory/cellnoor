import { redirect } from '@sveltejs/kit';
import { authClient } from '#lib/auth.js';
import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async ({ url }) => {
	const { data } = await authClient.getSession();

	if (!data) {
		redirect(307, `/sign-in/?redirect_to=${encodeURIComponent(url.pathname)}`);
	}

	return { user: data.user };
};
