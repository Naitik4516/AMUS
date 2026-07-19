<script lang="ts">
    import { Play } from "@lucide/svelte";
    import PlaylistCoverArt from "$components/ui/PlaylistCoverArt.svelte";

    let { data } = $props();
</script>

<a
    href="/library/playlists/{data.id}?data.name={data.name}"
    class="group flex flex-col gap-3 p-5 rounded-3xl bg-card transition-all duration-300 ring ring-border hover:ring-2 w-60 h-auto shadow-xl hover:shadow-card"
>
    <div class="aspect-square w-full rounded-3xl overflow-hidden relative">
        <PlaylistCoverArt playlist={data} />

        <div
            class="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center"
        >
            <div
                class="bg-accent text-black p-4 rounded-full shadow-xl transform translate-y-4 group-hover:translate-y-0 transition-transform"
            >
                <Play size={20} fill="var(--color-accent-foreground)" />
            </div>
        </div>
    </div>

    <div class="flex flex-col">
        <h3 class="font-bold truncate text-white text-lg">{data.name}</h3>
    </div>
</a>
