<script lang="ts">
  import { enhance } from "$app/forms";
  import { DATETIME_FORMATTER } from "$lib/date.js";

  const { data, form } = $props();
  const {
    apiTokens,
    user: { name: userName, email, image },
    today,
    sixMonthsFromNow,
  } = $derived(data);
  let apiKeysDialogBox: HTMLDialogElement;
</script>

<div class="min-h-1/2 mx-auto flex flex-col items-center w-fit">
  {#if image}
    <div class="avatar">
      <img
        class="rounded-full"
        src={image}
        alt="profile"
      />
    </div>
  {/if}
  <h1 class="text-4xl font-bold">{userName}</h1>
  <p class="text-xl font-bold">{email}</p>
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
            {#each apiTokens as { jti, name, description, iat, exp }}
              <tr>
                <td>
                  {name}
                </td>
                <td>
                  {description}
                </td>
                <td>
                  {
                    DATETIME_FORMATTER.format(
                      iat,
                    )
                  }
                </td>
                <td>
                  {DATETIME_FORMATTER.format(exp)}
                </td>
                <td>
                  <form method="post" use:enhance action="?/deleteApiToken">
                    <input name="jti" value={jti} type="hidden" />
                    <button class="btn btn-error">Delete</button>
                  </form>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
        <form
          class="flex flex-col grow gap-2"
          method="post"
          use:enhance
          action="?/createApiToken"
        >
          <!-- TODO: use svelte's snippet's and rendering -->
          <fieldset class="fieldset">
            <legend class="fieldset-legend">API token name</legend>
            <input name="name" type="text" class="input" />
          </fieldset>
          <fieldset class="fieldset">
            <legend class="fieldset-legend">Description</legend>
            <textarea name="description" class="textarea"></textarea>
          </fieldset>
          <fieldset class="fieldset">
            <legend class="fieldset-legend">Expires on</legend>
            <input
              class="input"
              name="expiresOn"
              type="date"
              min={today}
              max={sixMonthsFromNow}
              required
            />
          </fieldset>
          <button class="btn btn-success max-w-3/4">
            Create new API token
          </button>
        </form>
      </div>
      {#if form}
        <div class="wrap-anywhere py-1 text-left">
          {#if form?.apiToken}
            Your new API token is <code class="font-bold">{
              form?.apiToken
            }</code>. You will not be able to view this token after leaving or
            refreshing this page. Store this token securely.
          {:else if form?.error}
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
