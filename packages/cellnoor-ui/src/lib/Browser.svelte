<script lang="ts">
	import { tick, type Snippet } from 'svelte';
	import { page } from '$app/state';
	import { setFormSubmissionFn } from './filters/context.ts';

	let { filters, children }: { filters: Snippet; children: Snippet } = $props();

	let form: HTMLFormElement;
	setFormSubmissionFn(async () => {
		await tick();
		form.requestSubmit();
	});
</script>

<div class="browser">
	<aside class="stack">
		<header class="cluster">
			<h2 class="heading">Filters</h2>
			<a class="button" href={page.url.pathname}>Clear all</a>
		</header>
		<form class="stack" bind:this={form} data-sveltekit-reset="false">
			{@render filters()}
		</form>
	</aside>

	<section class="stack">
		{@render children()}
	</section>
</div>

<style>
	.browser {
		display: grid;
		grid-template-columns: minmax(min-content, 32rem) 1fr;
		align-items: start;

		> * {
			padding: var(--space-md);
		}
	}

	aside {
		position: sticky;
		inset-block-start: 0;
		max-block-size: 100dvh;
		overflow-y: auto;
		overscroll-behavior: contain;
	}

	header {
		justify-content: space-between;
	}

	form {
		gap: var(--space-xl);
	}
</style>
