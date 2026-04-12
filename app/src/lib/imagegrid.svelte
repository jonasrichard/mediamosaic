<script lang="ts">
    import type { Thumbnail } from '$lib/types';
    import ThumbnailComponent from '$lib/thumbnail.svelte';
	import { bytesToKB } from '$lib/util';
	import dustbinIcon from '$lib/assets/dustbin.png';

    let { thumbnails = $bindable() }: { thumbnails: Thumbnail[] } = $props();
	let selectedCount = $derived.by(() => thumbnails.filter((t: Thumbnail) => t.selected).length);
	let selectedSize = $derived.by(() => thumbnails.filter((t: Thumbnail) => t.selected).reduce((sum, t) => sum + t.fileSize, 0));
	let totalSize = $derived.by(() => thumbnails.reduce((sum, t) => sum + t.fileSize, 0));
	let selectedIndex = $state(-1);
	let mainImageSrc = $state('');

	function navigate(delta: number) {
		if (thumbnails.length === 0) return;
		selectedIndex = (selectedIndex + delta + thumbnails.length) % thumbnails.length;
		const thumb = thumbnails[selectedIndex];
		mainImageSrc = `/api/file/serve/${thumb.relativeBasePath}/${thumb.originalName}`;
	}

	function handleKeydown(e: KeyboardEvent) {
		if (!mainImageSrc) return;
		if (e.key === 'Escape') {
			mainImageSrc = '';
			selectedIndex = -1;
		} else if (e.key === 'ArrowLeft') {
			e.preventDefault();
			navigate(-1);
		} else if (e.key === 'ArrowRight') {
			e.preventDefault();
			navigate(1);
		} else if (e.key === 's') {
            e.preventDefault();
            if (selectedIndex >= 0) {
                thumbnails[selectedIndex].selected = !thumbnails[selectedIndex].selected;
            }
        }
	}

	async function deleteSelected() {
		const selected = thumbnails.filter((t) => t.selected);
		if (selected.length === 0) return;

        if (!confirm(`Are you sure you want to delete ${selected.length} selected image(s)? This action cannot be undone.`)) {
            return;
        }

		const paths = selected.map((t) => `${t.relativeBasePath}/${t.originalName}`);
		const res = await fetch('/api/delete', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(paths)
		});

		if (res.ok) {
			thumbnails = thumbnails.filter((t) => !t.selected);
			if (selectedIndex >= thumbnails.length) {
				selectedIndex = thumbnails.length - 1;
			}
		} else {
			console.error('Failed to delete images:', await res.text());
		}
	}
</script>

<style lang="css">
	.toolbar {
		background-color: #1f1b1b;
		border: 1px solid var(--border-color);
		display: flex;
		flex-direction: row;
		gap: 0.5em;
		justify-content: space-between;
		padding: 0.5em;
		position: sticky;
		top: 0;
	}

	.thumbnails {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
        justify-content: space-evenly;
	}

	#main-image-container {
		background-color: #555;
		padding: 2em;
		position: fixed;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		z-index: 1000;
		line-height: 0;
	}

	#main-image {
		display: block;
		max-width: calc(95vw - 4em);
		max-height: calc(95vh - 4em);
	}
</style>

<svelte:window onkeydown={handleKeydown} />

<div class="toolbar">
	<div>
		<span>{selectedCount} selected ({bytesToKB(selectedSize)})</span>
		<a
			href="/"
			style:margin-left="1em"
			onclick={(e) => { e.preventDefault(); deleteSelected(); }}>
			<img src={dustbinIcon} alt="Delete selected" class="icon" /> Delete selected...
		</a>
	</div>

	<div>
		Total images: {thumbnails.length},
		Total size: {bytesToKB(totalSize)}
	</div>
</div>

<div id="main-image-container" style:display={mainImageSrc ? 'block' : 'none'}>
    <a
        aria-label="Dismiss image"
		href="/"
        onclick={(e) => {
            mainImageSrc = '';
            selectedIndex = -1;
			e.preventDefault();
        }}>
        <img
            id="main-image"
            src={mainImageSrc}
            alt="Main image at {mainImageSrc}"
        />
    </a>
	{#if selectedIndex >= 0}
		{thumbnails[selectedIndex].originalName} ({bytesToKB(thumbnails[selectedIndex].fileSize)})
	{/if}
</div>

<div class="thumbnails">
    {#each thumbnails as thumbnail, index}
        <ThumbnailComponent bind:thumbnail={thumbnails[index]} {index} bind:mainImageSrc bind:selectedIndex />
    {/each}
</div>