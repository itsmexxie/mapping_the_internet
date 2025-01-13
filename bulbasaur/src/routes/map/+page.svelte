<script lang="ts">
	import * as hilbert from 'hilbert-curve';
	import map from '$lib/map';
	import ip from '$lib/ip';
	import { PUBLIC_API_URL } from '$env/static/public';

    let octets = $state([1, 1, 1]);
	let address = $state(0);
	let currentLevel = $state(3);
	let zoomLevel = $state(3);

	let data: { allocation_state: string; routed: boolean; online: boolean }[] = $state([]);

	function levelToPrefix(level: number): number {
		return [8, 16, 24, 32][level];
	}

    function updateCurrentLevel() {
        zoomLevel = currentLevel;

        for (let i = currentLevel; i < 3; i++) {
            octets[i] = 0;
        }
    }

	function updateAddress() {
		address = ip.octetToBinary(octets[0], octets[1], octets[2], 0);
	}

	function fetchData() {
		data = [];
		let _data: { prefix: number; data: any }[] = [];

		let start = address;
		for (let i = 0; i < 256; i++) {
			let new_address = start + (i << levelToPrefix(currentLevel));
			let new_octets = ip.binaryToOctet(new_address);
			fetch(
				`${PUBLIC_API_URL}/map/${new_octets[0]}.${new_octets[1]}.${new_octets[2]}.${new_octets[3]}/32`
			).then((body) => {
				body.json().then((res) => {
					_data.push({ prefix: new_octets[currentLevel], data: res });

					if (_data.length >= 256) {
						_data.sort((a, b) => {
							return a.prefix - b.prefix;
						});
						data = _data.map((x) => x.data);
					}
				});
			});
		}
	}
</script>

<div id="map">
	<div id="tools" class="p-3">
		<div class="mb-3">
			<div class="row g-0">
				<div class="col-3 d-flex align-items-center">
					{#if currentLevel > 0}
						<input
							type="number"
							class="form-control"
							bind:value={octets[0]}
							oninput={updateAddress}
						/>
					{:else}
						<span>{octets[0]}</span>
					{/if}
				</div>
				<div class="col-3 d-flex align-items-center">
					{#if currentLevel > 1}
						<input
							type="number"
							class="form-control"
							bind:value={octets[1]}
							oninput={updateAddress}
						/>
					{:else}
						<span>{octets[1]}</span>
					{/if}
				</div>
				<div class="col-3 d-flex align-items-center">
					{#if currentLevel > 2}
						<input
							type="number"
							class="form-control"
							bind:value={octets[2]}
							oninput={updateAddress}
						/>
					{:else}
						<span>{octets[2]}</span>
					{/if}
				</div>
				<div class="col-3 d-flex align-items-center">
					<span class="3">0</span>
				</div>
			</div>
		</div>
		<div class="mb-3">
			<div>
				<label for="currentLevel" class="form-label">Current level</label>
				<span class="ms-3">/{levelToPrefix(currentLevel)}</span>
			</div>
			<input
				type="range"
				id="currentLevel"
				class="form-range"
				min="0"
				max="3"
				bind:value={currentLevel}
				oninput={updateCurrentLevel}
			/>
		</div>
		<div class="mb-3">
			<div>
				<label for="zoomLevel" class="form-label">Zoom level</label>
				<span class="ms-3">/{levelToPrefix(zoomLevel)}</span>
			</div>
			<input
				type="range"
				id="zoomLevel"
				class="form-range"
				min={currentLevel}
				max="3"
				bind:value={zoomLevel}
				disabled={currentLevel > 2}
			/>
		</div>
		<div>
			<button
				class="btn btn-primary"
				onclick={() => {
					updateAddress();
					fetchData();
				}}>Render</button
			>
		</div>
	</div>
	<div id="canvas">
		{#if data.length > 0}
			{#each { length: 16 }, y}
				{#each { length: 16 }, x}
					{@const currAddress = data[hilbert.pointToIndex({ x, y }, 4)]}
					<div
						class="border"
						style:background={map.colorMap[
							map.mapToColorId(
								currAddress.allocation_state,
								currAddress.routed,
								currAddress.online
							)
						].value}
					></div>
				{/each}
			{/each}
		{/if}
	</div>
	<div id="info" class="p-3">
        <h3>Legenda</h3>
        {#each Object.entries(map.colorMap) as [colorMapItemId, colorMapItemDef]}
        <div class="d-flex align-items-center">
            <span>{colorMapItemDef.display}:</span>
            <div style:background={colorMapItemDef.value} style="width: 16px; height: 16px" class="ms-3"></div>
        </div>
        {/each}
    </div>
</div>

<style lang="scss">
	#map {
		height: calc(100vh - 56px);
		display: grid;
		grid-template-columns: 1fr calc(100vh - 56px) 1fr;
		grid-template-rows: 1fr;
		grid-column-gap: 0px;
		grid-row-gap: 0px;
	}

	#canvas {
		background: black;
		display: grid;
		grid-template-columns: repeat(16, 1fr);
		grid-template-rows: repeat(16, 1fr);
		grid-column-gap: 0px;
		grid-row-gap: 0px;
	}
</style>
