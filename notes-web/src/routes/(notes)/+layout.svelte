<script lang="ts">
	import type { LayoutProps } from './$types';

	let { children, data }: LayoutProps = $props();
</script>

<div class="columns is-gapless" style="height: 100%;">
	<!-- Sidebar class:is-active={note.id === page.params.id} -->
	<aside class="column is-3" style="border-right: 1px solid hsl(0, 0%, 20%);">
		<nav class="panel" style="box-shadow: none;">
			{#each data.notes as note (note.id)}
				<a class="panel-block" href={`/${note.id}`} data-sveltekit-reload>
					<span class="panel-icon">
						<i class="fas fa-sticky-note" aria-hidden="true"></i>
					</span>
					<span>
						<strong>{note.title}</strong>
					</span>
				</a>
			{/each}
		</nav>
	</aside>

	<!-- Main Content -->
	<main class="column">
		<div class="section mx-6">
			<div class="container">
				<div class="content">
					{@render children()}
				</div>
			</div>
		</div>
	</main>
</div>

<style>
	:global(body) {
		height: 100vh;
	}

	:global(h1) {
		font-size: xx-large;
	}

	:global(p) {
		margin: 0.8em 0;
	}

	.section,
	.container,
	.content {
		height: 100%;
	}
</style>
