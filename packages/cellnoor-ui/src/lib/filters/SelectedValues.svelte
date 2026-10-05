<script lang="ts">
	import X from '@lucide/svelte/icons/x';
	import { getFormSubmissionFn } from './context';
	import { SvelteMap } from 'svelte/reactivity';

	let {
		values = $bindable(),
		options = new SvelteMap()
	}: {
		values: string[];
		options?: Map<string, string>;
	} = $props();

	const submitForm = getFormSubmissionFn();

	function remove(removed: string) {
		values = values.filter((v) => v !== removed);
		submitForm();
	}
</script>

{#if values.length}
	<ul class="cluster">
		{#each values as value (value)}
			{@const label = options.get(value) ?? value}
			<li class="card">
				{label}
				<button type="button" aria-label="Remove {label}" onclick={() => remove(value)}>
					<X size="1em" />
				</button>
			</li>
		{/each}
	</ul>
{/if}

<style>
	ul {
		gap: var(--space-xs);
	}

	li {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		padding-inline: var(--space-xs);
	}

	button {
		display: grid;
		padding: 0;
		border: none;
		background: none;
		color: inherit;
		cursor: pointer;
	}
</style>
