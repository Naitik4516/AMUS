<script lang="ts">
    import { store } from "$lib/stores.svelte";

    let {
        size,
        profileImage,
        name,
    }: { size: number; profileImage: string | null | undefined; name: string } =
        $props();

    const getArtistPlaceHolder = (name: string) => {
        const cleaned = name.replace(/[^a-zA-Z0-9\s]/g, "").trim();
        const words = cleaned.split(/\s+/).filter(Boolean);

        if (words.length === 0) {
            return "";
        }

        const firstChar = words[0][0].toUpperCase();
        const lastChar = words[words.length - 1][0].toUpperCase();

        return firstChar + lastChar;
    };
</script>

<div
    class="rounded-full bg-zinc-800/60 border border-black/30"
    style="width: {size}px; height: {size}px;"
>
    {#if profileImage}
        <img
            src={store.getImageSrc(profileImage, "artist")}
            alt={name}
            class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
        />
    {:else}
        <div
            class="size-full flex items-center justify-center text-gray-400 font-satohsi font-medium"
            style="font-size: {size / 3}px;"
        >
            {getArtistPlaceHolder(name)}
        </div>
    {/if}
</div>
