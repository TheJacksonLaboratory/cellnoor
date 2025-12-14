<script lang="ts">
  import { authClient } from "$lib/auth-client";
  import { resolve } from "$app/paths";

  const { user_name } = $props();
</script>

<nav class="navbar bg-base-300 shadow sticky top-0 z-50">
  <div class="navbar-start">
    <a href={resolve("/")} class="btn btn-lg btn-ghost font-bold">scamplers</a>
  </div>
  <div class="navbar-end">
    <button class="btn btn-ghost font-bold mx-2">
      <a href={resolve("/chromium-datasets")}>Chromium Datasets</a>
    </button>
    <button class="btn btn-ghost font-bold mx-2">
      <a href={resolve("/specimens")}>Specimens</a>
    </button>
    <button
      class="btn btn-outline btn-primary mx-2"
      popovertarget="name-popover"
      style="anchor-name: --name-anchor"
    >
      {user_name}
    </button>
    <ul
      class="dropdown menu shadow rounded-box p-0 bg-base-100"
      popover
      id="name-popover"
      style="position-anchor: --name-anchor; min-width: anchor-size(width)"
    >
      <li>
        <a class="btn btn-ghost justify-start" href="/profile">Profile</a>
      </li>
      <li>
        <button
          class="btn btn-ghost justify-start"
          onclick={async () => {
            await authClient.signOut();
          }}
        >
          Sign Out
        </button>
      </li>
    </ul>
  </div>
</nav>
