<script lang="ts">
	import type { LayoutProps } from './$types';

	let { children, data }: LayoutProps = $props();
</script>

<div class="app">
	<nav
		class="navbar"
		aria-label="main navigation"
		style="border-bottom: 1px solid hsl(0, 0%, 20%);"
	>
		<form method="POST" class="navbar-menu">
			<div class="navbar-end">
				<div class="navbar-item">
					<div class="buttons">
						<button class="button is-small" formaction="/signout">
							Sign out
						</button>
					</div>
				</div>
			</div>
		</form>
	</nav>

	<div class="main columns is-gapless">
		<!-- Sidebar class:is-active={note.id === page.params.id} -->
		<aside class="column is-3" style="border-right: 1px solid hsl(0, 0%, 20%);">
			<nav class="panel" style="box-shadow: none;">
				<a class="panel-block" href={'/create'}>
					<i>+ create note</i>
				</a>

				{#each data.notes as note (note.id)}
					<a class="panel-block" href={`/${note.id}`}>
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

	.app {
		height: 100%;
		display: flex;
		flex-flow: column;
	}

	.main {
		flex: 1;
	}

	.section,
	.container,
	.content {
		height: 100%;
	}
</style>
