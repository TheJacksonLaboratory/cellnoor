<script lang="ts">
	import { resolve } from '$app/paths';
	import FilteredList from '#lib/FilteredList.svelte';
	import { cellnoorClient } from '#lib/client.ts';
	import DateRange from '#lib/filters/DateRange.svelte';
	import FilterGroup from '#lib/filters/FilterGroup.svelte';
	import MultiSelectFilter from '#lib/filters/MultiSelectFilter.svelte';
	import PageSize from '#lib/filters/PageSize.svelte';
	import Sort from '#lib/filters/Sort.svelte';
	import TagInput from '#lib/filters/TagInput.svelte';
	import ListRow from '#lib/ListRow.svelte';
	import Members from '#lib/Members.svelte';
	import Time from '#lib/Time.svelte';
	import type { ProjectField } from 'cellnoor-client/cellnoor-types.ts';
	import { FormFieldName } from './field-parsers.ts';

	let { data } = $props();

	const { projectList, personNames, orderBy, limit } = $derived(data);

	const sortFields = new Map<ProjectField, string>([
		['started_at', 'Date started'],
		['ended_at', 'Date ended'],
		['name', 'Name']
	]);
</script>

<FilteredList {limit} count={projectList.length}>
	{#snippet filters()}
		<FilterGroup label="Results">
			<Sort fields={sortFields} {orderBy} />
			<PageSize {limit} />
		</FilterGroup>
		<FilterGroup label="Project">
			<TagInput name={FormFieldName.Name} label="Name" />
			<MultiSelectFilter name={FormFieldName.CreatedBy} label="Created by" options={personNames} />
			<DateRange from={FormFieldName.ActiveFrom} to={FormFieldName.ActiveTo} label="Active" />
		</FilterGroup>
	{/snippet}

	{#if projectList.length}
		<ul class="divided">
			{#each projectList as project (project.id)}
				{@const { id } = project}
				<ListRow title={project.name} href={resolve('/(app)/projects/[id]', { id })}>
					<div class="muted">
						Created by {personNames.get(project.created_by)} ·
						<Time datetime={project.started_at} /> - <Time datetime={project.ended_at} />
					</div>
					<Members
						label="Members"
						members={project.members}
						options={personNames}
						add={(people) =>
							cellnoorClient.POST('/projects/{id}/people', {
								params: { path: { id } },
								body: people
							})}
						remove={(people) =>
							cellnoorClient.DELETE('/projects/{id}/people', {
								params: { path: { id } },
								body: people
							})}
					/>
				</ListRow>
			{/each}
		</ul>
	{:else}
		<p>No projects match these filters.</p>
	{/if}
</FilteredList>
