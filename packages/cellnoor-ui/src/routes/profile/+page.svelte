<script lang="ts">
  import { PUBLIC_AUTH_URL } from "$app/env/public";
  import { enhance } from "$app/forms";
  import { createAuthClient } from "better-auth/svelte";

  const authClient = createAuthClient({ baseURL: PUBLIC_AUTH_URL });

  let session = authClient.useSession();

  const { data: sessionData } = $derived($session);

  const { data, form } = $props();
  const { apiKeys } = $derived(data);
  let apiKeysDialogBox: HTMLDialogElement;
</script>

<div class="min-h-1/2 mx-auto flex flex-col items-center w-fit">
  {#if sessionData?.user.image}
    <div class="avatar">
      <img class="rounded-full" src={sessionData.user.image} alt="profile" />
    </div>
  {/if}
  <h1 class="text-4xl font-bold">{sessionData?.user.name}</h1>
  <p class="text-xl font-bold">{sessionData?.user.email}</p>
  <div class="divider"></div>
  <button
    class="btn btn-primary btn-outline"
    onclick={async () => {
      apiKeysDialogBox.showModal();
    }}
  >
    API Tokens
  </button>
  <dialog bind:this={apiKeysDialogBox} class="modal">
    <div class="modal-box max-w-full xl:max-w-1/2 lg:max-w-3/4 flex flex-col">
      <div class="flex flex-row justify-between gap-2 place-items-start">
        <table class="table max-w-3/4">
          <thead>
            <tr>
              <th>API token name</th>
              <th>Description</th>
              <th>Created at</th>
              <th>Expires on</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {#each apiKeys as { id, description, created_at, expires_at } (id)}
              <tr>
                <td>
                  {name}
                </td>
                <td>
                  {description}
                </td>
                <td>
                  {created_at}
                </td>
                <td>
                  {expires_at || ""}
                </td>
                <td>
                  <form method="post" use:enhance action="?/deleteApiToken">
                    <input name="jti" value={id} type="hidden" />
                    <button class="btn btn-error">Delete</button>
                  </form>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
        <form class="flex flex-col grow gap-2" method="post" use:enhance action="?/createApiToken">
          <!-- TODO: use svelte's snippet's and rendering -->
          <fieldset class="fieldset">
            <legend class="fieldset-legend">API token name</legend>
            <input name="name" type="text" class="input" />
          </fieldset>
          <fieldset class="fieldset">
            <legend class="fieldset-legend">Description</legend>
            <textarea name="description" class="textarea"></textarea>
          </fieldset>
          <button class="btn btn-success max-w-3/4"> Create new API token </button>
        </form>
      </div>
      {#if form}
        <div class="wrap-anywhere py-1 text-left">
          {#if form.apiKey}
            Your new API token is <code class="font-bold">{form?.apiKey}</code>. You will not be
            able to view this token after leaving or refreshing this page. Store this token
            securely.
          {:else if form.error}
            {form.error}
          {/if}
        </div>
      {/if}
      <form class="modal-action" method="dialog">
        <button class="btn btn-secondary btn-outline">Close</button>
      </form>
    </div>
    <form method="dialog" class="modal-backdrop">
      <button>close</button>
    </form>
  </dialog>
</div>
