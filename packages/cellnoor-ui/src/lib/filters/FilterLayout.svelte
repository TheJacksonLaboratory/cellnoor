<script module lang="ts">
	import { createContext } from 'svelte';

	export const [getSubmitFilters, setSubmitFilters] = createContext<() => void>();
</script>

<script lang="ts">
	import { tick, type Snippet } from 'svelte';

	let { filters, children }: { filters: Snippet; children: Snippet } = $props();

	let form: HTMLFormElement;

	setSubmitFilters(async () => {
		await tick();
		form.requestSubmit();
	});
</script>

<div class="layout">
	<form method="get" bind:this={form} data-sveltekit-keepfocus data-sveltekit-noscroll>
		{@render filters()}
	</form>
	<div class="data">
		{@render children()}
	</div>
</div>

<style>
	.layout {
		display: grid;
		grid-template-columns: minmax(0, 1fr) minmax(0, 2fr);
	}

	form,
	.data {
		padding: var(--md-gap);
	}

	form {
		box-sizing: border-box;
		position: sticky;
		top: 0;
		max-height: 100dvh;
		overflow-y: auto;
		border-inline-end: 1px solid var(--color-secondary);
		display: flex;
		flex-direction: column;
		gap: var(--md-gap);
	}

	form > :global(fieldset) {
		display: flex;
		flex-direction: column;
		gap: var(--md-gap);
	}

	form > :global(fieldset) > :global(legend) {
		font-size: larger;
	}
</style>
