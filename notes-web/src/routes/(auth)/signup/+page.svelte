<script lang="ts">
	import type { PageProps } from './$types';

	let { form }: PageProps = $props();
</script>

<section class="hero is-fullheight">
	<div class="hero-body">
		<div class="container">
			<div class="columns is-centered">
				<div class="column is-one-third">
					<h1 class="title">Sign up</h1>

					<form method="POST">
						<div class="field">
							<label class="label" for="username"> Choose a username </label>
							<div class="control has-icons-left">
								<input
									class="input"
									id="username"
									name="username"
									type="text"
									placeholder="e.g. johndoe"
									minlength="6"
									value={form?.username ?? ''}
									required
								/>
								<span class="icon is-small is-left">
									<i class="fas fa-user"></i>
								</span>
							</div>
						</div>

						{#if form?.message}
							<div class="field">
								<div class="message is-danger">
									<div class="message-body">
										{form.message}
									</div>
								</div>
							</div>
						{/if}

						<div class="field">
							<div class="control">
								<button class="button is-primary is-fullwidth" type="submit">
									Sign up
								</button>
							</div>
						</div>

						<div class="field has-text-right">
							or <a href="/signin">sign in</a>
						</div>
					</form>
				</div>
			</div>
		</div>
	</div>
</section>
