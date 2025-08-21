<script lang="ts">
</script>

<form method="POST" class="editor">
	<textarea name="markdown"></textarea>

	<div class="has-text-right">
		<a class="button" href="/"> Cancel </a>
		<button class="button is-primary" type="submit"> Save </button>
	</div>
</form>

<style>
	.editor {
		display: flex;
		flex-flow: column;
		width: 100%;
		height: 100%;
	}

	textarea {
		all: unset;
		flex: 1;
	}
</style>
