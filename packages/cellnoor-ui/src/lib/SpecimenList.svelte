<script lang="ts">
	import { resolve } from '$app/paths';
	import type { Person, SpecimenCompact } from 'cellnoor-client/cellnoor-types.ts';
	import Time from './Time.svelte';
	import { formatEmbeddingMatrix, formatEnum, formatFixative, formatSpecies } from './format.ts';

	let { specimens, people }: { specimens: SpecimenCompact[]; people: Map<string, Person> } =
		$props();

	type Field = {
		label: string;
		toText: (s: SpecimenCompact) => string | null | undefined;
	};

	const fields: Field[] = [
		{ label: 'Species', toText: (s) => formatSpecies(s.species) },
		{ label: 'Submitted by', toText: (s) => people.get(s.submitted_by)?.name },
		{ label: 'Type', toText: (s) => formatEnum(s.type_) },
		{ label: 'Tissue', toText: (s) => s.tissue },
		{
			label: 'Preservation',
			toText: (s) => s.thermal_preservation_method && formatEnum(s.thermal_preservation_method)
		},
		{
			label: 'Embedding matrix',
			toText: (s) => s.embedded_in && formatEmbeddingMatrix(s.embedded_in)
		},
		{ label: 'Fixative', toText: (s) => s.fixative && formatFixative(s.fixative) }
	];

	const isShared = ({ toText }: Field) => new Set(specimens.map(toText)).size === 1;

	const shared = $derived(
		fields
			.filter(isShared)
			.map(({ label, toText }) => ({ label, text: toText(specimens[0]) }))
			.filter(({ text }) => text)
	);
	const varying = $derived(fields.filter((f) => !isShared(f)));
</script>

{#if shared.length}
	<div class="scroll-x">
		<table>
			<thead>
				<tr>
					{#each shared as { label } (label)}
						<th scope="col">{label}</th>
					{/each}
				</tr>
			</thead>
			<tbody>
				<tr>
					{#each shared as { label, text } (label)}
						<td>{text}</td>
					{/each}
				</tr>
			</tbody>
		</table>
	</div>
{/if}

<div class="scroll-x">
	<table>
		<thead>
			<tr>
				<th scope="col">Name</th>
				{#each varying as { label } (label)}
					<th scope="col">{label}</th>
				{/each}
				<th scope="col">Received</th>
			</tr>
		</thead>
		<tbody>
			{#each specimens as s (s.id)}
				<tr>
					<td
						><a class="link-arrow" href={resolve('/(app)/specimens/[id]', { id: s.id })}>{s.name}</a
						></td
					>
					{#each varying as { label, toText } (label)}
						<td>{toText(s)}</td>
					{/each}
					<td><Time datetime={s.received_at} /></td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
