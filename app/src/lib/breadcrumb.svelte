<script lang="ts">
	import wrenchIcon from '$lib/assets/wrench-tool.png';

    let { generateThumbnails = $bindable(), path } = $props();

	function breadcrumbs() {
		const parts = path.split('/').filter((p: any) => p);
		const crumbs = [];
		let currentPath = '';
		for (const part of parts) {
			currentPath += `/${part}`;
			crumbs.push({ name: part, path: currentPath });
		}
		return crumbs;
	}
</script>

<style>
	#breadcrumb {
		border: 1px solid var(--border-color);
		display: flex;
		justify-content: space-between;
		font-size: 0.9em;
		padding: 1em;
	}
	#breadcrumb a {
		color: var(--link-color);
		text-decoration: none;
	}
	#breadcrumb a:hover {
		text-decoration: underline;
	}
</style>

<div id="breadcrumb">
	<div>
		<a href="/directory">Root</a> / 
		{#each breadcrumbs() as crumb, index}
			<a href={`/directory${crumb.path}`}>{crumb.name}</a>{index < breadcrumbs().length - 1 ? ' / ' : ''}
		{/each}
	</div>
	<div>
		<a href="/" onclick={(e) => { e.preventDefault(); generateThumbnails = true; }}>
			<img src={wrenchIcon} alt="Generate thumbnails" class="icon" />
			Generate thumbnails
		</a>
	</div>
</div>