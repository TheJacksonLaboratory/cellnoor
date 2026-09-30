<script lang="ts">
	import { afterNavigate } from '$app/navigation';
	import MultiSelect from '#lib/filters/MultiSelect.svelte';

	let { data } = $props();

	const {
		datasets,
		projects,
		assays,
		specimenTypes,
		species,
		embeddingMatrices,
		thermalPreservationMethods
	} = $derived(data);

	let selectedDatasets: string[] = $state([]);
	afterNavigate(() => (selectedDatasets = []));
</script>
