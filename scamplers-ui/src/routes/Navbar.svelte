<script lang="ts">
  let { name } = $props();
  import { authClient } from "$lib/auth-client";
</script>

<div class="navbar bg-base shadow fixed">
  <div class="navbar-start">
    <a href="/" class="btn btn-lg btn-ghost font-bold text-base-content">scamplers</a>
  </div>
  <div class="navbar-center"><input type="text" placeholder="" class="input"/></div>
  <div class="navbar-end">
    <button
      class="btn btn-outline btn-primary"
      popovertarget="name-popover"
      style="anchor-name: --name-anchor"
    >
      {name}
    </button>
    <ul
      class="dropdown menu"
      popover
      id="name-popover"
      style="position-anchor: --name-anchor"
    >
      <li><a class="btn btn-ghost" href="/profile">Profile</a></li>
      <li>
        <button class="btn btn-ghost"
          onclick={async () => {
            await authClient.signOut();
          }}
        >
          Sign Out
        </button>
      </li>
    </ul>
  </div>
</div>
