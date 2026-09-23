<script lang="ts">
	import { authClient } from '#lib/auth.ts';
	import { afterNavigate, goto } from '$app/navigation';

	let { children } = $props();

	let datasetsMenu: HTMLDivElement;
	afterNavigate(() => datasetsMenu.hidePopover());
</script>

<nav>
	<a class="brand" href="/">cellnoor</a>
	<a href="/specimens">Specimens</a>
	<a href="/libraries">Libraries</a>
	<button class="menu-trigger" popovertarget="datasets-menu">Datasets</button>
	<div id="datasets-menu" popover bind:this={datasetsMenu}>
		<a href="/chromium-datasets">Chromium Datasets</a>
	</div>

	<button
		onclick={() => authClient.signOut({ fetchOptions: { onSuccess: () => goto('/sign-in') } })}
		>Sign Out</button
	>
</nav>

{@render children()}

<style>
	.brand {
		font-family: 'Comfortaa Variable', system-ui, sans-serif;
		font-size: 2rem;
		margin-inline-end: auto;
	}

	.menu-trigger {
		anchor-name: --datasets;
		background: none;
		color: black;

		&:hover {
			color: var(--color-primary);
		}
	}

	#datasets-menu {
		position-anchor: --datasets;
		position-area: bottom span-right;
		inset: auto;
		margin: var(--sm-gap);
		padding: var(--sm-gap);
		border: 1px solid var(--color-secondary);
		border-radius: 0.5rem;
	}

	nav {
		display: flex;
		align-items: center;
		gap: var(--md-gap);
		padding: var(--md-gap);
		border-bottom: 1px solid var(--color-secondary);

		a {
			text-decoration: none;
			color: black;

			&:hover {
				color: var(--color-primary);
			}
		}
	}
</style>
