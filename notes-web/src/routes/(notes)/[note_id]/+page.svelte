<script lang="ts">
	import showdown from 'showdown';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let markdown_converter = new showdown.Converter();
	let markdown_html = markdown_converter.makeHtml(data.note.markdown);
</script>

{@html markdown_html}

<div class="has-text-right">
	<a class="button" href={`/${data.note.id}/edit`}> Edit </a>
</div>
