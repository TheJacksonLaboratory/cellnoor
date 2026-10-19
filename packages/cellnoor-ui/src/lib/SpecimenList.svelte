<script lang="ts">
	import type { Person, SpecimenCompact } from 'cellnoor-client/cellnoor-types.ts';
	import ListRow from './ListRow.svelte';
	import {
		DATE_FORMATTER,
		formatEmbeddingMatrix,
		formatEnum,
		formatFixative,
		formatSpecies
	} from './format.ts';

	let { specimens, people }: { specimens: SpecimenCompact[]; people: Map<string, Person> } =
		$props();

	type Field = [label: string, toText: (s: SpecimenCompact) => string | null | undefined];

	const fields: Field[] = [
		['Species', (s) => formatSpecies(s.species)],
		['Submitted by', (s) => people.get(s.submitted_by)?.name],
		['Type', (s) => formatEnum(s.type_)],
		['Tissue', (s) => s.tissue],
		['Preservation', (s) => s.thermal_preservation_method && formatEnum(s.thermal_preservation_method)],
		['Embedding matrix', (s) => s.embedded_in && formatEmbeddingMatrix(s.embedded_in)],
		['Fixative', (s) => s.fixative && formatFixative(s.fixative)]
	];

	const isShared = ([, toText]: Field) => new Set(specimens.map(toText)).size === 1;

	const shared = $derived(fields.filter(isShared));
	const varying = $derived(fields.filter((f) => !isShared(f)));
</script>

{#if shared.length}
	<dl class="properties">
		{#each shared as [label, toText] (label)}
			{@const text = toText(specimens[0])}
			{#if text}
				<dt>{label}</dt>
				<dd>{text}</dd>
			{/if}
		{/each}
	</dl>
{/if}

<ul class="divided">
	{#each specimens as s (s.id)}
		<ListRow title={s.name} href={s.links.self}>
			<div class="muted">
				{[...varying.map(([, toText]) => toText(s)).filter(Boolean), 'Received'].join(' · ')}
				<time datetime={s.received_at}>{DATE_FORMATTER.format(new Date(s.received_at))}</time>
			</div>
		</ListRow>
	{/each}
</ul>
