<script lang="ts">
	import { getFormSubmissionFn } from './context.ts';

	let { limit }: { limit: number | null } = $props();

	const id = $props.id();

	const submitForm = getFormSubmissionFn();
</script>

<div class="field">
	<label class="label" for={id}>Show</label>
	<select {id} class="input" name="limit" value={limit ?? 'all'} onchange={submitForm}>
		{#each [50, 100, 250, 500, 1000] as n (n)}
			<option value={n}>{n}</option>
		{/each}
		<option value="all">All</option>
	</select>
</div>
