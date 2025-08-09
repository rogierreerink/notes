<script lang="ts">
	import type { LayoutProps } from './$types';

	let { children }: LayoutProps = $props();
</script>

{@render children()}

<style>
	:global(body) {
		height: 100vh;
	}
</style>
