<script lang="ts">
	import { authClient } from '#lib/auth.ts';
	import { afterNavigate, goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';

	let { children } = $props();

	let datasetsMenu: HTMLElement;

	afterNavigate(({ shallow, type }) => {
		if (shallow && type === 'goto') return;

		datasetsMenu.hidePopover();
	});
</script>

<nav>
	<a class="brand" href={resolve('/(app)')}>cellnoor</a>
	<a href={resolve('/(app)/projects')}>Projects</a>
	<a href={resolve('/(app)/specimens')}>Specimens</a>
	<a href={resolve('/(app)/libraries')}>Libraries</a>

	<button class="plain-button" popovertarget="datasets-menu">
		Datasets <ChevronDown size="1em" />
	</button>
	<div id="datasets-menu" class="card menu" popover bind:this={datasetsMenu}>
		<a class="menu-item" href={resolve('/(app)/chromium-datasets')}>Chromium</a>
	</div>

	<button
		class="button"
		onclick={() =>
			authClient.signOut({ fetchOptions: { onSuccess: () => goto(resolve('/sign-in')) } })}
		>Sign Out</button
	>
</nav>

<main>
	{@render children()}
</main>

<style>
	nav {
		display: flex;
		align-items: center;
		gap: var(--space-lg);
		padding: var(--space-sm) var(--space-lg);
		border-block-end: 1px solid var(--color-primary);
		font-size: large;
	}

	.brand {
		font-family: var(--font-display);
		font-size: xx-large;
		margin-inline-end: auto;
	}

	nav > a,
	[popovertarget] {
		color: inherit;

		&:hover {
			color: var(--color-primary);
			text-decoration: none;
		}
	}

	[popovertarget] {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
	}

	[popover] {
		position-area: block-end span-inline-end;
		margin: var(--space-xs) 0 0;
	}
</style>
