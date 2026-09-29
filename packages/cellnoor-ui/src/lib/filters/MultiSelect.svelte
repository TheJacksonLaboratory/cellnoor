<script lang="ts" generics="QueryParameterName extends string">
	import { Combobox } from 'bits-ui';
	import { getFormContext } from './context';
	import { Check, ChevronsUpDown } from '@lucide/svelte';
	import { page } from '$app/state';

	let {
		name,
		fieldLabel,
		options
	}: {
		name: QueryParameterName;
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

	const form = getFormContext();
	const submit = () => form.requestSubmit();
</script>

<Combobox.Root
	{name}
	type="multiple"
	value={page.url.searchParams.getAll(name)}
	onValueChange={submit}
	onOpenChangeComplete={(isOpen) => (isOpen ? null : submit())}
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
