<script lang="ts">
	import type { Thumbnail } from '$lib/types';
	import { bytesToKB } from '$lib/util';
	import selectIcon from '$lib/assets/correct-symbol.png';

	let {
		index,
		mainImageSrc = $bindable(),
		selectedIndex = $bindable(),
		thumbnail = $bindable()
	}: {
		index: number;
		mainImageSrc: string;
		selectedIndex: number;
		thumbnail: Thumbnail;
	} = $props();
</script>

<style lang="css">
	.thumbnail-container {
		background-color: #34312f;
		border-radius: 10px;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: flex-end;
		margin-bottom: 10px;
		padding: 0.2em;
		text-align: center;
		width: 256px;
	}
	.thumbnail-image {
		cursor: pointer;
	}
</style>

<div class="thumbnail-container"
	style:border={thumbnail.selected ? '2px solid red' : '1px solid gray'}>
	<a
		href={`/api/file/serve/${thumbnail.relativeBasePath}/${thumbnail.originalName}`}
		aria-label="View {thumbnail.originalName}"
		onclick={(e) => {
			e.preventDefault();
			mainImageSrc = `/api/file/serve/${thumbnail.relativeBasePath}/${thumbnail.originalName}`;
			selectedIndex = index;
		}}>
		<div
			class="thumbnail-image"
			aria-label="View {thumbnail.originalName}"
			style:background-image="url('/api/file/serve/{thumbnail.relativeBasePath}/{thumbnail.thumbnailName}')"
			style:background-position="-{thumbnail.positionX}px 0px"
			style:background-size="cover"
			style:background-repeat="no-repeat"
			style:width="{thumbnail.width}px"
			style:height="{thumbnail.height}px"
		></div>
	</a>
	<div style:padding-top="0.5em">{thumbnail.originalName.slice(-10)} ({bytesToKB(thumbnail.fileSize)})</div>
	<div>
		<a
			aria-label="Select {thumbnail.originalName}"
			href="/"
			onclick={(e) => {
				e.preventDefault();
				e.stopPropagation();
				thumbnail.selected = !thumbnail.selected;
			}}><img src={selectIcon} alt="Select" class="icon" /> Select</a>
	</div>
</div>
