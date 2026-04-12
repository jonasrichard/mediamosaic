<script lang="ts">
import { page } from '$app/state';
</script>

This is the image page for {page.params.path}

<img src="/api/file/serve/{page.params.path}" alt="Image at {page.params.path}" />