<script lang="ts" generics="T extends string">
	import { page } from '$app/state';
	import MultiSelect from '../MultiSelect.svelte';
	import { getFormSubmissionFn } from './context.ts';
	import type { StringEnum } from './string-enum.ts';

	let {
		name,
		label,
		options
	}: {
		name: StringEnum<T>;
		label: string;
		options: Map<string, string>;
	} = $props();

	let value = $derived(page.url.searchParams.getAll(name));

	const submitForm = getFormSubmissionFn();
</script>

<MultiSelect {name} {label} {options} bind:value onValueChange={submitForm} />
