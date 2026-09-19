<script lang="ts">
	import { page } from '$app/state';
	import { authClient } from '#lib/auth.js';

	async function signInWithMicrosoft() {
		return await authClient.signIn.social({
			provider: 'microsoft',
			callbackURL: page.url.searchParams.get('redirect_to') ?? '/'
		});
	}
</script>

<button onclick={signInWithMicrosoft}>Sign in with Microsoft</button>
