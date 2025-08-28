<script lang="ts">
    let {start_time, stop_time, filename}: {start_time: Date, stop_time: Date, filename: string} = $props()
</script>

<div class="bg-gray-950 w-1/2 aspect-video rounded-2xl flex flex-col p-3 m-3">
    <h1 class="text-2xl">Movement detected</h1>
    <p class="text-gray-400">{Intl.DateTimeFormat(navigator.language, {timeStyle: 'medium', dateStyle: 'short'}).format(start_time)} - {Intl.DateTimeFormat(navigator.language, {timeStyle: 'medium', dateStyle: 'short'}).format(stop_time)} </p>
    <!-- svelte-ignore a11y_media_has_caption -->
    <video class="m-2 rounded-2xl" src="/api/protected/clips/{filename}.mkv" controls></video>
</div>