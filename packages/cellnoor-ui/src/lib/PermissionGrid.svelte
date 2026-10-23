<script lang="ts">
	import {
		actionValues,
		resourceValues,
		type Action,
		type Permission,
		type Resource
	} from 'cellnoor-client/cellnoor-types.ts';
	import { mutate, type Mutation } from './client.ts';
	import { formatEnum } from './format.ts';

	let {
		permissions,
		save
	}: {
		permissions: Permission[];
		save: (grant: Permission[], revoke: Permission[]) => Mutation;
	} = $props();

	const has = (resource: Resource, action: Action) =>
		permissions.some((p) => p.resource === resource && p.action === action);

	async function toggle(input: HTMLInputElement, permission: Permission) {
		const granted = input.checked;

		if (!(await mutate(granted ? save([permission], []) : save([], [permission])))) {
			input.checked = !granted;
		}
	}
</script>

<div class="scroll-x">
	<table>
		<thead>
			<tr>
				<td></td>
				{#each actionValues as action (action)}
					<th scope="col">{formatEnum(action)}</th>
				{/each}
			</tr>
		</thead>
		<tbody>
			{#each resourceValues as resource (resource)}
				<tr>
					<th scope="row">{formatEnum(resource)}</th>
					{#each actionValues as action (action)}
						<td>
							<input
								type="checkbox"
								aria-label="{formatEnum(action)} {formatEnum(resource)}"
								checked={has(resource, action)}
								onchange={(e) => toggle(e.currentTarget, { resource, action })}
							/>
						</td>
					{/each}
				</tr>
			{/each}
		</tbody>
	</table>
</div>
