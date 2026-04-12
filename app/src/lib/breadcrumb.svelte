<script lang="ts">
    let { path } = $props();

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
	<a href="/directory">Root</a> / 
	{#each breadcrumbs() as crumb, index}
		<a href={`/directory${crumb.path}`}>{crumb.name}</a>{index < breadcrumbs().length - 1 ? ' / ' : ''}
	{/each}
</div>