<script lang="ts">
	import Browser from '#lib/Browser.svelte';
	import { cellnoorClient } from '#lib/client.ts';
	import FilterGroup from '#lib/filters/FilterGroup.svelte';
	import MultiSelect from '#lib/filters/MultiSelect.svelte';
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

	function updatePerson(
		{ id, permissions: _, ...person }: PersonWithPermissions,
		changes: Partial<PersonUpdate>
	) {
		return cellnoorClient.PUT('/people/{id}', {
			params: { path: { id } },
			body: { ...person, permissions_to_grant: [], permissions_to_revoke: [], ...changes }
		});
	}
</script>

<Browser {limit} count={personList.length}>
	{#snippet filters()}
		<FilterGroup label="Results">
			<Sort fields={sortFields} {orderBy} />
			<PageSize {limit} />
		</FilterGroup>
		<FilterGroup label="Person">
			<TagInput name={FormFieldName.Name} label="Name" />
			<TagInput name={FormFieldName.Email} label="Email" />
			<MultiSelect name={FormFieldName.IsStaff} label="Staff" options={staff} />
			<MultiSelect
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
						save={(grant, revoke) =>
							updatePerson(person, { permissions_to_grant: grant, permissions_to_revoke: revoke })}
					/>
				</ListRow>
			{/each}
		</ul>
	{:else}
		<p>No people match these filters.</p>
	{/if}
</Browser>
