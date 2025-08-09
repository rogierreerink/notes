<script lang="ts">
	import type { PageProps } from './$types';

	let { form }: PageProps = $props();
</script>

<section class="hero is-fullheight">
	<div class="hero-body">
		<div class="container">
			<div class="columns is-centered">
				<div class="column is-one-third">
					<h1 class="title">Set password</h1>

					<form method="POST">
						<div class="field">
							<label class="label" for="password"> Set a password </label>
							<div class="control has-icons-left">
								<input
									class="input"
									id="password"
									name="password"
									type="password"
									placeholder="********"
									minlength="6"
									required
								/>
								<span class="icon is-small is-left">
									<i class="fas fa-key"></i>
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
									Save
								</button>
							</div>
						</div>
					</form>
				</div>
			</div>
		</div>
	</div>
</section>
