<script lang="ts">
    import "../style/main.scss";

	let { children } = $props();
</script>

<nav class="navbar navbar-expand-lg bg-body-tertiary">
	<div class="container-fluid">
		<a class="navbar-brand" href="/">
			Mapping the Internet
		  </a>
		<button
			class="navbar-toggler"
			type="button"
			data-bs-toggle="collapse"
			data-bs-target="#navbarSupportedContent"
			aria-controls="navbarSupportedContent"
			aria-expanded="false"
			aria-label="Toggle navigation"
		>
			<span class="navbar-toggler-icon"></span>
		</button>
		<div class="collapse navbar-collapse" id="navbarSupportedContent">
			<ul class="navbar-nav">
				<li class="nav-item">
					<a class="nav-link" href="/address">Adresy</a>
				</li>
				<li class="nav-item">
					<a class="nav-link" href="/map">Mapa</a>
				</li>
			</ul>
		</div>
	</div>
</nav>

{@render children()}
