<script lang="ts">
	import { page } from '$app/state';
	import { Combobox, useListCollection } from '@ark-ui/svelte/combobox';
	import { useFilter } from '@ark-ui/svelte/locale';
	import { Portal } from '@ark-ui/svelte/portal';
	import X from '@lucide/svelte/icons/x';

	let { label, name, items }: { label: string; name: string; items: [string, string][] } = $props();

	let selectedItems = $derived(page.url.searchParams.getAll(name));

	const filters = useFilter({ sensitivity: 'base' });

	// You have to pass a closure here to make svelte shut up
	const { collection, filter } = useListCollection(() => ({
		initialItems: items,
		itemToValue: ([itemValue]) => itemValue,
		itemToString: ([, itemLabel]) => itemLabel,
		filter: filters().contains
	}));
</script>

<Combobox.Root
	{collection}
	multiple
	openOnClick
	bind:value={selectedItems}
	onInputValueChange={(details) => filter(details.inputValue)}
>
	<Combobox.Label>{label}</Combobox.Label>
	<Combobox.Control>
		{#each selectedItems as itemValue (itemValue)}
			{@const itemLabel = items.find(([v]) => v === itemValue)?.[1]}
			<span>
				{itemLabel}
				<button
					type="button"
					aria-label="Remove {itemLabel}"
					onclick={() => (selectedItems = selectedItems.filter((v) => v !== itemValue))}
					><X size={14} /></button
				>
			</span>
			<input type="hidden" {name} value={itemValue} />
		{/each}
		<Combobox.Input />
	</Combobox.Control>
	<Portal>
		<Combobox.Positioner>
			<Combobox.Content>
				{#each collection().items as item (item[0])}
					<Combobox.Item {item}>
						<Combobox.ItemText>{item[1]}</Combobox.ItemText>
						<Combobox.ItemIndicator>✓</Combobox.ItemIndicator>
					</Combobox.Item>
				{/each}
			</Combobox.Content>
		</Combobox.Positioner>
	</Portal>
</Combobox.Root>

<style>
	:global([data-scope='combobox'][data-part='content']) {
		max-height: 20rem;
		overflow-y: auto;
		background: white;
		border: 1px solid var(--color-secondary);
	}

	:global([data-scope='combobox'][data-part='item'][data-highlighted]) {
		background: var(--color-secondary);
		color: white;
	}
</style>
