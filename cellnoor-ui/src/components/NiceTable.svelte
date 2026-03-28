<script lang="ts">
  const {
    fieldNames,
    data,
    extractDatum,
  }: {
    fieldNames: string[];
    data: Record<string, unknown>[];
    extractDatum: (fieldName: string, row: Record<string, unknown>) => unknown;
  } = $props();
</script>

<table class="table bg-base-200 rounded">
  <thead>
    <tr>
      {#each fieldNames as fieldName (fieldName)}
        <th class="text-neutral">{fieldName}</th>
      {/each}
    </tr>
    {#each data as row (row)}
      <tr>
        {#each fieldNames as fieldName (fieldName)}
          <td class="text-neutral font-normal">{extractDatum(fieldName, row)}</td>
        {/each}
      </tr>
    {/each}
  </thead>
</table>
