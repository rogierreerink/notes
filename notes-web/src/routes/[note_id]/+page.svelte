<script lang="ts">
	import { page } from '$app/state';
	import showdown from 'showdown';

	let markdown_converter = new showdown.Converter();
	let markdown_html = markdown_converter.makeHtml(page.data.note.markdown);
</script>

<div class="columns is-gapless" style="height: 100%;">
	<!-- Sidebar -->
	<aside class="column is-3" style="border-right: 1px solid hsl(0, 0%, 20%);">
		<nav class="panel" style="box-shadow: none;">
			{#each page.data.notes as note (note.id)}
				<a
					class="panel-block"
					class:is-active={note.id === page.params.id}
					href={`/${note.id}`}
					data-sveltekit-reload
				>
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
					{@html markdown_html}
				</div>
			</div>
		</div>
	</main>
</div>

<style>
	:global(h1) {
		font-size: xx-large;
	}

	:global(p) {
		margin: 0.8em 0;
	}
</style>
