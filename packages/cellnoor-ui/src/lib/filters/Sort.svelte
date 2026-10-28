<script lang="ts" generics="F extends string">
	import ArrowDownWideNarrow from '@lucide/svelte/icons/arrow-down-wide-narrow';
	import ArrowUpNarrowWide from '@lucide/svelte/icons/arrow-up-narrow-wide';
	import { getFormSubmissionFn } from './context.ts';

	let {
		fields,
		orderBy
	}: {
		fields: Map<F, string>;
		orderBy: { field: F; desc: boolean };
	} = $props();

	const id = $props.id();

	let desc = $derived(orderBy.desc);

	const submitForm = getFormSubmissionFn();

	function toggleDirection() {
		desc = !desc;
		submitForm();
	}
</script>

<div class="field">
	<label class="label" for={id}>Sort by</label>
	<div class="control">
		<select {id} class="input" name="sort" value={orderBy.field} onchange={submitForm}>
			{#each fields as [field, label] (field)}
				<option value={field}>{label}</option>
			{/each}
		</select>
		<input type="hidden" name="desc" value={desc} />
		<button
			type="button"
			class="button"
			aria-label="Sort descending"
			aria-pressed={desc}
			onclick={toggleDirection}
		>
			{#if desc}
				<ArrowDownWideNarrow size="1em" />
			{:else}
				<ArrowUpNarrowWide size="1em" />
			{/if}
		</button>
	</div>
</div>

<style>
	.control {
		display: grid;
		grid-template-columns: 1fr auto;
		gap: var(--space-xs);
	}
</style>
