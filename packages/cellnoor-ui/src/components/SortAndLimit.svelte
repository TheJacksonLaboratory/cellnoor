<script lang="ts">
  import { ArrowDownNarrowWide, ArrowUpNarrowWide } from "@lucide/svelte";
  import { tick } from "svelte";

  let {
    orderByField = $bindable(),
    orderByDescending = $bindable(),
    choices,
    parentForm,
  }: {
    orderByField?: string;
    orderByDescending?: boolean;
    choices: { label: string; value: string }[];
    parentForm: HTMLFormElement;
  } = $props();

  async function onchange() {
    await tick();
    parentForm.requestSubmit();
  }
</script>

<div class="flex flex-row gap-1">
  <select {onchange} bind:value={orderByField} class="select">
    {#each choices as { label, value } (value)}
      <option {value}>Sort by: {label}</option>
    {/each}
  </select>
  <button
    onclick={async () => {
      orderByDescending = !orderByDescending;
      await onchange();
    }}
  >
    {#if orderByDescending}
      <ArrowDownNarrowWide />
    {:else}
      <ArrowUpNarrowWide />
    {/if}
  </button>
</div>
