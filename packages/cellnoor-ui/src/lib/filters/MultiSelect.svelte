<script lang="ts">
	import { Combobox } from 'bits-ui';
	import { Check, ChevronDown } from '@lucide/svelte';
	import { page } from '$app/state';
	import { getFormSubmissionFn } from './context';
	import SelectedValues from './SelectedValues.svelte';

	let {
		name,
		label,
		options
	}: {
		name: string;
		label: string;
		options: Map<string, string>;
	} = $props();

	const id = $props.id();

	let value = $derived(page.url.searchParams.getAll(name));
	let searchValue = $state('');

	const filteredOptions = $derived(
		options
			.entries()
			.filter(([, optText]) => optText.toLowerCase().includes(searchValue.toLowerCase()))
	);

	const submitForm = getFormSubmissionFn();
</script>

<div class="field">
	<label class="label" for={id}>{label}</label>
	<SelectedValues bind:values={value} {options} />
	<Combobox.Root {name} type="multiple" bind:value onValueChange={submitForm}>
		<div class="control">
			<Combobox.Input
				{id}
				autocomplete="off"
				class="input"
				oninput={(e) => (searchValue = e.currentTarget.value)}
			/>
			<Combobox.Trigger class="button">
				<ChevronDown size="1em" />
			</Combobox.Trigger>
		</div>
		<Combobox.Portal>
			<Combobox.Content class="card menu" style="inline-size: var(--bits-combobox-anchor-width)">
				{#each filteredOptions as [optValue, optText] (optValue)}
					<Combobox.Item class="menu-item" value={optValue} label={optText}>
						{#snippet children({ selected })}
							{optText}
							{#if selected}
								<Check size="1em" />
							{/if}
						{/snippet}
					</Combobox.Item>
				{:else}
					<div class="empty">No matching options</div>
				{/each}
			</Combobox.Content>
		</Combobox.Portal>
	</Combobox.Root>
</div>

<style>
	.control {
		display: grid;
		grid-template-columns: 1fr auto;
		gap: var(--space-xs);
	}

	.empty {
		padding: var(--space-sm);
	}
</style>
