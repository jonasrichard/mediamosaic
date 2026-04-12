<script lang="ts">
	import { page } from '$app/state';

	import type { Thumbnail, DirectoryEntry } from '$lib/types';
	import Breadcrumb from '$lib/breadcrumb.svelte';
	import Directory from '$lib/directory.svelte';
	import ImageGrid from '$lib/imagegrid.svelte';

	const path = $derived(page.params.path || '/');

	let thumbnails = $state([] as Thumbnail[]);
	let entries = $state([] as DirectoryEntry[]);

	let hasThumbnails = $state(false);
	let generateThumbnails = $state(false);

	function mapDirectoryEntry(item: any): DirectoryEntry {
		return {
			name: item.name,
			entryType: item.entry_type,
			size: item.size
		};
	}

	function mapThumbnail(item: any): Thumbnail {
		return {
			relativeBasePath: item.relative_base_path,
			absoluteBasePath: item.absolute_base_path,
			thumbnailName: item.thumbnail_name,
			positionX: item.position_x,
			width: item.width,
			height: item.height,
			originalName: item.original_name,
			fileSize: item.file_size,
			selected: false
		};
	}

	$effect(() => {
		fetch(`/api/info${path}`)
			.then((res) => {
				if (res.ok) {
					hasThumbnails = true;
					return res.json();
				}
			})
			.then((data) => {
				if (data) {
					if (data.length === 0) {
						hasThumbnails = false;
						return data;
					}

					console.log(data[0]);

					if (data[0].entry_type) {
						hasThumbnails = false;
						entries = data
							.map(mapDirectoryEntry)
							.sort((a: DirectoryEntry, b: DirectoryEntry) => {
								if (a.entryType === b.entryType) {
									return a.name.localeCompare(b.name);
								} else if (a.entryType === 'directory') {
									return -1;
								} else {
									return 1;
								}
							});
					} else {
						hasThumbnails = true;
						thumbnails = data
							.map(mapThumbnail)
                    		.sort((a: Thumbnail, b: Thumbnail) => a.originalName.localeCompare(b.originalName));
					}

				}
			})
			.catch((err) => {
				console.error('Error checking for thumbnail:', err);
			});
	});

	$effect(() => {
		if (generateThumbnails) {
			fetch(`/api/directory/thumbnail/${path}`)
				.then((res) => res.text())
				.then((data) => {
					console.log(data);
					generateThumbnails = false;
					// Re-fetch info to get the newly generated thumbnails
					fetch(`/api/info${path}`).then(res => res.json()).then(data => {
						thumbnails = data.map(mapThumbnail).sort((a: Thumbnail, b: Thumbnail) => a.originalName.localeCompare(b.originalName));
						hasThumbnails = true;
					});
				})
				.catch((err) => {
					console.error('Error generating thumbnails:', err);
					generateThumbnails = false;
				});
		}
	});
</script>

<Breadcrumb path={path} bind:generateThumbnails/>

{#if hasThumbnails}
	<ImageGrid bind:thumbnails />
{:else}
	<Directory containingPath={path} {entries} />
{/if}
