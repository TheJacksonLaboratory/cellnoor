<script lang="ts">
	import { authClient } from '#lib/auth.ts';
	import { afterNavigate, goto } from '$app/navigation';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';

	let { children } = $props();

	let datasetsMenu: HTMLElement;
	afterNavigate(() => datasetsMenu.hidePopover());
</script>

<nav>
	<a class="brand" href="/">cellnoor</a>
	<a href="/specimens">Specimens</a>
	<a href="/libraries">Libraries</a>

	<button popovertarget="datasets-menu">Datasets <ChevronDown size="1em" /></button>
	<div id="datasets-menu" class="card menu" popover bind:this={datasetsMenu}>
		<a class="menu-item" href="/chromium-datasets">Chromium</a>
	</div>

	<button
		class="button"
		onclick={() => authClient.signOut({ fetchOptions: { onSuccess: () => goto('/sign-in') } })}
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
		font-family: 'Comfortaa Variable', system-ui, sans-serif;
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
		padding: 0;
		font: inherit;
		background: none;
		border: none;
		cursor: pointer;
	}

	[popover] {
		position-area: block-end span-inline-end;
		margin: var(--space-xs) 0 0;
	}
</style>
