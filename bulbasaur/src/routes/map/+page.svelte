<script lang="ts">
	import {PUBLIC_API_URL} from "$env/static/public";

    let firstOctet = $state(1);
    let secondOctet = $state(1);
    let thirdOctet = $state(1);
    let address = $state(0);
    let currentLevel = $state(3);
    let zoomLevel = $state(3);

    let data: { prefix: number, data: any }[] = [];

    function levelToPrefix(level: number): number {
        return [8, 16, 24, 32][level];
    }

    function ipBinaryToOctet(value: number): number[] {
        return [(value >> 24) & 255, (value >> 16) & 255, (value >> 8) & 255, (value) & 255];
    }

    function ipOctetToBinary(first: number, second: number, third: number, fourth: number): number {
        return (first << 24) + (second << 16) + (third << 8) + fourth;
    }

    function updateAddress() {
        address = ipOctetToBinary(firstOctet, secondOctet, thirdOctet, 0);
    }

    function fetchData() {
        updateAddress();

        let start = address;
        data = [];

        for (let i = 0; i < 256; i++) {
            let new_address = start + (i << levelToPrefix(currentLevel));
            let octets = ipBinaryToOctet(new_address);
            fetch(`${PUBLIC_API_URL}/map/${octets[0]}.${octets[1]}.${octets[2]}.${octets[3]}/32`).then((body) => {
                body.json().then((res) => {
                    data.push({ prefix: octets[currentLevel], data: res });

                    if (data.length >= 256) {
                        data.sort((a, b) => { return a.prefix - b.prefix });
                        data = data.map(x => x.data);
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
                    <input type="number" class="form-control" bind:value={firstOctet} oninput={updateAddress}>
                    {:else}
                    <span>{firstOctet}</span>
                    {/if}
                </div>
                <div class="col-3 d-flex align-items-center">
                    {#if currentLevel > 1}
                    <input type="number" class="form-control" bind:value={secondOctet} oninput={updateAddress}>
                    {:else}
                    <span>{secondOctet}</span>
                    {/if}
                </div>
                <div class="col-3 d-flex align-items-center">
                    {#if currentLevel > 2}
                    <input type="number" class="form-control" bind:value={thirdOctet} oninput={updateAddress}>
                    {:else}
                    <span>{thirdOctet}</span>
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
                <span class="ms-3">/{ levelToPrefix(currentLevel) }</span>
            </div>
            <input type="range" id="currentLevel" class="form-range" min="0" max="3" bind:value={currentLevel} oninput={() => zoomLevel = currentLevel}>
        </div>
        <div class="mb-3">
            <div>
                <label for="zoomLevel" class="form-label">Zoom level</label>
                <span class="ms-3">/{ levelToPrefix(zoomLevel) }</span>
            </div>
            <input type="range" id="zoomLevel" class="form-range" min={currentLevel} max="3" bind:value="{zoomLevel}" disabled={currentLevel > 2}>
        </div>
        <div>
            <button class="btn btn-primary" onclick={fetchData}>Render</button>
        </div>
    </div>
    <div id="canvas">

    </div>
    <div id="info">

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
    }
</style>
