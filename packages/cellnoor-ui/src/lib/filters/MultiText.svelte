<script lang="ts">
	import { page } from '$app/state';
	import { TagsInput } from '@ark-ui/svelte/tags-input';
	import X from '@lucide/svelte/icons/x';

	let { label, name }: { label: string; name: string } = $props();

	let value = $derived(page.url.searchParams.getAll(name));
</script>

<TagsInput.Root
	{value}
	onValueChange={(details) => (value = details.value)}
	blurBehavior="add"
	editable={false}
>
	<TagsInput.Label>{label}</TagsInput.Label>
	<TagsInput.Control>
		{#each value as v, index (index)}
			<TagsInput.Item {index} value={v}>
				<TagsInput.ItemPreview>
					<TagsInput.ItemText>{v}</TagsInput.ItemText>
					<TagsInput.ItemDeleteTrigger><X size={14} /></TagsInput.ItemDeleteTrigger>
				</TagsInput.ItemPreview>
			</TagsInput.Item>
			<input type="hidden" {name} value={v} />
		{/each}
		<TagsInput.Input />
	</TagsInput.Control>
</TagsInput.Root>
