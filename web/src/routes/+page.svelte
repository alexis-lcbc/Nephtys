<script lang="ts">
	import Hls from "hls.js";
	import { onMount } from "svelte";

    let video_elm: HTMLVideoElement | undefined = $state();
    let video_src = "http://localhost:8080/static/stream/stream.m3u8"
    onMount(() => {
        if (Hls.isSupported() && video_elm) {
            var hls = new Hls();
            hls.loadSource(video_src);
            hls.attachMedia(video_elm);
        }
    })
</script>

<div class="bg-gray-900 text-white w-full h-full p-5">
    <div class="flex items-center justify-center flex-col">
        <h1 class="text-4xl">Nephtys Camera Software</h1>
        <!-- svelte-ignore a11y_media_has_caption -->
        <video bind:this={video_elm} controls></video>
    </div>
</div>
