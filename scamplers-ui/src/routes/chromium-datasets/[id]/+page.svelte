<script lang="ts">
  import { libraryTypeMap, multiplexingTypeMap } from "$lib/string-maps";
  import Header from "../../Header.svelte";

  const { data } = $props();

  const {
    dataset: {
      name,
      lab,
      assay,
      parsed_metrics_files,
      links,
      delivered_at,
    },
    specimens,
    libraries,
  } = $derived(data);
</script>
<Header header={name} />
<div class="px-8 mx-auto flex flex-col items-center lg:max-w-3/4">
  <div class="grid md:grid-cols-2 gap-4">
    <div>
      <div class="py-4 border-b">
        <span class="font-bold mb-2">{assay.name}:</span><span>({
            assay.chemistry_version
          })</span>
        <div class="flex flex-row flex-wrap gap-1">
          {#each assay.library_types as libType}
            <div class="badge badge-neutral">{libraryTypeMap.get(libType)}</div>
          {/each}
        </div>
      </div>
      <a class="px-0 btn btn-ghost" href={assay.protocol_url}>Protocol</a>
    </div>
    <div
      class="p-4 border border-base-300 rounded-box min-w-full"
      id="metrics"
    >
      <h2 class="text-3xl font-semibold">Metrics</h2>
      <p>
        some awesome content skljfsdlkjfskdljfklsdjflksdjfskldfjklsdj
        sklfjdsflkjdskfljsdklfjdsklfjsdklfjkljsdfj sdkfljdskfljdsklfjdkslf
        sdklfjsdlkfjsdklfjsdklfjklj
      </p>
    </div>
  </div>
</div>
