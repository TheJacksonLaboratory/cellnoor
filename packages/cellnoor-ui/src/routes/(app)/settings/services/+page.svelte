<script lang="ts">
	import FilteredList from '#lib/FilteredList.svelte';
	import Plus from '@lucide/svelte/icons/plus';
	import { cellnoorClient, mutate } from '#lib/client.ts';
	import DateRange from '#lib/filters/DateRange.svelte';
	import FilterGroup from '#lib/filters/FilterGroup.svelte';
	import MultiSelectFilter from '#lib/filters/MultiSelectFilter.svelte';
	import { staff } from '#lib/filters/options.ts';
	import PageSize from '#lib/filters/PageSize.svelte';
	import Sort from '#lib/filters/Sort.svelte';
	import TagInput from '#lib/filters/TagInput.svelte';
	import ListRow from '#lib/ListRow.svelte';
	import Members from '#lib/Members.svelte';
	import MutationCheckbox from '#lib/MutationCheckbox.svelte';
	import MutationInput from '#lib/MutationInput.svelte';
	import PermissionGrid from '#lib/PermissionGrid.svelte';
	import Time from '#lib/Time.svelte';
	import type {
		ServiceDetailed,
		ServiceField,
		ServiceSimpleFields
	} from 'cellnoor-client/cellnoor-types.ts';
	import { FormFieldName } from './field-parsers.ts';

	let { data } = $props();

	const { serviceList, personNames, orderBy, limit } = $derived(data);

	const sortFields = new Map<ServiceField, string>([
		['created_at', 'Date created'],
		['description', 'Description']
	]);

	async function createService(e: SubmitEvent & { currentTarget: HTMLFormElement }) {
		e.preventDefault();
		const form = e.currentTarget;
		const description = new FormData(form).get('description') as string;

		const created = await mutate(
			cellnoorClient.POST('/services', {
				body: {
					description: description || null,
					is_staff: false,
					users: [],
					permissions_to_grant: []
				}
			}),
			'Created'
		);

		if (created) {
			form.reset();
		}
	}

	function updateService(
		{ id, description, is_staff }: ServiceDetailed,
		changes: Partial<ServiceSimpleFields>
	) {
		return cellnoorClient.PUT('/services/{id}', {
			params: { path: { id } },
			body: { description, is_staff, ...changes }
		});
	}
</script>

<FilteredList {limit} count={serviceList.length}>
	{#snippet actions()}
		<form class="card create-card" onsubmit={createService}>
			<h2 class="subheading card-header"><Plus size="1em" /> New service</h2>
			<div class="stack card-body">
				<label class="field">
					<span class="label">Description</span>
					<input class="input" name="description" />
				</label>
				<div>
					<button class="button">Create service</button>
				</div>
			</div>
		</form>
	{/snippet}

	{#snippet filters()}
		<FilterGroup label="Results">
			<Sort fields={sortFields} {orderBy} />
			<PageSize {limit} />
		</FilterGroup>
		<FilterGroup label="Service">
			<TagInput name={FormFieldName.Description} label="Description" />
			<MultiSelectFilter name={FormFieldName.IsStaff} label="Staff" options={staff} />
			<MultiSelectFilter name={FormFieldName.OwnedBy} label="Owner" options={personNames} />
			<DateRange
				from={FormFieldName.CreatedFrom}
				to={FormFieldName.CreatedTo}
				label="Date created"
			/>
		</FilterGroup>
	{/snippet}

	{#if serviceList.length}
		<ul class="divided">
			{#each serviceList as service (service.id)}
				{@const { id } = service}
				<ListRow title={service.description ?? id}>
					<div class="muted">
						Owned by {personNames.get(service.owned_by)} · Created
						<Time datetime={service.created_at} />
					</div>
					<MutationInput
						label="Description"
						save={(description) => updateService(service, { description })}
					/>
					<MutationCheckbox
						label="Staff"
						checked={service.is_staff}
						save={(is_staff) => updateService(service, { is_staff })}
					/>
					<PermissionGrid
						permissions={service.permissions}
						grant={(body) =>
							cellnoorClient.POST('/services/{id}/permissions', { params: { path: { id } }, body })}
						revoke={(body) =>
							cellnoorClient.DELETE('/services/{id}/permissions', {
								params: { path: { id } },
								body
							})}
					/>
					<Members
						label="Members"
						members={service.members}
						options={personNames}
						add={(people) =>
							cellnoorClient.POST('/services/{id}/people', {
								params: { path: { id } },
								body: people
							})}
						remove={(people) =>
							cellnoorClient.DELETE('/services/{id}/people', {
								params: { path: { id } },
								body: people
							})}
					/>
					<div>
						<button
							class="button"
							onclick={() => {
								if (confirm(`Delete service "${service.description ?? id}"?`)) {
									mutate(
										cellnoorClient.DELETE('/services/{id}', { params: { path: { id } } }),
										'Deleted'
									);
								}
							}}
						>
							Delete service
						</button>
					</div>
				</ListRow>
			{/each}
		</ul>
	{:else}
		<p>No services match these filters.</p>
	{/if}
</FilteredList>
