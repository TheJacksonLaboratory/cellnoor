<script lang="ts">
  import "../app.css";
  import { authClient } from "$lib/auth-client";
  import Navbar from "./Navbar.svelte";

  let { children, data } = $props();
  let user = $derived(data);
</script>

<svelte:head></svelte:head>

{#if user}
  <Navbar userName={user.name} />
{/if}

{@render children?.()}
