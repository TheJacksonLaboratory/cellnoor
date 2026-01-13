<script lang="ts">
  import "../app.css";
  import { authClient } from "$lib/auth-client";
  import Navbar from "./Navbar.svelte";

  let { children } = $props();
  const session = authClient.useSession();
</script>

<svelte:head></svelte:head>

{#if $session.data}
  <Navbar userName={$session.data.user.name} />
{/if}

{@render children?.()}
