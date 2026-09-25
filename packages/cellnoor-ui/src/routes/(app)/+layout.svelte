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

	<button popovertarget="datasets-menu">Datasets <ChevronDown size={16} /></button>
	<div id="datasets-menu" popover bind:this={datasetsMenu}>
		<a href="/chromium-datasets">Chromium</a>
	</div>

	<button
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
		gap: var(--md-gap);
		padding-block: var(--sm-gap);
		padding-inline: var(--md-gap);
		border-block-end: 1px solid var(--color-secondary);
	}

	.brand {
		font-family: 'Comfortaa Variable', system-ui, sans-serif;
		font-size: 2rem;
		margin-inline-end: auto;
	}

	a,
	[popovertarget] {
		color: inherit;
		text-decoration: none;

		&:hover {
			color: var(--color-primary);
		}
	}

	[popovertarget] {
		display: flex;
		align-items: center;
		gap: 0.25em;
		padding: 0;
		font: inherit;
		background: none;
		border: none;
		cursor: pointer;
	}

	[popover] {
		position-area: block-end span-inline-end;
		margin: 0;
		border: 1px solid var(--color-secondary);
		a {
			display: block;
			padding-block: var(--sm-gap);
		}
	}
</style>
