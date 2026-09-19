import { createAuthClient } from 'better-auth/svelte';
import { inferAdditionalFields } from 'better-auth/client/plugins';

export const authClient = createAuthClient({
	plugins: [
		inferAdditionalFields({
			user: {
				institution_id: { type: 'string' },
				is_staff: { type: 'boolean' }
			}
		})
	]
});

export type User = typeof authClient.$Infer.Session.user;
