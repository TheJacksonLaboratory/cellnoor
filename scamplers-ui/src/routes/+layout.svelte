<script lang="ts">
  import "../app.css";
  import { authClient } from "$lib/auth-client";
  import Navbar from "./Navbar.svelte";

  const session = authClient.useSession();
  let { children } = $props();
</script>

<svelte:head></svelte:head>

{#if $session.data}
  <Navbar user_name={$session.data.user.name} />
{/if}

{@render children?.()}
