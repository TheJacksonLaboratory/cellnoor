<script lang="ts">
	import FilteredList from '#lib/FilteredList.svelte';
	import { cellnoorClient } from '#lib/client.ts';
	import FilterGroup from '#lib/filters/FilterGroup.svelte';
	import MultiSelectFilter from '#lib/filters/MultiSelectFilter.svelte';
	import { staff } from '#lib/filters/options.ts';
	import PageSize from '#lib/filters/PageSize.svelte';
	import Sort from '#lib/filters/Sort.svelte';
	import TagInput from '#lib/filters/TagInput.svelte';
	import ListRow from '#lib/ListRow.svelte';
	import MutationCheckbox from '#lib/MutationCheckbox.svelte';
	import PermissionGrid from '#lib/PermissionGrid.svelte';
	import type {
		PersonField,
		PersonUpdate,
		PersonWithPermissions
	} from 'cellnoor-client/cellnoor-types.ts';
	import { FormFieldName } from './field-parsers.ts';

	let { data } = $props();

	const { personList, institutionNames, orderBy, limit } = $derived(data);

	const sortFields = new Map<PersonField, string>([
		['name', 'Name'],
		['email', 'Email']
	]);

	function updatePerson({ id, ...person }: PersonWithPermissions, changes: Partial<PersonUpdate>) {
		return cellnoorClient.PUT('/people/{id}', {
			params: { path: { id } },
			body: { ...person, ...changes }
		});
	}
</script>

<FilteredList {limit} count={personList.length}>
	{#snippet filters()}
		<FilterGroup label="Results">
			<Sort fields={sortFields} {orderBy} />
			<PageSize {limit} />
		</FilterGroup>
		<FilterGroup label="Person">
			<TagInput name={FormFieldName.Name} label="Name" />
			<TagInput name={FormFieldName.Email} label="Email" />
			<MultiSelectFilter name={FormFieldName.IsStaff} label="Staff" options={staff} />
			<MultiSelectFilter
				name={FormFieldName.InstitutionId}
				label="Institution"
				options={institutionNames}
			/>
		</FilterGroup>
	{/snippet}

	{#if personList.length}
		<ul class="divided">
			{#each personList as person (person.id)}
				<ListRow title={person.name}>
					<div class="muted">
						{#if person.email}{person.email} ·{/if}
						{institutionNames.get(person.institution_id)}
					</div>
					<MutationCheckbox
						label="Staff"
						checked={person.is_staff}
						save={(is_staff) => updatePerson(person, { is_staff })}
					/>
					<PermissionGrid
						permissions={person.permissions}
						grant={(body) =>
							cellnoorClient.POST('/people/{id}/permissions', {
								params: { path: { id: person.id } },
								body
							})}
						revoke={(body) =>
							cellnoorClient.DELETE('/people/{id}/permissions', {
								params: { path: { id: person.id } },
								body
							})}
					/>
				</ListRow>
			{/each}
		</ul>
	{:else}
		<p>No people match these filters.</p>
	{/if}
</FilteredList>
