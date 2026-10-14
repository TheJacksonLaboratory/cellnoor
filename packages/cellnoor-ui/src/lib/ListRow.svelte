<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		title,
		href,
		subtitle,
		subtitleHref,
		checked = $bindable(),
		onCheckChange,
		children
	}: {
		title: string;
		href: string;
		subtitle?: string;
		subtitleHref?: string;
		checked?: boolean;
		onCheckChange?: () => void;
		children?: Snippet;
	} = $props();
</script>

<li>
	{#if checked !== undefined}
		<input type="checkbox" aria-label="Select {title}" bind:checked onchange={onCheckChange} />
	{/if}
	<div class="stack">
		<div>
			<a class="heading" {href}>{title}</a>
			{#if subtitle}
				<a class="subheading link-arrow" href={subtitleHref}>{subtitle}</a>
			{/if}
		</div>
		{@render children?.()}
	</div>
</li>

<style>
	li {
		display: grid;
		grid-template-columns: auto 1fr;
		align-items: baseline;
		gap: var(--space-md);
		padding-block: var(--space-lg);
		overflow-wrap: anywhere;
	}

	.heading {
		color: var(--color-primary);
	}

	.subheading {
		margin-left: var(--space-sm);
		padding-left: var(--space-sm);
		border-left: var(--border);
	}
</style>
