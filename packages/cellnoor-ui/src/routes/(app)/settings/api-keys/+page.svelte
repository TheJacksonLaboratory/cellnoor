<script lang="ts">
	import { invalidate } from '$app/navigation';
	import { cellnoorClient, errorMessage, MUTABLE_DATA, mutate } from '#lib/client.ts';
	import { endOfDay } from '#lib/filters/utils.ts';
	import ListRow from '#lib/ListRow.svelte';
	import Plus from '@lucide/svelte/icons/plus';
	import MutationInput from '#lib/MutationInput.svelte';
	import SidebarLayout from '#lib/SidebarLayout.svelte';
	import Time from '#lib/Time.svelte';
	import { toast } from '#lib/toasts.svelte.ts';
	import type { ApiKeyUpdate, SavedApiKeyRecord } from 'cellnoor-client/cellnoor-types.ts';

	let { data } = $props();

	const { apiKeys, serviceNames, people } = $derived(data);

	let secret = $state<string>();

	async function createApiKey(e: SubmitEvent & { currentTarget: HTMLFormElement }) {
		e.preventDefault();

		const form = e.currentTarget;
		const fields = new FormData(form);
		const expiresOn = fields.get('expires_on') as string;

		const { data: apiKey, error } = await cellnoorClient.POST('/api-keys', {
			body: {
				description: (fields.get('description') as string) || null,
				owner_id: (fields.get('owner_id') as string) || null,
				expires_at: expiresOn ? endOfDay(expiresOn) : null
			}
		});

		if (error) {
			toast(errorMessage(error), false);
			return;
		}

		secret = apiKey.secret;
		form.reset();
		await invalidate(MUTABLE_DATA);
		toast('Created', true);
	}

	function updateApiKey(
		{ id, description, expires_at }: SavedApiKeyRecord,
		changes: Partial<ApiKeyUpdate>
	) {
		return cellnoorClient.PUT('/api-keys/{id}', {
			params: { path: { id } },
			body: { description, expires_at, ...changes }
		});
	}
</script>

<SidebarLayout>
	{#snippet sidebar()}
		<form class="card create-card" onsubmit={createApiKey}>
			<h2 class="subheading card-header"><Plus size="1em" /> New API key</h2>
			<div class="stack card-body">
				<label class="field">
					<span class="label">Description</span>
					<input class="input" name="description" />
				</label>
				<label class="field">
					<span class="label">For service</span>
					<select class="input" name="owner_id">
						<option selected></option>
						{#each serviceNames as [id, name] (id)}
							<option value={id}>{name}</option>
						{/each}
					</select>
				</label>
				<label class="field">
					<span class="label">Expires on</span>
					<input class="input" type="date" name="expires_on" />
				</label>
				<div>
					<button class="button">Create API key</button>
				</div>
			</div>
		</form>
	{/snippet}

	{#if secret}
		<div class="card card-body">
			Your API key is <code>{secret}</code>. It will disappear once you leave or refresh this page.
		</div>
	{/if}

	{#if apiKeys.length}
		<ul class="divided">
			{#each apiKeys as key (key.id)}
				<ListRow title={key.description ?? key.id}>
					<div class="muted">
						{serviceNames.get(key.owner_id) ?? people.get(key.owner_id)?.name} · Created
						<Time datetime={key.created_at} /> ·
						{#if key.expires_at}
							Expires <Time datetime={key.expires_at} />
						{:else}
							Never expires
						{/if}
					</div>
					<MutationInput
						label="Description"
						save={(description) => updateApiKey(key, { description })}
					/>
					<MutationInput
						label="Expires on"
						type="date"
						save={(date) => updateApiKey(key, { expires_at: endOfDay(date) })}
					/>
					<div>
						<button
							class="button"
							onclick={() => {
								if (confirm(`Delete API key "${key.description ?? key.id}"?`)) {
									mutate(
										cellnoorClient.DELETE('/api-keys/{id}', { params: { path: { id: key.id } } }),
										'Deleted'
									);
								}
							}}
						>
							Delete API key
						</button>
					</div>
				</ListRow>
			{/each}
		</ul>
	{:else}
		<p>No API keys.</p>
	{/if}
</SidebarLayout>

<style>
	code {
		font-weight: bold;
	}
</style>
