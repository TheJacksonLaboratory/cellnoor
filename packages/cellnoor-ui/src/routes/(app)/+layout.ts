import { authClient } from '#lib/auth.ts';
import { redirect } from '@sveltejs/kit';

export async function load({ url }) {
	const session = await authClient.getSession();

	const user = session.data?.user;

	if (!user) {
		redirect(307, `/sign-in?redirect_to=${url.pathname}`);
	}
}
