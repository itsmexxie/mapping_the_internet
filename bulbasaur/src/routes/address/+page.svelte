<script lang="ts">
	import countries from "$lib/countries";
	import {PUBLIC_API_URL} from "$env/static/public";

	interface Address {
		id: string;
		allocation_state: string;
		allocation_state_comment: string | null;
		routed: boolean;
		online: boolean;
		top_rir: string;
		rir: string;
		autsys: string;
		country: string;
		updated_at: string;
	}

	function booleanHandler(value: boolean): string {
		return value ? 'Ano' : 'Ne';
	}

	function allocStateHandler(value: string): string {
		return {
			unallocated: "nealokovaná",
			reserved: "rezervovaná",
			allocated: "alokovaná",
		}[value] ?? "neznámý";
	}

	function rirHandler(value: string): string {
		return (
			{
				arin: 'ARIN',
				ripencc: 'RIPENCC',
				apnic: 'APNIC',
				other: 'Ostatní',
				unknown: '??'
			}[value] ?? '??'
		);
	}

	let address: Address | null = $state(null);
	let addressValue = $state('');
	let addressDelay: number;

	async function updateAddressInfo() {
		if (/^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4}$/.test(addressValue)) {
			try {
				let res = await fetch(`${PUBLIC_API_URL}/address/${addressValue}`);
				address = await res.json();
			} catch (err) {
				// error handling
			}
		}
	}

	function onAddressInput() {
		clearTimeout(addressDelay);
		addressDelay = setTimeout(updateAddressInfo, 300);
	}
</script>

<div class="container-xl">
	<div class="row justify-content-center mb-3">
		<div class="col-6">
			<input
				class="form-control"
				placeholder="Enter an address..."
				bind:value={addressValue}
				oninput={onAddressInput}
			/>
		</div>
	</div>
	<div class="row justify-content-center">
		<div class="col-6">
			{#if address}
				<table class="table">
					<thead>
						<tr>
							<th>Property</th>
							<th>Value</th>
						</tr>
					</thead>
					<tbody>
						<tr>
							<td>Adresa</td>
							<td>{address.id}</td>
						</tr>
						<tr>
							<td>Stav alokace</td>
							<td>{allocStateHandler(address.allocation_state)}</td>
						</tr>
						<tr>
							<td>Směrovaná</td>
							<td>{booleanHandler(address.routed)}</td>
						</tr>
						<tr>
							<td>Online</td>
							<td>{booleanHandler(address.online)} {address.online ? '🟩' : '🟥'}</td>
						</tr>
						<tr>
							<td>Vrchní RIR</td>
							<td>{rirHandler(address.top_rir)}</td>
						</tr>
						<tr>
							<td>RIR</td>
							<td>{rirHandler(address.rir)}</td>
						</tr>
						<tr>
							<td>Autonomní systém</td>
							<td>{address.autsys ?? "??"}</td>
						</tr>
						<tr>
							<td>Země</td>
							<td>{countries[address.country]?.name ?? '??'} {countries[address.country]?.emoji}</td>
						</tr>
						<tr>
							<td class="text-body-secondary">Aktualizováno</td>
							<td class="text-body-secondary">{address.updated_at}</td>
						</tr>
					</tbody>
				</table>
			{/if}
		</div>
	</div>
</div>
