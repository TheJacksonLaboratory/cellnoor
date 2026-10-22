<script lang="ts">
	import { tick, type Snippet } from 'svelte';
	import { setFormSubmissionFn } from './filters/context.ts';
	import SidebarLayout from './SidebarLayout.svelte';
	import { SvelteURLSearchParams } from 'svelte/reactivity';
	import { page } from '$app/state';

	let {
		filters,
		children,
		limit,
		count
	}: { filters: Snippet; children: Snippet; limit: number | null; count: number } = $props();

	function withLimit(limit: number) {
		const params = new SvelteURLSearchParams(page.url.search);
		params.set('limit', String(limit));
		return `?${params}`;
	}

	let form: HTMLFormElement;
	setFormSubmissionFn(async () => {
		await tick();
		form.requestSubmit();
	});
</script>

<SidebarLayout>
	{#snippet sidebar()}
		<header class="cluster">
			<h2 class="heading">Filters</h2>
			<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
			<a class="button" href="?">Clear all</a>
		</header>
		<form class="stack" bind:this={form} data-sveltekit-reset="false">
			{@render filters()}
		</form>
	{/snippet}

	{@render children()}

	{#if limit !== null && count >= limit}
		<footer class="cluster">
			<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
			<a class="button" href={withLimit(limit + 50)} data-sveltekit-reset="false">Load more</a>
		</footer>
	{/if}
</SidebarLayout>

<style>
	header {
		justify-content: space-between;
	}

	form {
		gap: var(--space-xl);
	}
</style>
