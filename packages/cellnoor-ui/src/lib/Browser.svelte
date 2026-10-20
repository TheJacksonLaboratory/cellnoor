<script lang="ts">
	import { tick, type Snippet } from 'svelte';
	import { page } from '$app/state';
	import { setFormSubmissionFn } from './filters/context.ts';
	import SidebarLayout from './SidebarLayout.svelte';

	let { filters, children }: { filters: Snippet; children: Snippet } = $props();

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
			<a class="button" href={page.url.pathname}>Clear all</a>
		</header>
		<form class="stack" bind:this={form} data-sveltekit-reset="false">
			{@render filters()}
		</form>
	{/snippet}

	{@render children()}
</SidebarLayout>

<style>
	header {
		justify-content: space-between;
	}

	form {
		gap: var(--space-xl);
	}
</style>
