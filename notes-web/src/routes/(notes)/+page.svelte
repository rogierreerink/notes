<script lang="ts">
	import type { PageProps } from './$types';
	import showdown from 'showdown';

	let { data }: PageProps = $props();

	let markdown_converter = new showdown.Converter();
	let markdown_html =
		data.note && markdown_converter.makeHtml(data.note.markdown);
</script>

{#if markdown_html}
	{@html markdown_html}
{:else}
	<h1>Welcome</h1>
	<p>Create your first note!</p>
{/if}
