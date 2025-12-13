<script lang="ts">
  let { data } = $props();
</script>

<div>
  <table class="table">
    <thead>
      <tr>
        <td>
          Name
        </td>
        <td>
          Delivered At
        </td>
      </tr>
    </thead>
    <tbody>
      {#each data.chromiumDatasets as ds}
        <tr>
          <td>
            <button class="btn btn-primary">{ds.name}</button>
          </td>
          <td>
            {ds.delivered_at}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
