<script lang="ts">
	import showdown from 'showdown';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let markdown_converter = new showdown.Converter();
</script>

{@html markdown_converter.makeHtml(data.note.markdown)}

<div class="has-text-right">
	<form method="POST">
		<button class="button has-text-danger" type="submit" formaction="?/delete">
			Delete
		</button>
	</form>

	<a class="button" href={`/${data.note.id}/edit`}> Edit </a>
</div>

<style>
	form {
		display: inline-block;
	}
</style>
