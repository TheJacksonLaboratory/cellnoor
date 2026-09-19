<script lang="ts">
	import favicon from '#lib/assets/favicon.svg';
	import { Avatar } from '@ark-ui/svelte/avatar';

	let { data, children } = $props();
</script>

<nav>
	<a class="brand" href="/">
		<img src={favicon} alt="" width="28" height="28" />
		<span>cellnoor</span>
	</a>

	<a href="/specimens/">Specimens</a>
	<a href="/libraries/">Libraries</a>
	<a href="/chromium-datasets/">Chromium Datasets</a>

	<a class="user" href="/profile/">
		<Avatar.Root>
			<Avatar.Fallback>{data.user.name}</Avatar.Fallback>
			{#if data.user.image}
				<Avatar.Image src={data.user.image} alt="" />
			{/if}
		</Avatar.Root>
		<span>{data.user.name}</span>
	</a>
</nav>

{@render children()}

<style>
	nav {
		display: flex;
		align-items: center;
		gap: var(--gap);
		padding: calc(var(--gap) / 2) var(--gap);
		border-bottom: 1px solid var(--brand);

		a {
			display: flex;
			align-items: center;
			gap: calc(var(--gap) / 2);
			text-decoration: none;
			color: inherit;

			&:hover {
				color: var(--brand);
			}
		}
	}

	.brand {
		font-family: 'Comfortaa Variable', system-ui, sans-serif;
		font-size: 2rem;
		margin-inline-end: auto;
	}

	.user {
		color: var(--brand);

		:global([data-scope='avatar'][data-part='root']) {
			display: inline-flex;
			align-items: center;
			justify-content: center;
			width: 2rem;
			height: 2rem;
			border-radius: 50%;
			overflow: hidden;
			background: var(--brand);
			color: white;
		}

		:global([data-scope='avatar'][data-part='image']) {
			width: 100%;
			height: 100%;
			object-fit: cover;
		}
	}
</style>
