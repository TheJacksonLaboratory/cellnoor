<script lang="ts">
  import { libraryTypeMap, multiplexingTypeMap } from "$lib/string-maps";
  import Header from "../../Header.svelte";
  import LibraryTypeBadges from "../../LibraryTypeBadges.svelte";
  import LinkButton from "../../LinkButton.svelte";

  const { data } = $props();

  const {
    dataset: {
      name,
      lab,
      assay,
      links,
      delivered_at,
    },
    specimens,
    libraries,
  } = $derived(data);
</script>

<Header header={name} />

<!-- TODO: factor this layout into a component -->

<div class="grid md:grid-cols-3 gap-4 lg:max-w-4/5 mx-auto align-top">
  <div
    class="p-4 border border-base-300 rounded-box min-w-full col-span-2"
    id="metrics"
  >
    <h2 class="text-3xl font-semibold">Metrics</h2>
    <p>this box is under construction. sorry :)</p>
  </div>
  <div class="col-span-1">
    <span class="font-bold">{assay.name}:</span> <span>({
        assay.chemistry_version
      })</span>
    <LibraryTypeBadges libraryTypes={assay.library_types} />
    <div class="divider"></div>
    <LinkButton link={assay.protocol_url} buttonText={"Protocol"} />
    <LinkButton link={assay.protocol_url} buttonText={"Protocol"} />
  </div>
  <div
    class="p-4 border border-base-300 rounded-box min-w-full col-span-2"
    id="specimens"
  >
    <h2 class="text-3xl font-semibold">Specimens</h2>
    <p>Here's another box under construction</p>
  </div>
  <div
    class="p-4 border border-base-300 rounded-box min-w-full col-span-2"
    id="specimens"
  >
    <h2 class="text-3xl font-semibold">Libraries</h2>
    <p>and yeet one more!</p>
  </div>
</div>
