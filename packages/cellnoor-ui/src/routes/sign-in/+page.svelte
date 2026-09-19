<script lang="ts">
	import { createAuthClient } from 'better-auth/svelte';

	const authClient = createAuthClient();

	const redirectTo = () =>
		window.location.search ? new URLSearchParams(window.location.search).get('redirect_to') : '/';

	async function signInWithMicrosoft() {
		return await authClient.signIn.social({
			provider: 'microsoft',
			callbackURL: redirectTo() ?? '/'
		});
	}
</script>

<button onclick={signInWithMicrosoft}>Sign in with Microsoft</button>
