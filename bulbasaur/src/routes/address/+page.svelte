<script lang="ts">
	import countries from "$lib/countries";
	import {PUBLIC_API_URL} from "$env/static/public";

	interface Address {
		id: string;
		allocation_state_id: string;
		allocation_state_comment: string | null;
		routed: boolean;
		online: boolean;
		top_rir_id: string;
		rir_id: string;
		autsys_id: string;
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
    let message = $state("");

	async function updateAddressInfo() {
		if (/^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4}$/.test(addressValue)) {
			try {
				let res = await fetch(`${PUBLIC_API_URL}/address/${addressValue}`);
                switch (res.status) {
                    case 200:
                        address = (await res.json())[0];
                        break;

                    case 404:
                        address = null;
                        message = "Pro tuto adresu nebyl nalezen žádný záznam..."
                }
			} catch (err) {
                console.error(err);
			}
		} else {
            address = null;
            message = "Zadejte platnou IPv4 adresu!";
        }
	}

	function onAddressInput() {
		clearTimeout(addressDelay);
		addressDelay = setTimeout(updateAddressInfo, 300);
	}
</script>

<div class="container-xl p-3">
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
							<td>{allocStateHandler(address.allocation_state_id)}</td>
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
							<td>{rirHandler(address.top_rir_id)}</td>
						</tr>
						<tr>
							<td>RIR</td>
							<td>{rirHandler(address.rir_id)}</td>
						</tr>
						<tr>
							<td>Autonomní systém</td>
							<td>{address.autsys_id ?? "??"}</td>
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
			{:else}
                <p>{message}</p>
            {/if}
		</div>
	</div>
</div>
