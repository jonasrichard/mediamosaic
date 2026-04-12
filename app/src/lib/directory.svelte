<script lang="ts">
	import type { DirectoryEntry } from '$lib/types';
	import { joinPaths } from '$lib/util';

	let { entries, containingPath }: { entries: DirectoryEntry[], containingPath: string } = $props();
</script>

<style lang="css">
	.directory {
		display: flex;
		flex-direction: column;
		margin: 0em 2em;
	}

	.directory .item {
		margin: 0.25em 0;
	}

	.directory a {
		border-radius: 0.25em;
		color: var(--link-color);
		padding: 0.25em 0.5em;
		text-decoration: none;
	}
</style>

<div class="directory">
	{#each entries as entry}
		<div class="item">
			{#if entry.entryType === 'directory'}
				<a href={joinPaths('/directory', containingPath, entry.name)}>{entry.name}</a>
			{:else}
				<a href={joinPaths('/image', containingPath, entry.name)}
					>{joinPaths(containingPath, entry.name)}</a
				>
			{/if}
		</div>
	{/each}
</div>

