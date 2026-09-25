<script lang="ts">
	import { page } from '$app/state';
	import { getSubmitFilters } from './FilterLayout.svelte';

	let { label, name }: { label: string; name: string } = $props();

	const submit = getSubmitFilters();
</script>

<fieldset>
	<legend>{label}</legend>
	<label>
		From
		<input
			type="date"
			name="{name}_from"
			value={page.url.searchParams.get(`${name}_from`) ?? ''}
			onchange={submit}
		/>
	</label>
	<label>
		To
		<input
			type="date"
			name="{name}_to"
			value={page.url.searchParams.get(`${name}_to`) ?? ''}
			onchange={submit}
		/>
	</label>
</fieldset>

<style>
	fieldset {
		display: grid;
		gap: var(--sm-gap);
	}

	label {
		display: grid;
	}
</style>
