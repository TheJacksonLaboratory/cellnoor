<script lang="ts">
	import { page } from '$app/state';
	import { TagsInput } from '@ark-ui/svelte/tags-input';
	import X from '@lucide/svelte/icons/x';
	import { getSubmitFilters } from './FilterLayout.svelte';

	let { label, name }: { label: string; name: string } = $props();

	let value = $derived(page.url.searchParams.getAll(name));

	const submit = getSubmitFilters();
</script>

<TagsInput.Root
	{value}
	onValueChange={(details) => {
		value = details.value;
		submit();
	}}
	blurBehavior="add"
	editable={false}
>
	<TagsInput.Label>{label}</TagsInput.Label>
	<TagsInput.Control>
		{#each value as v, index (index)}
			<TagsInput.Item {index} value={v}>
				<TagsInput.ItemPreview class="badge">
					<TagsInput.ItemText>{v}</TagsInput.ItemText>
					<TagsInput.ItemDeleteTrigger><X size={14} /></TagsInput.ItemDeleteTrigger>
				</TagsInput.ItemPreview>
			</TagsInput.Item>
			<input type="hidden" {name} value={v} />
		{/each}
		<TagsInput.Input />
	</TagsInput.Control>
</TagsInput.Root>

<style>
	:global([data-scope='tags-input'][data-part='control']) {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--sm-gap);
	}

	:global([data-scope='tags-input'][data-part='input']) {
		flex-basis: 100%;
	}

	:global([data-scope='tags-input'][data-part='item-delete-trigger']) {
		display: inline-flex;
		vertical-align: middle;
		padding: 0;
	}
</style>
