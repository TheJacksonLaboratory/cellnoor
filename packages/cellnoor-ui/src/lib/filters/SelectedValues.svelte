<script lang="ts">
	import X from '@lucide/svelte/icons/x';
	import { getFormSubmissionFn } from './context.ts';

	let {
		values = $bindable(),
		options = new Map()
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
				<button
					type="button"
					class="plain-button"
					aria-label="Remove {label}"
					onclick={() => remove(value)}
				>
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
	}
</style>
