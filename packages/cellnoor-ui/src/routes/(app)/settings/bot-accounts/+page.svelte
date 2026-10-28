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
		BotAccountDetailed,
		BotAccountField,
		BotAccountSimpleFields
	} from 'cellnoor-client/cellnoor-types.ts';
	import { FormFieldName } from './field-parsers.ts';

	let { data } = $props();

	const { bot_accountList, personNames, orderBy, limit } = $derived(data);

	const sortFields = new Map<BotAccountField, string>([
		['created_at', 'Date created'],
		['description', 'Description']
	]);

	async function createBotAccount(e: SubmitEvent & { currentTarget: HTMLFormElement }) {
		e.preventDefault();
		const form = e.currentTarget;
		const description = new FormData(form).get('description') as string;

		const created = await mutate(
			cellnoorClient.POST('/bot-accounts', {
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

	function updateBotAccount(
		{ id, description, is_staff }: BotAccountDetailed,
		changes: Partial<BotAccountSimpleFields>
	) {
		return cellnoorClient.PUT('/bot-accounts/{id}', {
			params: { path: { id } },
			body: { description, is_staff, ...changes }
		});
	}
</script>

<FilteredList {limit} count={bot_accountList.length}>
	{#snippet actions()}
		<form class="card create-card" onsubmit={createBotAccount}>
			<h2 class="subheading card-header"><Plus size="1em" />New Bot Account</h2>
			<div class="stack card-body">
				<label class="field">
					<span class="label">Description</span>
					<input class="input" name="description" />
				</label>
				<div>
					<button class="button">Create bot account</button>
				</div>
			</div>
		</form>
	{/snippet}

	{#snippet filters()}
		<FilterGroup label="Results">
			<Sort fields={sortFields} {orderBy} />
			<PageSize {limit} />
		</FilterGroup>
		<FilterGroup label="BotAccount">
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

	{#if bot_accountList.length}
		<ul class="divided">
			{#each bot_accountList as bot_account (bot_account.id)}
				{@const { id } = bot_account}
				<ListRow title={bot_account.description ?? id}>
					<div class="muted">
						Owned by {personNames.get(bot_account.owned_by)} · Created
						<Time datetime={bot_account.created_at} />
					</div>
					<MutationInput
						label="Description"
						save={(description) => updateBotAccount(bot_account, { description })}
					/>
					<MutationCheckbox
						label="Staff"
						checked={bot_account.is_staff}
						save={(is_staff) => updateBotAccount(bot_account, { is_staff })}
					/>
					<PermissionGrid
						permissions={bot_account.permissions}
						grant={(body) =>
							cellnoorClient.POST('/bot-accounts/{id}/permissions', {
								params: { path: { id } },
								body
							})}
						revoke={(body) =>
							cellnoorClient.DELETE('/bot-accounts/{id}/permissions', {
								params: { path: { id } },
								body
							})}
					/>
					<Members
						label="Members"
						members={bot_account.members}
						options={personNames}
						add={(people) =>
							cellnoorClient.POST('/bot-accounts/{id}/people', {
								params: { path: { id } },
								body: people
							})}
						remove={(people) =>
							cellnoorClient.DELETE('/bot-accounts/{id}/people', {
								params: { path: { id } },
								body: people
							})}
					/>
					<div>
						<button
							class="button"
							onclick={() => {
								if (confirm(`Delete bot account "${bot_account.description ?? id}"?`)) {
									mutate(
										cellnoorClient.DELETE('/bot-accounts/{id}', { params: { path: { id } } }),
										'Deleted'
									);
								}
							}}
						>
							Delete bot_account
						</button>
					</div>
				</ListRow>
			{/each}
		</ul>
	{:else}
		<p>No bot accounts match these filters.</p>
	{/if}
</FilteredList>
