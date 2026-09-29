<script lang="ts">
	import { Combobox } from 'bits-ui';
	import { getFormSubmissionFn } from './context';
	import { Check, ChevronsUpDown } from '@lucide/svelte';
	import { page } from '$app/state';

	let {
		name,
		fieldLabel,
		options
	}: {
		name: string;
		fieldLabel: string;
		options: { value: string; label: string }[];
	} = $props();

	let searchValue = $state('');

	async function filterOptions() {
		return searchValue
			? options.filter((opt) => opt.value.toLowerCase().includes(searchValue.toLowerCase()))
			: options;
	}

	const filteredOptions = $derived(await filterOptions());

	const submitForm = getFormSubmissionFn();
</script>

<label
	>{fieldLabel}
	<Combobox.Root
		{name}
		type="multiple"
		value={page.url.searchParams.getAll(name)}
		onValueChange={submitForm}
		onOpenChangeComplete={(isOpen) => (isOpen ? null : submitForm())}
	>
		<div>
			<Combobox.Input oninput={(e) => (searchValue = e.currentTarget.value)} />
			<Combobox.Trigger>
				<ChevronsUpDown />
			</Combobox.Trigger>
		</div>
		<Combobox.Portal>
			<Combobox.Content>
				{#each filteredOptions as opt (opt.value)}
					<Combobox.Item {...opt}>
						{#snippet children({ selected })}
							{opt.label}
							{#if selected}
								<Check />
							{/if}
						{/snippet}
					</Combobox.Item>
				{/each}
			</Combobox.Content>
		</Combobox.Portal>
	</Combobox.Root>
</label>
