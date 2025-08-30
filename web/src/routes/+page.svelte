<script lang="ts">
	import EventItem from "$lib/components/event_item.svelte";
	import Hls from "hls.js";
	import { onMount } from "svelte";

    let events: {events: {start: string, end: string, filename: string}[]} = $state({events: []});
    let events_list: {start: string, end: string, filename: string}[] = $derived(events.events);

    let video_elm: HTMLVideoElement | undefined = $state();
    let video_src = "/api/protected/stream/stream.m3u8"
    onMount(async () => {

        let check_setup = await fetch('/api/check_setup')
        if (await check_setup.text() == "setup") {
            window.location.href = "/setup"
        }

        let check_token = await fetch('/api/protected/check')
        if (!check_token.ok) {
            window.location.href = "/login"
        }

        if (Hls.isSupported() && video_elm != undefined) {
            var hls = new Hls({
                liveSyncMode: "edge",
                liveSyncDurationCount: 1,
                enableWorker: true,
                liveBackBufferLength: 0,
                maxBufferLength: 1,
                liveSyncDuration: 0,
                liveMaxLatencyDuration: 5,
                liveDurationInfinity: true,
                highBufferWatchdogPeriod: 1,
            });
            hls.loadSource(video_src);
            hls.attachMedia(video_elm);
            hls.on((Hls.Events.MEDIA_ATTACHED), () => {
                video_elm?.play();
            })
        }

        setInterval(poll_events, 5000)
        
    })

    async function poll_events() {
        let check_setup = await fetch('/api/protected/clips/index.json')
        let list_json = await check_setup.json();
        console.log(list_json);
        events = list_json;
    }
</script>

<div class="bg-gray-800 text-white w-full h-min-full">
    <div class="flex items-center justify-center flex-col p-5">
        <h1 class="text-4xl">Nephtys Camera Software</h1>
        <!-- svelte-ignore a11y_media_has_caption -->
        <video class="rounded-2xl m-3 bg-gray-950" bind:this={video_elm} autoplay muted></video>
        <h1 class="text-4xl">Last detected movements</h1>
        <div class="flex flex-row-reverse items-center justify-center flex-wrap">
            {#each events_list as event}
                <EventItem filename={event.filename} start_time={new Date(event.start)} stop_time={new Date(event.end)}></EventItem>
            {/each}
        </div>
    </div>
</div>
