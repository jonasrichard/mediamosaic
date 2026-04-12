<script lang="ts">
	import './layout.css';
	import homePng from '$lib/assets/home.png';

	let { children } = $props();
</script>

<style lang="css">
	nav {
		background-color: #1f1b1b;
		border: 1px solid var(--border-color);
	}

	ul {
		display: flex;
		gap: 1em;
		padding: 0.3em;
	}

	li {
		list-style: none;
	}

	li.selected a {
		background-color: var(--navbar-link-selected-background);
		padding: 0.25em 0.5em;
		border-radius: 1em;
	}

	a {
		color: var(--navbar-link-color);
		text-decoration: none;
	}

	a:hover {
		color: var(--navbar-link-hover-color);
	}
</style>

<svelte:head></svelte:head>

<nav>
	<ul>
		<li class="selected"><a href="/"><img src={homePng} alt="Home" class="icon" /> Home</a></li>
		<li><a href="/about">About</a></li>
	</ul>
</nav>

{@render children()}
