<script lang="ts">
  const { link, buttonText }: { link: string; buttonText: string } =
    $props();
</script>

<a class="btn btn-primary btn-outline" href={link}>{buttonText}</a>
